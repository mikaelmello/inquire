use crate::{
    input::InputAction,
    ui::{InnerAction, Key},
};

use super::config::CustomTypeConfig;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum CustomTypePromptAction {
    ValueInput(InputAction),
}

impl InnerAction<CustomTypeConfig> for CustomTypePromptAction {
    fn from_key(key: Key, _config: &CustomTypeConfig) -> Option<Self> {
        let action = match key {
            key => match InputAction::from_key(key, &()) {
                Some(action) => Self::ValueInput(action),
                None => return None,
            },
        };

        Some(action)
    }
}
