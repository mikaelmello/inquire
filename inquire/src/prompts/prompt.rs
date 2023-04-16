use crate::{
    error::InquireResult,
    ui::{CommonBackend, InnerPromptAction, PromptAction},
    validator::ErrorMessage,
    InquireError,
};

pub struct Prompt<'a, Fmt, Vld> {
    message: &'a str,
    help_message: Option<&'a str>,
    formatter: Fmt,
    validators: Vec<Vld>,
    error: Option<ErrorMessage>,
}

pub trait PromptTrait<Backend, Config, Action, ReturnType>
where
    Backend: CommonBackend,
    Action: InnerPromptAction<Config>,
    Self: Sized,
{
    fn message(&self) -> &str;
    fn config(&self) -> &Config;
    fn format_answer(&self, answer: &ReturnType) -> String;

    fn setup(&mut self) -> InquireResult<()>;
    fn submit(&mut self) -> InquireResult<Option<ReturnType>>;
    fn handle(&mut self, action: Action) -> InquireResult<()>;
    fn render(&self, backend: &mut Backend) -> InquireResult<()>;

    fn prompt(mut self, backend: &mut Backend) -> InquireResult<ReturnType> {
        self.setup()?;

        let final_answer = loop {
            backend.frame_setup()?;
            self.render(backend)?;
            backend.frame_finish()?;

            let key = backend.read_key()?;
            let action = PromptAction::<Action>::from_key(key, self.config());

            match action {
                PromptAction::Inner(action) => self.handle(action)?,
                PromptAction::Submit => match self.submit()? {
                    Some(answer) => break answer,
                    None => (),
                },
                PromptAction::Cancel => {
                    backend.frame_setup()?;
                    backend.render_canceled_prompt(self.message())?;
                    backend.frame_finish()?;
                    return Err(InquireError::OperationCanceled);
                }
                PromptAction::Interrupt => return Err(InquireError::OperationInterrupted),
                PromptAction::None => (),
            }
        };

        let formatted = self.format_answer(&final_answer);

        backend.frame_setup()?;
        backend.render_prompt_with_answer(self.message(), &formatted)?;
        backend.frame_finish()?;

        return Ok(final_answer);
    }
}
