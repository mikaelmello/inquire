use crate::{
    error::InquireResult,
    formatter::BoolFormatter,
    prompts::{
        confirm::config::ConfirmConfig,
        prompt::{ActionState, Prompt},
    },
    ui::ConfirmBackend,
    Confirm,
};

use super::action::ConfirmPromptAction;

pub struct ConfirmPrompt<'a> {
    message: &'a str,
    help_message: Option<&'a str>,
    default: Option<bool>,
    /// 当前聚焦/选择的布尔值（取代了原先的 input 字符缓冲区）
    current_value: bool,
    formatter: BoolFormatter<'a>,
    default_value_formatter: BoolFormatter<'a>,
}

impl<'a> From<Confirm<'a>> for ConfirmPrompt<'a> {
    fn from(co: Confirm<'a>) -> Self {
        // 如果用户设置了默认值，初始焦点就停在默认值上；否则默认停在 true (Y) 上
        let current_value = co.default.unwrap_or(true);

        Self {
            message: co.message,
            default: co.default,
            current_value,
            help_message: co.help_message,
            formatter: co.formatter,
            default_value_formatter: co.default_value_formatter,
        }
    }
}

impl<'a, Backend> Prompt<Backend> for ConfirmPrompt<'a>
where
    Backend: ConfirmBackend,
{
    type Config = ConfirmConfig;
    type InnerAction = ConfirmPromptAction;
    type Output = bool;

    fn message(&self) -> &str {
        self.message
    }

    fn config(&self) -> &Self::Config {
        &ConfirmConfig {}
    }

    fn format_answer(&self, answer: &bool) -> String {
        (self.formatter)(*answer)
    }

    fn submit(&mut self) -> InquireResult<Option<bool>> {
        // 因为 Confirm 天然合法，不需要 validate_current_answer 这一步
        // 只要触发了提交，就直接把当前选中的布尔值作为最终答案送出
        Ok(Some(self.current_value))
    }

    fn handle(&mut self, action: ConfirmPromptAction) -> InquireResult<ActionState> {
        let result = match action {
            // 1. y/n 键立即提交对应的布尔值
            ConfirmPromptAction::DirectSubmit(val) => {
                self.current_value = val;
                ActionState::RequestSubmit
            }
            // 2. 按回车直接提交。如果有默认值，以默认值为准
            ConfirmPromptAction::SubmitDefault => {
                if let Some(default_val) = self.default {
                    self.current_value = default_val;
                }
                ActionState::RequestSubmit
            }
            // 4. 退出
            ConfirmPromptAction::PressEscape => ActionState::RequestCancel,
        };

        Ok(result.into())
    }

    fn render(&self, backend: &mut Backend) -> InquireResult<()> {
        let prompt = &self.message;

        // 格式化括号内默认值的提示，如 "Y/n"
        let default_value_formatter = self.default_value_formatter;
        let default_message = self
            .default
            .as_ref()
            .map(|val| default_value_formatter(*val));

        // 调用 ConfirmBackend 对应的绘制方法
        backend.render_prompt(prompt, default_message.as_deref(), self.current_value)?;

        if let Some(message) = self.help_message {
            backend.render_help_message(message)?;
        }

        Ok(())
    }
}
