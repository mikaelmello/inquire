//! Global config definitions.

use std::sync::Mutex;

use lazy_static::lazy_static;

use crate::ui::RenderConfig;

lazy_static! {
    static ref INQUIRE_CONFIGURATION: Mutex<InquireConfiguration> =
        Mutex::new(InquireConfiguration::default());
}

pub(in crate) fn get_configuration() -> InquireConfiguration {
    INQUIRE_CONFIGURATION.lock().unwrap().clone()
}

/// Acquires a write lock to the global InquireConfiguration object
/// and updates the inner value with the provided argument.
pub fn set_configuration(config: InquireConfiguration) {
    let mut guard = INQUIRE_CONFIGURATION.lock().unwrap();
    *guard = config;
}

/// Struct containing all inquire-relevant configuration to be stored
/// globally on a binary's context.
#[derive(Copy, Clone, Debug)]
pub struct InquireConfiguration {
    /// Settings specific to render operations, such as
    /// which colors or prefixes to use when rendering a
    /// prompt.
    pub render_config: RenderConfig,

    /// Page size in prompts which may display a list
    /// of options: [`Text`], [`Select`] and [`MultiSelect`]
    ///
    /// [`Text`]: crate::Text
    /// [`Select`]: crate::Select
    /// [`MultiSelect`]: crate::MultiSelect
    pub page_size: usize,
}

impl Default for InquireConfiguration {
    fn default() -> Self {
        Self {
            render_config: RenderConfig::default(),
            page_size: 7,
        }
    }
}
