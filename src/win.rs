//! Minimal Windows tray host using only the operating system's Win32 APIs.
//! No GUI framework or runtime DLL is required.

use std::ffi::c_void;
use std::io;
use std::mem::{size_of, zeroed};
use std::path::PathBuf;
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, Ordering};

type Bool = i32;
type Uint = u32;
type Wparam = usize;
type Lparam = isize;
type Lresult = isize;
type Hwnd = *mut c_void;
type Hinstance = *mut c_void;
type Hicon = *mut c_void;
type Hmenu = *mut c_void;

const WM_DESTROY: Uint = 0x0002;
const WM_COMMAND: Uint = 0x0111;
const WM_APP: Uint = 0x8000;
const WM_TRAY: Uint = WM_APP + 1;
const WM_RBUTTONUP: Uint = 0x0205;
const WM_LBUTTONUP: Uint = 0x0202;
const NIM_ADD: Uint = 0;
const NIM_DELETE: Uint = 2;
const NIF_MESSAGE: Uint = 1;
const NIF_ICON: Uint = 2;
const NIF_TIP: Uint = 4;
const MF_STRING: Uint = 0;
const MF_CHECKED: Uint = 8;
const TPM_RIGHTBUTTON: Uint = 2;
const MB_OK: Uint = 0;
const MB_ICONINFORMATION: Uint = 0x40;
const MB_ICONERROR: Uint = 0x10;
const IDI_APPLICATION: *const u16 = 32512_usize as *const u16;
const ID_TOGGLE: usize = 1001;
const ID_SETTINGS: usize = 1002;
const ID_GUIDE: usize = 1003;
const ID_EXIT: usize = 1004;

static ENABLED: AtomicBool = AtomicBool::new(true);

#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
struct Message {
    hwnd: Hwnd,
    message: Uint,
    w_param: Wparam,
    l_param: Lparam,
    time: Uint,
    point: Point,
    private: Uint,
}

type WindowProc = Option<unsafe extern "system" fn(Hwnd, Uint, Wparam, Lparam) -> Lresult>;

#[repr(C)]
struct WndClassW {
    style: Uint,
    wnd_proc: WindowProc,
    cls_extra: i32,
    wnd_extra: i32,
    instance: Hinstance,
    icon: Hicon,
    cursor: *mut c_void,
    background: *mut c_void,
    menu_name: *const u16,
    class_name: *const u16,
}

#[repr(C)]
struct NotifyIconDataW {
    cb_size: Uint,
    hwnd: Hwnd,
    id: Uint,
    flags: Uint,
    callback_message: Uint,
    icon: Hicon,
    tip: [u16; 128],
    state: Uint,
    state_mask: Uint,
    info: [u16; 256],
    timeout_or_version: Uint,
    info_title: [u16; 64],
    info_flags: Uint,
    guid_item: [u8; 16],
    balloon_icon: Hicon,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterClassW(class: *const WndClassW) -> u16;
    fn CreateWindowExW(
        ex_style: Uint,
        class_name: *const u16,
        window_name: *const u16,
        style: Uint,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: Hwnd,
        menu: Hmenu,
        instance: Hinstance,
        param: *mut c_void,
    ) -> Hwnd;
    fn DefWindowProcW(hwnd: Hwnd, message: Uint, w_param: Wparam, l_param: Lparam) -> Lresult;
    fn GetMessageW(message: *mut Message, hwnd: Hwnd, min: Uint, max: Uint) -> Bool;
    fn TranslateMessage(message: *const Message) -> Bool;
    fn DispatchMessageW(message: *const Message) -> Lresult;
    fn PostQuitMessage(exit_code: i32);
    fn LoadIconW(instance: Hinstance, name: *const u16) -> Hicon;
    fn MessageBoxW(hwnd: Hwnd, text: *const u16, caption: *const u16, kind: Uint) -> i32;
    fn CreatePopupMenu() -> Hmenu;
    fn AppendMenuW(menu: Hmenu, flags: Uint, id: usize, text: *const u16) -> Bool;
    fn TrackPopupMenu(
        menu: Hmenu,
        flags: Uint,
        x: i32,
        y: i32,
        reserved: i32,
        hwnd: Hwnd,
        rect: *const c_void,
    ) -> Uint;
    fn DestroyMenu(menu: Hmenu) -> Bool;
    fn GetCursorPos(point: *mut Point) -> Bool;
    fn SetForegroundWindow(hwnd: Hwnd) -> Bool;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(module_name: *const u16) -> Hinstance;
}

#[link(name = "shell32")]
unsafe extern "system" {
    fn Shell_NotifyIconW(message: Uint, data: *mut NotifyIconDataW) -> Bool;
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn settings_path() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("ThinkPadAutoScroll").join("settings.ini")
}

fn load_first_run() -> bool {
    std::fs::read_to_string(settings_path())
        .map(|text| {
            !text
                .lines()
                .any(|line| line.trim() == "onboarding_complete=true")
        })
        .unwrap_or(true)
}

fn save_first_run_complete() -> io::Result<()> {
    let path = settings_path();
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("settings path has no parent"))?;
    std::fs::create_dir_all(parent)?;
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<&str> = existing
        .lines()
        .filter(|line| !line.trim_start().starts_with("onboarding_complete="))
        .collect();
    lines.push("onboarding_complete=true");
    std::fs::write(path, lines.join("\n"))
}

fn save_enabled(enabled: bool) -> io::Result<()> {
    let path = settings_path();
    let parent = path
        .parent()
        .ok_or_else(|| io::Error::other("settings path has no parent"))?;
    std::fs::create_dir_all(parent)?;
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let mut lines: Vec<&str> = existing
        .lines()
        .filter(|line| !line.trim_start().starts_with("enabled="))
        .collect();
    lines.push(if enabled {
        "enabled=true"
    } else {
        "enabled=false"
    });
    std::fs::write(path, lines.join("\n"))
}

fn load_enabled() -> bool {
    std::fs::read_to_string(settings_path())
        .map(|text| !text.lines().any(|line| line.trim() == "enabled=false"))
        .unwrap_or(true)
}

unsafe fn show_guide(hwnd: Hwnd) {
    let text = wide(
        "Auto Scroll is running in the notification area.\n\nWhen a window is partly outside the left or right edge of the screen, move the pointer along that edge toward the visible area to bring the hidden content into view.\n\nUse the Auto Scroll tray icon to pause the feature, open settings, or exit.",
    );
    let title = wide("Welcome to Auto Scroll");
    unsafe {
        MessageBoxW(
            hwnd,
            text.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        )
    };
}

unsafe fn show_settings_placeholder(hwnd: Hwnd) {
    let text = wide(
        "Settings will be available here in a later build.\n\nYou can currently pause or resume Auto Scroll from this tray menu.",
    );
    let title = wide("Auto Scroll Settings");
    unsafe {
        MessageBoxW(
            hwnd,
            text.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        )
    };
}

unsafe fn show_tray_menu(hwnd: Hwnd) {
    let menu = unsafe { CreatePopupMenu() };
    if menu.is_null() {
        return;
    }
    let toggle = wide(if ENABLED.load(Ordering::Relaxed) {
        "Pause Auto Scroll"
    } else {
        "Resume Auto Scroll"
    });
    let settings = wide("Settings...");
    let guide = wide("Show welcome guide");
    let exit = wide("Exit");
    unsafe {
        AppendMenuW(menu, MF_STRING, ID_TOGGLE, toggle.as_ptr());
        AppendMenuW(menu, MF_STRING, ID_SETTINGS, settings.as_ptr());
        AppendMenuW(menu, MF_STRING, ID_GUIDE, guide.as_ptr());
        AppendMenuW(menu, MF_STRING, ID_EXIT, exit.as_ptr());
        let mut point = Point { x: 0, y: 0 };
        GetCursorPos(&mut point);
        SetForegroundWindow(hwnd);
        TrackPopupMenu(menu, TPM_RIGHTBUTTON, point.x, point.y, 0, hwnd, null());
        DestroyMenu(menu);
    }
}

unsafe extern "system" fn window_proc(
    hwnd: Hwnd,
    message: Uint,
    w_param: Wparam,
    l_param: Lparam,
) -> Lresult {
    match message {
        WM_TRAY if l_param as Uint == WM_RBUTTONUP || l_param as Uint == WM_LBUTTONUP => {
            unsafe { show_tray_menu(hwnd) };
            0
        }
        WM_COMMAND => {
            match w_param & 0xffff {
                ID_TOGGLE => {
                    let enabled = !ENABLED.load(Ordering::Relaxed);
                    ENABLED.store(enabled, Ordering::Relaxed);
                    let _ = save_enabled(enabled);
                }
                ID_SETTINGS => unsafe { show_settings_placeholder(hwnd) },
                ID_GUIDE => unsafe { show_guide(hwnd) },
                ID_EXIT => unsafe { PostQuitMessage(0) },
                _ => {}
            }
            0
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, message, w_param, l_param) },
    }
}

fn make_tray_data(hwnd: Hwnd, icon: Hicon) -> NotifyIconDataW {
    let mut data: NotifyIconDataW = unsafe { zeroed() };
    data.cb_size = size_of::<NotifyIconDataW>() as Uint;
    data.hwnd = hwnd;
    data.id = 1;
    data.flags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    data.callback_message = WM_TRAY;
    data.icon = icon;
    let tip = wide("ThinkPad Auto Scroll");
    let n = tip.len().min(data.tip.len());
    data.tip[..n].copy_from_slice(&tip[..n]);
    data
}

fn show_error(message: &str) {
    let text = wide(message);
    let title = wide("Auto Scroll");
    unsafe {
        MessageBoxW(
            null_mut(),
            text.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR,
        )
    };
}

pub fn run() -> io::Result<()> {
    ENABLED.store(load_enabled(), Ordering::Relaxed);
    let first_run = load_first_run();
    let class_name = wide("ThinkPadAutoScrollHiddenHost");
    let instance = unsafe { GetModuleHandleW(null()) };
    let class = WndClassW {
        style: 0,
        wnd_proc: Some(window_proc),
        cls_extra: 0,
        wnd_extra: 0,
        instance,
        icon: null_mut(),
        cursor: null_mut(),
        background: null_mut(),
        menu_name: null(),
        class_name: class_name.as_ptr(),
    };
    if unsafe { RegisterClassW(&class) } == 0 {
        return Err(io::Error::last_os_error());
    }

    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            class_name.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            instance,
            null_mut(),
        )
    };
    if hwnd.is_null() {
        return Err(io::Error::last_os_error());
    }

    let icon = unsafe { LoadIconW(null_mut(), IDI_APPLICATION) };
    let mut tray = make_tray_data(hwnd, icon);
    if unsafe { Shell_NotifyIconW(NIM_ADD, &mut tray) } == 0 {
        return Err(io::Error::last_os_error());
    }

    if first_run {
        unsafe { show_guide(hwnd) };
        if let Err(error) = save_first_run_complete() {
            show_error(&format!("Could not save first-run state: {error}"));
        }
    }

    let mut message: Message = unsafe { zeroed() };
    loop {
        let result = unsafe { GetMessageW(&mut message, null_mut(), 0, 0) };
        if result <= 0 {
            break;
        }
        unsafe {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    unsafe { Shell_NotifyIconW(NIM_DELETE, &mut tray) };
    Ok(())
}
