# ThinkPad Auto Scroll (Windows)

Windows implementation of the ThinkPad Auto Scroll behavior: when the active window extends beyond the left or right edge of a monitor, move the pointer along that edge toward the visible area to pan the window and reveal the clipped content.

## Current status

The Windows app now starts as a notification-area tray utility using native Win32 APIs, with no external GUI framework or runtime dependency. On first launch it shows a short usage guide and remembers completion under `%APPDATA%\\ThinkPadAutoScroll\\settings.ini`. The tray menu provides pause/resume, show guide, settings entry, and exit. The actual edge-drag detection and window panning behavior, plus a real settings window, are still pending.

Build a Windows x64 release from PowerShell with:

```powershell
.\scripts\build-windows.ps1
```

For Windows ARM64, run `.\scripts\build-windows.ps1 -TargetCpu arm64`. The script writes the executable to `dist`.

This repository is Windows-only for now. macOS will be a separate implementation.

The tray host is created with direct Win32 calls, so the Windows executable uses the GUI subsystem and does not open a console window.

## Settings behavior

- Theme options: Follow system, Light, Dark.
- Open Settings: `Ctrl + ,` on Windows; the separate macOS app should use `Command + ,`.
