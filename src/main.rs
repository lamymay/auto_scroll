#![cfg_attr(windows, windows_subsystem = "windows")]
#![cfg_attr(not(windows), allow(dead_code))]

mod settings;

#[cfg(windows)]
mod win;

#[cfg(windows)]
fn main() {
    if let Err(error) = win::run() {
        eprintln!("Auto Scroll failed: {error}");
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("ThinkPad Auto Scroll is a Windows-only application.");
}
