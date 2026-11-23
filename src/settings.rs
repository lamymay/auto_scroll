//! Shared user preferences for the desktop application.
//!
//! The Windows UI should expose `Theme` as a selector in Settings and persist
//! the chosen value. On macOS the same preference can be reused by its own UI.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

impl Theme {
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];

    pub const fn label(self) -> &'static str {
        match self {
            Self::System => "Follow system",
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Settings {
    pub theme: Theme,
}

/// Platform convention for opening Settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsShortcut {
    CtrlComma,
    CommandComma,
}

#[cfg(target_os = "macos")]
pub const SETTINGS_SHORTCUT: SettingsShortcut = SettingsShortcut::CommandComma;

#[cfg(not(target_os = "macos"))]
pub const SETTINGS_SHORTCUT: SettingsShortcut = SettingsShortcut::CtrlComma;

impl SettingsShortcut {
    pub const fn label(self) -> &'static str {
        match self {
            Self::CtrlComma => "Ctrl + ,",
            Self::CommandComma => "Command + ,",
        }
    }
}
