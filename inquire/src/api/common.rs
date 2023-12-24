use crate::{formatter::SubmissionFormatter, ui::RenderConfig, validator::SubmissionValidator};

#[derive(Clone)]
pub struct CommonConfig<'a, OutputType, OutputAsArgumentType> {
    /// Message to be presented to the user.
    pub message: String,

    /// Help message to be presented to the user.
    pub help_message: Option<String>,

    /// Default value to be used.
    pub default: Option<OutputType>,

    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: Box<dyn SubmissionFormatter<OutputAsArgumentType>>,

    /// Collection of validators to apply to the user input.
    ///
    /// Validators are executed in the order they are stored, stopping at and displaying to the user
    /// only the first validation error that might appear.
    ///
    /// The possible error is displayed to the user one line above the prompt.
    pub validators: Vec<Box<dyn SubmissionValidator<OutputAsArgumentType>>>,

    /// RenderConfig to apply to the rendered interface.
    ///
    /// Note: The default render config considers if the NO_COLOR environment variable
    /// is set to decide whether to render the colored config or the empty one.
    ///
    /// When overriding the config in a prompt, NO_COLOR is no longer considered and your
    /// config is treated as the only source of truth. If you want to customize colors
    /// and still support NO_COLOR, you will have to do this on your end.
    pub render_config: RenderConfig<'a>,
}

macro_rules! common_config_builder_methods {
    // macro with generic type as arg
    ($output_type:ty, $output_as_arg_type:ty) => {
        /// Sets the message of the prompt.
        /// The message is presented to the user as the first line of the prompt.
        pub fn with_message(mut self, message: impl Into<String>) -> Self {
            self.common.message = message.into();
            self
        }

        /// Sets the help message of the prompt.
        /// The help message is presented to the user as the last line of the prompt.
        /// It is used to provide additional information about the prompt.
        ///
        /// If you want to provide a help message, but only in certain cases, you can use [`with_opt_help_message`].
        pub fn with_help_message(mut self, help_message: impl Into<String>) -> Self {
            self.common.help_message = Some(help_message.into());
            self
        }

        /// Sets the help message of the prompt.
        /// The help message is presented to the user as the last line of the prompt.
        /// It is used to provide additional information about the prompt.
        ///
        /// If you want a non-optional alternative to this method, you can use [`with_help_message`].
        pub fn with_opt_help_message(mut self, help_message: Option<impl Into<String>>) -> Self {
            self.common.help_message = help_message.map(Into::into);
            self
        }

        pub fn with_default(mut self, default: $output_type) -> Self {
            self.common.default = Some(default);
            self
        }

        /// Sets the formatter of the prompt.
        /// The formatter is responsible for formatting the user input and presenting it to the user as the final rendering of the prompt.
        /// The default formatter is the [`DefaultFormatter`].
        ///
        /// # Example
        /// ```rust
        /// use inquire::{Text, formatter::DefaultFormatter};
        /// let formatter = DefaultFormatter::new();
        /// let prompt = Text::new("What's your name?")
        ///    .with_formatter(formatter);
        /// ```
        ///
        /// [`Text`]: crate::Text
        /// [`DefaultFormatter`]: crate::formatter::DefaultFormatter
        #[allow(unused_qualifications)]
        pub fn with_formatter(
            mut self,
            formatter: impl crate::formatter::SubmissionFormatter<$output_as_arg_type> + 'static,
        ) -> Self {
            self.common.formatter = Box::new(formatter);
            self
        }

        /// Adds a validator to the prompt.
        /// Validators are executed in the order they are stored, stopping at and displaying to the user
        /// only the first validation error that might appear.
        ///
        /// The possible error is displayed to the user one line above the prompt.
        ///
        /// # Example
        /// ```rust
        /// use inquire::{Text, validator::Validation};
        /// let validator = |input: &str| if input.chars().count() > 140 {
        ///    Ok(Validation::Invalid("You're only allowed 140 characters.".into()))
        /// } else {
        ///   Ok(Validation::Valid)
        /// };
        /// let prompt = Text::new("What are you thinking about?")
        ///   .with_validator(validator);
        /// ```
        ///
        /// [`Text`]: crate::Text
        pub fn with_validator(
            mut self,
            validator: impl crate::validator::SubmissionValidator<$output_as_arg_type> + 'static,
        ) -> Self {
            self.common.validators.push(Box::new(validator));
            self
        }

        /// Sets new render settings for the prompt.
        ///
        /// The default render config considers if the NO_COLOR environment variable
        /// is set to decide whether to render the colored config or the empty one.
        ///
        /// When overriding the config in a prompt, NO_COLOR is no longer considered and your
        /// config is treated as the only source of truth. If you want to customize colors
        /// and still support NO_COLOR, you will have to do this on your end.
        #[allow(clippy::large_types_passed_by_value)]
        pub fn with_render_config(mut self, render_config: crate::ui::RenderConfig<'a>) -> Self {
            self.common.render_config = render_config;
            self
        }

        /// Parses the provided behavioral and rendering options and prompts
        /// the CLI user for input according to the defined rules.
        ///
        /// This method is intended for flows where the user skipping/cancelling
        /// the prompt - by pressing ESC - is considered normal behavior. In this case,
        /// it does not return `Err(InquireError::OperationCanceled)`, but `Ok(None)`.
        ///
        /// Meanwhile, if the user does submit an answer, the method wraps the return
        /// type with `Some`.
        #[allow(unused_qualifications)]
        pub fn prompt_skippable(self) -> crate::error::InquireResult<Option<$output_type>> {
            match self.prompt() {
                Ok(answer) => Ok(Some(answer)),
                Err(crate::error::InquireError::OperationCanceled) => Ok(None),
                Err(err) => Err(err),
            }
        }

        /// Parses the provided behavioral and rendering options and prompts
        /// the CLI user for input according to the defined rules.
        #[allow(unused_qualifications)]
        pub fn prompt(self) -> crate::error::InquireResult<$output_type> {
            let terminal = crate::terminal::get_default_terminal()?;
            let mut backend = crate::ui::Backend::new(terminal, self.common.render_config)?;
            self.prompt_with_backend(&mut backend)
        }

        #[allow(unused_qualifications)]
        pub(crate) fn prompt_with_backend<T: crate::terminal::Terminal>(
            self,
            backend: &mut crate::ui::Backend<'a, T>,
        ) -> crate::error::InquireResult<$output_type> {
            let (common_config, inner_impl) = self.inner_impl()?;
            let prompt = crate::new_prompts::base::Prompt::new(
                common_config.message,
                common_config.help_message,
                common_config.validators,
                common_config.formatter,
                backend,
                inner_impl,
            );

            prompt.prompt()
        }
    };
}
