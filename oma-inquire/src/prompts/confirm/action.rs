use crate::{
    prompts::confirm::config::ConfirmConfig,
    ui::{Key, KeyModifiers},
    EscapePolicy, InnerAction,
};

/// Confim 组件
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConfirmPromptAction {
    /// 用户直接按了 Y/y 或 N/n，会立即提交
    DirectSubmit(bool),
    /// 用户按了回车（Enter），决定使用当前的默认值提交。
    SubmitDefault,
    /// 动态接管 Escape 键。
    /// 允许用户在有特殊交互时放下，或直接触发退出（Cancel）。
    PressEscape,
}

impl<'a> InnerAction for ConfirmPromptAction {
    type Config = ConfirmConfig;

    const ESCAPE_POLICY: EscapePolicy = EscapePolicy::IgnoreEscape;

    fn from_key(key: Key, _config: &Self::Config) -> Option<Self> {
        match key {
            Key::Char('y', KeyModifiers::NONE) | Key::Char('Y', KeyModifiers::NONE) => {
                Some(Self::DirectSubmit(true))
            }
            Key::Char('n', KeyModifiers::NONE) | Key::Char('N', KeyModifiers::NONE) => {
                Some(Self::DirectSubmit(false))
            }
            Key::Enter
            | Key::Char('\n', KeyModifiers::NONE)
            | Key::Char('j', KeyModifiers::CONTROL) => Some(Self::SubmitDefault),
            Key::Escape => Some(Self::PressEscape),
            _ => None,
        }
    }
}
