//! Global config definitions.

use std::sync::Mutex;

use once_cell::sync::Lazy;

use crate::ui::RenderConfig;

static GLOBAL_RENDER_CONFIGURATION: Lazy<Mutex<RenderConfig<'static>>> =
    Lazy::new(|| Mutex::new(RenderConfig::default()));

pub fn get_configuration() -> RenderConfig<'static> {
    *GLOBAL_RENDER_CONFIGURATION.lock().unwrap()
}

/// Acquires a write lock to the global RenderConfig object
/// and updates the inner value with the provided argument.
pub fn set_global_render_config(config: RenderConfig<'static>) {
    let mut guard = GLOBAL_RENDER_CONFIGURATION.lock().unwrap();
    *guard = config;
}

/// Default page size when displaying options to the user.
pub const DEFAULT_PAGE_SIZE: usize = 7;

/// Default value of vim mode.
pub const DEFAULT_VIM_MODE: bool = false;
