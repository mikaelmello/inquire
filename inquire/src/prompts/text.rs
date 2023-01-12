use std::cmp::min;

use crate::{
    autocompletion::{Autocomplete, NoAutoCompletion, Replacement},
    config::{self, get_configuration},
    error::{InquireError, InquireResult},
    formatter::{StringFormatter, DEFAULT_STRING_FORMATTER},
    input::Input,
    list_option::ListOption,
    terminal::get_default_terminal,
    ui::{Backend, Key, KeyModifiers, RenderConfig, TextBackend},
    utils::paginate,
    validator::{ErrorMessage, StringValidator, Validation},
};

const DEFAULT_HELP_MESSAGE_WITH_AC: &str = "↑↓ to move, tab to autocomplete, enter to submit";

/// Standard text prompt that returns the user string input.
///
/// This is the standard the standard kind of prompt you would expect from a library like this one. It displays a message to the user, prompting them to type something back. The user's input is then stored in a `String` and returned to the prompt caller.
///
///
/// ## Configuration options
///
/// - **Prompt message**: Main message when prompting the user for input, `"What is your name?"` in the example below.
/// - **Help message**: Message displayed at the line below the prompt.
/// - **Default value**: Default value returned when the user submits an empty response.
/// - **Initial value**: Initial value of the prompt's text input, in case you want to display the prompt with something already filled in.
/// - **Placeholder**: Short hint that describes the expected value of the input.
/// - **Validators**: Custom validators to the user's input, displaying an error message if the input does not pass the requirements.
/// - **Formatter**: Custom formatter in case you need to pre-process the user input before showing it as the final answer.
/// - **Suggester**: Custom function that returns a list of input suggestions based on the current text input. See more on "Autocomplete" below.
///
/// ## Default behaviors
///
/// Default behaviors for each one of `Text` configuration options:
///
/// - The input formatter just echoes back the given input.
/// - No validators are called, accepting any sort of input including empty ones.
/// - No default values or help messages.
/// - No autocompletion features set-up.
/// - Prompt messages are always required when instantiating via `new()`.
///
/// ## Autocomplete
///
/// With `Text` inputs, it is also possible to set-up an autocompletion system to provide a better UX when necessary.
///
/// You can call `with_autocomplete()` and provide a value that implements the `Autocomplete` trait. The `Autocomplete` trait has two provided methods: `get_suggestions` and `get_completion`.
///
/// - `get_suggestions` is called whenever the user's text input is modified, e.g. a new letter is typed, returning a `Vec<String>`. The `Vec<String>` is the list of suggestions that the prompt displays to the user according to their text input. The user can then navigate through the list and if they submit while highlighting one of these suggestions, the suggestion is treated as the final answer.
/// - `get_completion` is called whenever the user presses the autocompletion hotkey (`tab` by default), with the current text input and the text of the currently highlighted suggestion, if any, as parameters. This method should return whether any text replacement (an autocompletion) should be made. If the prompt receives a replacement to be made, it substitutes the current text input for the string received from the `get_completion` call.
///
/// For example, in the `complex_autocompletion.rs` example file, the `FilePathCompleter` scans the file system based on the current text input, storing a list of paths that match the current text input.
///
/// Everytime `get_suggestions` is called, the method returns the list of paths that match the user input. When the user presses the autocompletion hotkey, the `FilePathCompleter` checks whether there is any path selected from the list, if there is, it decides to replace the current text input for it. The interesting piece of functionality is that if there isn't a path selected from the list, the `FilePathCompleter` calculates the longest common prefix amongst all scanned paths and updates the text input to an unambiguous new value. Similar to how terminals work when traversing paths.
///
/// # Example
///
/// ```no_run
/// use inquire::Text;
///
/// let name = Text::new("What is your name?").prompt();
///
/// match name {
///     Ok(name) => println!("Hello {}", name),
///     Err(_) => println!("An error happened when asking for your name, try again later."),
/// }
/// ```
#[derive(Clone)]
pub struct Text<'a> {
    /// Message to be presented to the user.
    pub message: &'a str,

    /// Initial value of the prompt's text input.
    ///
    /// If you want to set a default value for the prompt, returned when the user's submission is empty, see [`default`].
    ///
    /// [`default`]: Self::default
    pub initial_value: Option<&'a str>,

    /// Default value, returned when the user input is empty.
    pub default: Option<&'a str>,

    /// Short hint that describes the expected value of the input.
    pub placeholder: Option<&'a str>,

    /// Help message to be presented to the user.
    pub help_message: Option<&'a str>,

    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: StringFormatter<'a>,

    /// Autocompleter responsible for handling suggestions and input completions.
    pub autocompleter: Option<Box<dyn Autocomplete>>,

    /// Collection of validators to apply to the user input.
    ///
    /// Validators are executed in the order they are stored, stopping at and displaying to the user
    /// only the first validation error that might appear.
    ///
    /// The possible error is displayed to the user one line above the prompt.
    pub validators: Vec<Box<dyn StringValidator>>,

    /// Page size of the suggestions displayed to the user, when applicable.
    pub page_size: usize,

    /// RenderConfig to apply to the rendered interface.
    ///
    /// Note: The default render config considers if the NO_COLOR environment variable
    /// is set to decide whether to render the colored config or the empty one.
    ///
    /// When overriding the config in a prompt, NO_COLOR is no longer considered and your
    /// config is treated as the only source of truth. If you want to customize colors
    /// and still suport NO_COLOR, you will have to do this on your end.
    pub render_config: RenderConfig,
}

impl<'a> Text<'a> {
    /// Default formatter, set to [DEFAULT_STRING_FORMATTER](crate::formatter::DEFAULT_STRING_FORMATTER)
    pub const DEFAULT_FORMATTER: StringFormatter<'a> = DEFAULT_STRING_FORMATTER;

    /// Default page size, equal to the global default page size [config::DEFAULT_PAGE_SIZE]
    pub const DEFAULT_PAGE_SIZE: usize = config::DEFAULT_PAGE_SIZE;

    /// Default validators added to the [Text] prompt, none.
    pub const DEFAULT_VALIDATORS: Vec<Box<dyn StringValidator>> = vec![];

    /// Default help message.
    pub const DEFAULT_HELP_MESSAGE: Option<&'a str> = None;

    /// Creates a [Text] with the provided message and default options.
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            placeholder: None,
            initial_value: None,
            default: None,
            help_message: Self::DEFAULT_HELP_MESSAGE,
            validators: Self::DEFAULT_VALIDATORS,
            formatter: Self::DEFAULT_FORMATTER,
            page_size: Self::DEFAULT_PAGE_SIZE,
            autocompleter: None,
            render_config: get_configuration(),
        }
    }

    /// Sets the help message of the prompt.
    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
        self
    }

    /// Sets the initial value of the prompt's text input.
    ///
    /// If you want to set a default value for the prompt, returned when the user's submission is empty, see [`with_default`].
    ///
    /// [`with_default`]: Self::with_default
    pub fn with_initial_value(mut self, message: &'a str) -> Self {
        self.initial_value = Some(message);
        self
    }

    /// Sets the default input.
    pub fn with_default(mut self, message: &'a str) -> Self {
        self.default = Some(message);
        self
    }

    /// Sets the placeholder.
    pub fn with_placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    /// Sets a new autocompleter
    pub fn with_autocomplete<AC>(mut self, ac: AC) -> Self
    where
        AC: Autocomplete + 'static,
    {
        self.autocompleter = Some(Box::new(ac));
        self
    }

    /// Sets the formatter.
    pub fn with_formatter(mut self, formatter: StringFormatter<'a>) -> Self {
        self.formatter = formatter;
        self
    }

    /// Sets the page size
    pub fn with_page_size(mut self, page_size: usize) -> Self {
        self.page_size = page_size;
        self
    }

    /// Adds a validator to the collection of validators. You might want to use this feature
    /// in case you need to require certain features from the user's answer, such as
    /// defining a limit of characters.
    ///
    /// Validators are executed in the order they are stored, stopping at and displaying to the user
    /// only the first validation error that might appear.
    ///
    /// The possible error is displayed to the user one line above the prompt.
    pub fn with_validator<V>(mut self, validator: V) -> Self
    where
        V: StringValidator + 'static,
    {
        // Directly make space for at least 5 elements, so we won't to re-allocate too often when
        // calling this function repeatedly.
        if self.validators.capacity() == 0 {
            self.validators.reserve(5);
        }

        self.validators.push(Box::new(validator));
        self
    }

    /// Adds the validators to the collection of validators in the order they are given.
    /// You might want to use this feature in case you need to require certain features
    /// from the user's answer, such as defining a limit of characters.
    ///
    /// Validators are executed in the order they are stored, stopping at and displaying to the user
    /// only the first validation error that might appear.
    ///
    /// The possible error is displayed to the user one line above the prompt.
    pub fn with_validators(mut self, validators: &[Box<dyn StringValidator>]) -> Self {
        for validator in validators {
            #[allow(clippy::clone_double_ref)]
            self.validators.push(validator.clone());
        }
        self
    }

    /// Sets the provided color theme to this prompt.
    ///
    /// Note: The default render config considers if the NO_COLOR environment variable
    /// is set to decide whether to render the colored config or the empty one.
    ///
    /// When overriding the config in a prompt, NO_COLOR is no longer considered and your
    /// config is treated as the only source of truth. If you want to customize colors
    /// and still suport NO_COLOR, you will have to do this on your end.
    pub fn with_render_config(mut self, render_config: RenderConfig) -> Self {
        self.render_config = render_config;
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
    pub fn prompt_skippable(self) -> InquireResult<Option<String>> {
        match self.prompt() {
            Ok(answer) => Ok(Some(answer)),
            Err(InquireError::OperationCanceled) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to the defined rules.
    pub fn prompt(self) -> InquireResult<String> {
        let terminal = get_default_terminal()?;
        let mut backend = Backend::new(terminal, self.render_config)?;
        self.prompt_with_backend(&mut backend)
    }

    pub(crate) fn prompt_with_backend<B: TextBackend>(
        self,
        backend: &mut B,
    ) -> InquireResult<String> {
        TextPrompt::from(self).prompt(backend)
    }
}

struct TextPrompt<'a> {
    message: &'a str,
    default: Option<&'a str>,
    help_message: Option<&'a str>,
    input: Input,
    formatter: StringFormatter<'a>,
    validators: Vec<Box<dyn StringValidator>>,
    error: Option<ErrorMessage>,
    autocompleter: Box<dyn Autocomplete>,
    suggested_options: Vec<String>,
    cursor_index: usize,
    page_size: usize,
}

impl<'a> From<Text<'a>> for TextPrompt<'a> {
    fn from(so: Text<'a>) -> Self {
        let input = Input::new_with(so.initial_value.unwrap_or_default());
        let input = if let Some(placeholder) = so.placeholder {
            input.with_placeholder(placeholder)
        } else {
            input
        };

        Self {
            message: so.message,
            default: so.default,
            help_message: so.help_message,
            formatter: so.formatter,
            autocompleter: so
                .autocompleter
                .unwrap_or_else(|| Box::new(NoAutoCompletion::default())),
            input,
            error: None,
            cursor_index: 0,
            page_size: so.page_size,
            suggested_options: vec![],
            validators: so.validators,
        }
    }
}

impl<'a> From<&'a str> for Text<'a> {
    fn from(val: &'a str) -> Self {
        Text::new(val)
    }
}

impl<'a> TextPrompt<'a> {
    fn update_suggestions(&mut self) -> InquireResult<()> {
        self.suggested_options = self.autocompleter.get_suggestions(self.input.content())?;
        self.cursor_index = 0;

        Ok(())
    }

    fn get_highlighted_suggestion(&self) -> Option<&str> {
        if self.cursor_index > 0 {
            let index = self.cursor_index - 1;
            let suggestion = self.suggested_options.get(index).unwrap().as_ref();
            Some(suggestion)
        } else {
            None
        }
    }

    fn move_cursor_up(&mut self, qty: usize) -> bool {
        self.cursor_index = self.cursor_index.saturating_sub(qty);

        false
    }

    fn move_cursor_down(&mut self, qty: usize) -> bool {
        self.cursor_index = min(
            self.cursor_index.saturating_add(qty),
            self.suggested_options.len(),
        );

        false
    }

    fn handle_tab_key(&mut self) -> InquireResult<bool> {
        let suggestion = self.get_highlighted_suggestion().map(|s| s.to_owned());
        match self
            .autocompleter
            .get_completion(self.input.content(), suggestion)?
        {
            Replacement::Some(value) => {
                self.input = Input::new_with(&value);
                Ok(true)
            }
            Replacement::None => Ok(false),
        }
    }

    fn on_change(&mut self, key: Key) -> InquireResult<()> {
        let dirty = match key {
            Key::Up(KeyModifiers::NONE) => self.move_cursor_up(1),
            Key::PageUp => self.move_cursor_up(self.page_size),

            Key::Down(KeyModifiers::NONE) => self.move_cursor_down(1),
            Key::PageDown => self.move_cursor_down(self.page_size),

            Key::Tab => self.handle_tab_key()?,

            key => self.input.handle_key(key),
        };

        if dirty {
            self.update_suggestions()?;
        }

        Ok(())
    }

    fn get_current_answer(&self) -> &str {
        // If there is a highlighted suggestion, assume user wanted it as
        // the answer.
        if let Some(suggestion) = self.get_highlighted_suggestion() {
            return suggestion;
        }

        // Empty input with default values override any validators.
        if self.input.content().is_empty() {
            if let Some(val) = self.default {
                return val;
            }
        }

        self.input.content()
    }

    fn validate_current_answer(&self) -> InquireResult<Validation> {
        for validator in &self.validators {
            match validator.validate(self.get_current_answer()) {
                Ok(Validation::Valid) => {}
                Ok(Validation::Invalid(msg)) => return Ok(Validation::Invalid(msg)),
                Err(err) => return Err(InquireError::Custom(err)),
            }
        }

        Ok(Validation::Valid)
    }

    fn render<B: TextBackend>(&mut self, backend: &mut B) -> InquireResult<()> {
        let prompt = &self.message;

        backend.frame_setup()?;

        if let Some(err) = &self.error {
            backend.render_error_message(err)?;
        }

        backend.render_prompt(prompt, self.default, &self.input)?;

        let choices = self
            .suggested_options
            .iter()
            .enumerate()
            .map(|(i, val)| ListOption::new(i, val.as_ref()))
            .collect::<Vec<ListOption<&str>>>();

        let list_index = self.cursor_index.saturating_sub(1);
        let mut page = paginate(self.page_size, &choices, list_index);

        let cursor_on_input = self.cursor_index == 0;
        if cursor_on_input {
            page.selection = usize::MAX;
        }

        backend.render_suggestions(page)?;

        if let Some(message) = self.help_message {
            backend.render_help_message(message)?;
        } else if !choices.is_empty() {
            backend.render_help_message(DEFAULT_HELP_MESSAGE_WITH_AC)?;
        }

        backend.frame_finish()?;

        Ok(())
    }

    fn prompt<B: TextBackend>(mut self, backend: &mut B) -> InquireResult<String> {
        let final_answer: String;
        self.update_suggestions()?;

        loop {
            self.render(backend)?;

            let key = backend.read_key()?;

            match key {
                Key::Interrupt => interrupt_prompt!(),
                Key::Cancel => cancel_prompt!(backend, self.message),
                Key::Submit => match self.validate_current_answer()? {
                    Validation::Valid => {
                        final_answer = self.get_current_answer().to_owned();
                        break;
                    }
                    Validation::Invalid(msg) => self.error = Some(msg),
                },
                key => self.on_change(key)?,
            }
        }

        let formatted = (self.formatter)(&final_answer);

        finish_prompt_with_answer!(backend, self.message, &formatted, final_answer);
    }
}

#[cfg(test)]
#[cfg(feature = "crossterm")]
mod test {
    use super::Text;
    use crate::{
        terminal::crossterm::CrosstermTerminal,
        ui::{Backend, RenderConfig},
        validator::{ErrorMessage, Validation},
    };
    use crossterm::event::{KeyCode, KeyEvent};

    fn default<'a>() -> Text<'a> {
        Text::new("Question?")
    }

    macro_rules! text_to_events {
        ($text:expr) => {{
            $text.chars().map(KeyCode::Char)
        }};
    }

    macro_rules! text_test {
        ($name:ident,$input:expr,$output:expr) => {
            text_test! {$name, $input, $output, default()}
        };

        ($name:ident,$input:expr,$output:expr,$prompt:expr) => {
            #[test]
            fn $name() {
                let read: Vec<KeyEvent> = $input.into_iter().map(KeyEvent::from).collect();
                let mut read = read.iter();

                let mut write: Vec<u8> = Vec::new();

                let terminal = CrosstermTerminal::new_with_io(&mut write, &mut read);
                let mut backend = Backend::new(terminal, RenderConfig::default()).unwrap();

                let ans = $prompt.prompt_with_backend(&mut backend).unwrap();

                assert_eq!($output, ans);
            }
        };
    }

    text_test!(empty, vec![KeyCode::Enter], "");

    text_test!(single_letter, vec![KeyCode::Char('b'), KeyCode::Enter], "b");

    text_test!(
        letters_and_enter,
        text_to_events!("normal input\n"),
        "normal input"
    );

    text_test!(
        letters_and_enter_with_emoji,
        text_to_events!("with emoji 🧘🏻‍♂️, 🌍, 🍞, 🚗, 📞\n"),
        "with emoji 🧘🏻‍♂️, 🌍, 🍞, 🚗, 📞"
    );

    text_test!(
        input_and_correction,
        {
            let mut events = vec![];
            events.append(&mut text_to_events!("anor").collect());
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.append(&mut text_to_events!("normal input").collect());
            events.push(KeyCode::Enter);
            events
        },
        "normal input"
    );

    text_test!(
        input_and_excessive_correction,
        {
            let mut events = vec![];
            events.append(&mut text_to_events!("anor").collect());
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.append(&mut text_to_events!("normal input").collect());
            events.push(KeyCode::Enter);
            events
        },
        "normal input"
    );

    text_test!(
        input_correction_after_validation,
        {
            let mut events = vec![];
            events.append(&mut text_to_events!("1234567890").collect());
            events.push(KeyCode::Enter);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.push(KeyCode::Backspace);
            events.append(&mut text_to_events!("yes").collect());
            events.push(KeyCode::Enter);
            events
        },
        "12345yes",
        Text::new("").with_validator(|ans: &str| match ans.len() {
            len if len > 5 && len < 10 => Ok(Validation::Valid),
            _ => Ok(Validation::Invalid(ErrorMessage::Default)),
        })
    );
}
