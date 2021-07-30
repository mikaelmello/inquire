use std::cmp::min;

use crate::{
    config::{self, Suggester},
    error::{InquireError, InquireResult},
    formatter::{StringFormatter, DEFAULT_STRING_FORMATTER},
    input::Input,
    key::{Key, KeyModifiers},
    option_answer::OptionAnswer,
    renderer::Renderer,
    terminal::Terminal,
    utils::paginate,
    validator::StringValidator,
};

const DEFAULT_HELP_MESSAGE: &str = "↑↓ to move, tab to auto-complete, enter to submit";

/// Prompts the user for a single line of text input.
#[derive(Clone)]
pub struct Text<'a> {
    /// Message to be presented to the user.
    pub message: &'a str,

    /// Default value, returned when the user input is empty.
    pub default: Option<&'a str>,

    /// Help message to be presented to the user.
    pub help_message: Option<&'a str>,

    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: StringFormatter<'a>,

    /// Collection of validators to apply to the user input.
    ///
    /// Validators are executed in the order they are stored, stopping at and displaying to the user
    /// only the first validation error that might appear.
    ///
    /// The possible error is displayed to the user one line above the prompt.
    pub validators: Vec<StringValidator<'a>>,

    /// Page size of the suggestions displayed to the user, when applicable.
    pub page_size: usize,

    /// Function that provides a list of suggestions to the user based on the current input.
    pub suggester: Option<Suggester<'a>>,
}

impl<'a> Text<'a> {
    /// Default formatter, set to [DEFAULT_STRING_FORMATTER](crate::formatter::DEFAULT_STRING_FORMATTER)
    pub const DEFAULT_FORMATTER: StringFormatter<'a> = DEFAULT_STRING_FORMATTER;

    /// Default page size, equal to the global default page size [config::DEFAULT_PAGE_SIZE]
    pub const DEFAULT_PAGE_SIZE: usize = config::DEFAULT_PAGE_SIZE;

    /// Default validators added to the [Text] prompt, none.
    pub const DEFAULT_VALIDATORS: Vec<StringValidator<'a>> = vec![];

    /// Default help message.
    pub const DEFAULT_HELP_MESSAGE: Option<&'a str> = None;

    /// Creates a [Text] with the provided message and default options.
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            default: None,
            help_message: Self::DEFAULT_HELP_MESSAGE,
            validators: Self::DEFAULT_VALIDATORS,
            formatter: Self::DEFAULT_FORMATTER,
            page_size: Self::DEFAULT_PAGE_SIZE,
            suggester: None,
        }
    }

    /// Sets the help message of the prompt.
    pub fn with_help_message(mut self, message: &'a str) -> Self {
        self.help_message = Some(message);
        self
    }

    /// Sets the default input.
    pub fn with_default(mut self, message: &'a str) -> Self {
        self.default = Some(message);
        self
    }

    /// Sets the suggester.
    pub fn with_suggester(mut self, suggester: Suggester<'a>) -> Self {
        self.suggester = Some(suggester);
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
    pub fn with_validator(mut self, validator: StringValidator<'a>) -> Self {
        self.validators.push(validator);
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
    pub fn with_validators(mut self, validators: &[StringValidator<'a>]) -> Self {
        for validator in validators {
            self.validators.push(validator.clone());
        }
        self
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to the defined rules.
    pub fn prompt(self) -> InquireResult<String> {
        let terminal = Terminal::new()?;
        let mut renderer = Renderer::new(terminal)?;
        self.prompt_with_renderer(&mut renderer)
    }

    pub(in crate) fn prompt_with_renderer(self, renderer: &mut Renderer) -> InquireResult<String> {
        TextPrompt::from(self).prompt(renderer)
    }
}

struct TextPrompt<'a> {
    message: &'a str,
    default: Option<&'a str>,
    help_message: Option<&'a str>,
    input: Input,
    original_input: Option<Input>,
    formatter: StringFormatter<'a>,
    validators: Vec<StringValidator<'a>>,
    error: Option<String>,
    suggester: Option<Suggester<'a>>,
    suggested_options: Vec<String>,
    cursor_index: usize,
    page_size: usize,
}

impl<'a> From<Text<'a>> for TextPrompt<'a> {
    fn from(so: Text<'a>) -> Self {
        Self {
            message: so.message,
            default: so.default,
            help_message: so.help_message,
            formatter: so.formatter,
            validators: so.validators,
            suggester: so.suggester,
            input: Input::new(),
            original_input: None,
            error: None,
            cursor_index: 0,
            page_size: so.page_size,
            suggested_options: match so.suggester {
                Some(s) => s(""),
                None => vec![],
            },
        }
    }
}

impl<'a> From<&'a str> for Text<'a> {
    fn from(val: &'a str) -> Self {
        Text::new(val)
    }
}

impl<'a> TextPrompt<'a> {
    fn update_suggestions(&mut self) {
        match self.suggester {
            Some(suggester) => {
                self.suggested_options = suggester(self.input.content());
                self.cursor_index = 0;
            }
            None => {}
        }
    }

    fn move_cursor_up(&mut self) {
        self.cursor_index = self.cursor_index.saturating_sub(1);
    }

    fn move_cursor_down(&mut self) {
        self.cursor_index = min(
            self.cursor_index.saturating_add(1),
            self.suggested_options.len(),
        );
    }

    fn on_change(&mut self, key: Key) {
        match key {
            Key::Up(KeyModifiers::NONE) => self.move_cursor_up(),
            Key::Down(KeyModifiers::NONE) => self.move_cursor_down(),
            key => {
                let dirty = self.input.handle_key(key);

                if dirty {
                    self.original_input.take();
                    self.update_suggestions();
                }
            }
        }

        self.update_current_input();
    }

    fn update_current_input(&mut self) {
        if self.cursor_index == 0 {
            if let Some(input) = self.original_input.take() {
                self.input = input;
            }
        } else {
            let suggestion = self
                .suggested_options
                .get(self.cursor_index - 1)
                .map(|s| &**s);

            if let Some(suggestion) = suggestion {
                if self.original_input.is_none() {
                    self.original_input = Some(self.input.clone());
                }
                self.input.reset_with(suggestion);
            }
        }
    }

    fn get_final_answer(&self) -> Result<String, String> {
        if self.input.content().is_empty() {
            match self.default {
                Some(val) => return Ok(val.to_string()),
                None => {}
            }
        }

        for validator in &self.validators {
            match validator(self.input.content()) {
                Ok(_) => {}
                Err(err) => return Err(err),
            }
        }

        Ok(self.input.content().into())
    }

    fn render(&mut self, renderer: &mut Renderer) -> InquireResult<()> {
        let prompt = &self.message;

        renderer.reset_prompt()?;

        if let Some(err) = &self.error {
            renderer.print_error_message(err)?;
        }

        renderer.print_prompt_input(&prompt, self.default, &self.input)?;

        let choices = self
            .suggested_options
            .iter()
            .enumerate()
            .map(|(i, val)| OptionAnswer::new(i, val))
            .collect::<Vec<OptionAnswer>>();

        let display_cursor = self.cursor_index > 0;
        let list_index = self.cursor_index.saturating_sub(1);
        let page = paginate(self.page_size, &choices, list_index);

        for (idx, opt) in page.content.iter().enumerate() {
            renderer.print_option(display_cursor && page.selection == idx, &opt.value)?;
        }

        if let Some(message) = self.help_message {
            renderer.print_help(message)?;
        } else if !choices.is_empty() {
            renderer.print_help(DEFAULT_HELP_MESSAGE)?;
        }

        renderer.flush()?;

        Ok(())
    }

    fn prompt(mut self, renderer: &mut Renderer) -> InquireResult<String> {
        let final_answer: String;

        loop {
            self.render(renderer)?;

            let key = renderer.read_key()?;

            match key {
                Key::Cancel => return Err(InquireError::OperationCanceled),
                Key::Submit => match self.get_final_answer() {
                    Ok(answer) => {
                        final_answer = answer;
                        break;
                    }
                    Err(err) => self.error = Some(err),
                },
                key => self.on_change(key),
            }
        }

        renderer.cleanup(&self.message, &(self.formatter)(&final_answer))?;

        Ok(final_answer)
    }
}

#[cfg(test)]
mod test {
    use crossterm::event::{KeyCode, KeyEvent};
    use ntest::timeout;

    use crate::{renderer::Renderer, terminal::Terminal};

    use super::Text;

    fn default<'a>() -> Text<'a> {
        Text::new("Question?")
    }

    macro_rules! text_to_events {
        ($text:expr) => {{
            $text.chars().map(|c| KeyCode::Char(c))
        }};
    }

    macro_rules! text_test {
        ($name:ident,$input:expr,$output:expr) => {
            text_test! {$name, $input, $output, default()}
        };

        ($name:ident,$input:expr,$output:expr,$prompt:expr) => {
            #[test]
            #[timeout(100)]
            fn $name() {
                let read: Vec<KeyEvent> = $input.into_iter().map(KeyEvent::from).collect();
                let mut read = read.iter();

                let mut write: Vec<u8> = Vec::new();
                let terminal = Terminal::new_with_io(&mut write, &mut read);
                let mut renderer = Renderer::new(terminal).unwrap();

                let ans = $prompt.prompt_with_renderer(&mut renderer).unwrap();

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
        Text::new("").with_validator(&|ans| match ans.len() {
            len if len > 5 && len < 10 => Ok(()),
            _ => Err("Invalid".to_string()),
        })
    );
}
