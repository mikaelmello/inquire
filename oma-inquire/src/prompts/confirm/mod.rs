mod action;
mod prompt;
mod config;
#[cfg(test)]
mod test; // 存放你接下来要实现的全新 ConfirmPrompt

pub use action::*;

use crate::{
    config::get_configuration, error::{InquireError, InquireResult}, formatter::{BoolFormatter, DEFAULT_BOOL_FORMATTER}, prompts::prompt::Prompt, terminal::get_default_terminal, ui::{Backend, ConfirmBackend, RenderConfig},
};

use self::prompt::ConfirmPrompt;

/// y/n widget
#[derive(Clone)]
pub struct Confirm<'a> {
    /// 提示用户的信息。
    pub message: &'a str,

    /// 默认值。当用户直接敲击回车时以此值为准。
    pub default: Option<bool>,

    /// 提示帮助信息，展示在最下方。
    pub help_message: Option<&'a str>,

    /// 格式化最终展示结果的函数。
    /// 默认将 `true` 格式化为 "Yes"，`false` 格式化为 "No"。
    pub formatter: BoolFormatter<'a>,

    /// 格式化默认值文本展示的函数。
    /// 默认将 `true` 格式化为 "Y/n"，`false` 格式化为 "y/N"。
    pub default_value_formatter: BoolFormatter<'a>,

    /// 终端渲染的主题样式配置。
    pub render_config: RenderConfig<'a>,
}

impl<'a> Confirm<'a> {
    /// 默认输出格式化器，默认 true 为 "Yes", false 为 "No"。
    pub const DEFAULT_FORMATTER: BoolFormatter<'a> = DEFAULT_BOOL_FORMATTER;

    /// 默认未敲定答案时，提示文本的格式化器。
    pub const DEFAULT_DEFAULT_VALUE_FORMATTER: BoolFormatter<'a> = &|ans| match ans {
        true => String::from("Y/n"),
        false => String::from("y/N"),
    };

    /// 创建一个全新的、带默认配置的 [Confirm] 实例。
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            default: None,
            help_message: None,
            formatter: Self::DEFAULT_FORMATTER,
            default_value_formatter: Self::DEFAULT_DEFAULT_VALUE_FORMATTER,
            render_config: get_configuration(),
        }
    }

    /// 设置默认的确认选项。
    pub fn with_default(mut self, default: bool) -> Self {
        self.default = Some(default);
        self
    }

    /// 设置下方的帮助文本信息。
    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
        self
    }

    /// 清空帮助文本信息。
    pub fn without_help_message(mut self) -> Self {
        self.help_message = None;
        self
    }

    /// 覆写最终结果的格式化器。
    pub fn with_formatter(mut self, formatter: BoolFormatter<'a>) -> Self {
        self.formatter = formatter;
        self
    }

    /// 覆写未敲定默认值前的展示格式化器。
    pub fn with_default_value_formatter(mut self, formatter: BoolFormatter<'a>) -> Self {
        self.default_value_formatter = formatter;
        self
    }

    /// 设置当前组件使用的终端样式色彩主题。
    pub fn with_render_config(mut self, render_config: RenderConfig<'a>) -> Self {
        self.render_config = render_config;
        self
    }

    /// 启动询问。如果用户按了 Esc 退出，会被视为平稳退出（不报错，而是返回 Ok(None)）。
    pub fn prompt_skippable(self) -> InquireResult<Option<bool>> {
        match self.prompt() {
            Ok(answer) => Ok(Some(answer)),
            Err(InquireError::OperationCanceled) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// 启动询问。获取用户的即时单键选择，或通过回车选择默认值。
    pub fn prompt(self) -> InquireResult<bool> {
        let (input_reader, terminal) = get_default_terminal()?;
        let mut backend = Backend::new(input_reader, terminal, self.render_config)?;
        self.prompt_with_backend(&mut backend)
    }

    /// 🟢 核心重构：使用全新的 ConfirmPrompt 组件并由 SelectBackend（或你具体的 Backend 类型）渲染
    pub(crate) fn prompt_with_backend<B: ConfirmBackend>(
        self,
        backend: &mut B,
    ) -> InquireResult<bool> {
        ConfirmPrompt::from(self).prompt(backend)
    }
}

impl<'a> From<&'a str> for Confirm<'a> {
    fn from(val: &'a str) -> Self {
        Confirm::new(val)
    }
}
