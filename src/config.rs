//! Global config definitions.

use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::ui::RenderConfig;

lazy_static! {
    static ref GLOBAL_RENDER_CONFIGURATION: Mutex<RenderConfig> =
        Mutex::new(RenderConfig::default());
}

pub fn get_configuration() -> RenderConfig {
    *GLOBAL_RENDER_CONFIGURATION.lock().unwrap()
}

/// Acquires a write lock to the global RenderConfig object
/// and updates the inner value with the provided argument.
pub fn set_global_render_config(config: RenderConfig) {
    let mut guard = GLOBAL_RENDER_CONFIGURATION.lock().unwrap();
    *guard = config;
}

/// Default page size when displaying options to the user.
pub const DEFAULT_PAGE_SIZE: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq)]
/// Specifies the keymap mode to use when navigating prompts.
pub enum KeymapMode {
    /// No keybindings are enabled
    None,
    /// Vim keybindings (hjkl) are enabled
    Vim,
    /// Emacs keybindings (C-p, C-n, C-f, C-b) are enabled
    Emacs,
}

/// Default value of keymap mode.
pub const DEFAULT_KEYMAP_MODE: KeymapMode = KeymapMode::None;
