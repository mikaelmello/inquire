use unicode_segmentation::UnicodeSegmentation;

use termion::event::Key;

use crate::{
    answer::OptionAnswer,
    config::{self, Suggester},
    error::{InquireError, InquireResult},
    formatter::{StringFormatter, DEFAULT_STRING_FORMATTER},
    renderer::Renderer,
    terminal::Terminal,
    utils::paginate,
    validator::StringValidator,
};

const DEFAULT_HELP_MESSAGE: &str = "↑↓ to move, tab to auto-complete, enter to submit";

/// Presents a message to the user and retrieves a single line of text input.
#[derive(Clone)]
pub struct Text<'a> {
    /// Message to be presented to the user.
    pub message: &'a str,

    /// Default value, returned when the user input is empty.
    pub default: Option<&'a str>,

    /// Help message to be presented to the user.
    pub help_message: Option<&'a str>,

    /// Function that formats the user input and presents it to the user as the final rendering of the prompt.
    pub formatter: StringFormatter,

    /// Collection of validators to apply to the user input.
    /// Validation errors are displayed to the user one line above the prompt.
    pub validators: Vec<StringValidator<'a>>,

    /// Page size of the suggestions displayed to the user, when applicable.
    pub page_size: usize,

    /// Function that provides a list of suggestions to the user based on the current input.
    pub suggester: Option<Suggester>,
}

impl<'a> Text<'a> {
    /// Default page size, equal to the global default page size [config::DEFAULT_PAGE_SIZE]
    pub const DEFAULT_PAGE_SIZE: usize = config::DEFAULT_PAGE_SIZE;
    /// Default formatter.
    pub const DEFAULT_FORMATTER: StringFormatter = DEFAULT_STRING_FORMATTER;
    /// Default collection of validators.
    pub const DEFAULT_VALIDATORS: Vec<StringValidator<'a>> = Vec::new();
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
    pub fn with_suggester(mut self, suggester: Suggester) -> Self {
        self.suggester = Some(suggester);
        self
    }

    /// Sets the formatter
    pub fn with_formatter(mut self, formatter: StringFormatter) -> Self {
        self.formatter = formatter;
        self
    }

    /// Adds a validator to the collection of validators.
    pub fn with_validator(mut self, validator: StringValidator<'a>) -> Self {
        self.validators.push(validator);
        self
    }

    /// Adds the validators to the collection of validators.
    pub fn with_validators(mut self, validators: &[StringValidator<'a>]) -> Self {
        for validator in validators {
            self.validators.push(validator.clone());
        }
        self
    }

    /// Parses the provided behavioral and rendering options and prompts
    /// the CLI user for input according to them.
    pub fn prompt(self) -> InquireResult<String> {
        let terminal = Terminal::new()?;
        let mut renderer = Renderer::new(terminal)?;
        self.prompt_with_renderer(&mut renderer)
    }

    pub(in crate) fn prompt_with_renderer(self, renderer: &mut Renderer) -> InquireResult<String> {
        TextPrompt::from(self).prompt(renderer)
    }
}

/// Trait to call prompt on a collection of [Text] instances.
pub trait PromptMany {
    /// Calls prompt on a collection of [Text] instances and return their respective
    /// responses or the first error that appears.
    fn prompt(self) -> InquireResult<Vec<String>>;
}

impl<'a, I> PromptMany for I
where
    I: Iterator<Item = Text<'a>>,
{
    fn prompt(self) -> InquireResult<Vec<String>> {
        self.map(Text::prompt).collect()
    }
}

struct TextPrompt<'a> {
    message: &'a str,
    default: Option<&'a str>,
    help_message: Option<&'a str>,
    content: String,
    formatter: StringFormatter,
    validators: Vec<StringValidator<'a>>,
    error: Option<String>,
    suggester: Option<Suggester>,
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
            content: String::new(),
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
                self.suggested_options = suggester(&self.content);
                if self.suggested_options.len() > 0
                    && self.suggested_options.len() <= self.cursor_index
                {
                    self.cursor_index = self.suggested_options.len().saturating_sub(1);
                }
            }
            None => {}
        }
    }

    fn move_cursor_up(&mut self) {
        self.cursor_index = self
            .cursor_index
            .checked_sub(1)
            .or(self.suggested_options.len().checked_sub(1))
            .unwrap_or_else(|| 0);
    }

    fn move_cursor_down(&mut self) {
        self.cursor_index = self.cursor_index.saturating_add(1);
        if self.cursor_index >= self.suggested_options.len() {
            self.cursor_index = 0;
        }
    }

    fn on_change(&mut self, key: Key) {
        let mut dirty = false;

        match key {
            Key::Backspace => {
                let len = self.content[..].graphemes(true).count();
                let new_len = len.saturating_sub(1);
                self.content = self.content[..].graphemes(true).take(new_len).collect();
                dirty = true;
            }
            Key::Up => self.move_cursor_up(),
            Key::Down => self.move_cursor_down(),
            Key::Char('\x17') | Key::Char('\x18') => {
                self.content.clear();
                dirty = true;
            }
            Key::Char(c) => {
                self.content.push(c);
                dirty = true;
            }
            _ => {}
        }

        if dirty {
            self.update_suggestions();
        }
    }

    fn use_select_option(&mut self) {
        let selected_suggestion = self.suggested_options.get(self.cursor_index);

        if let Some(ans) = selected_suggestion {
            self.content = ans.clone();
            self.update_suggestions();
        }
    }

    fn get_final_answer(&self) -> Result<String, String> {
        if self.content.is_empty() {
            match self.default {
                Some(val) => return Ok(val.to_string()),
                None => {}
            }
        }

        for validator in &self.validators {
            match validator(&self.content) {
                Ok(_) => {}
                Err(err) => return Err(err),
            }
        }

        Ok(self.content.clone())
    }

    fn render(&mut self, renderer: &mut Renderer) -> InquireResult<()> {
        let prompt = &self.message;

        renderer.reset_prompt()?;

        if let Some(err) = &self.error {
            renderer.print_error_message(err)?;
        }

        renderer.print_prompt(&prompt, self.default, Some(&self.content))?;

        let choices = self
            .suggested_options
            .iter()
            .enumerate()
            .map(|(i, val)| OptionAnswer::new(i, val))
            .collect::<Vec<OptionAnswer>>();

        let (paginated_opts, rel_sel) = paginate(self.page_size, &choices, self.cursor_index);
        for (idx, opt) in paginated_opts.iter().enumerate() {
            renderer.print_option(rel_sel == idx, &opt.value)?;
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
                Key::Ctrl('c') => return Err(InquireError::OperationCanceled),
                Key::Char('\t') => self.use_select_option(),
                Key::Char('\n') | Key::Char('\r') => match self.get_final_answer() {
                    Ok(answer) => {
                        final_answer = answer;
                        break;
                    }
                    Err(err) => self.error = Some(err),
                },
                key => self.on_change(key),
            }
        }

        renderer.cleanup(&self.message, (self.formatter)(&final_answer))?;

        Ok(final_answer)
    }
}

#[cfg(test)]
mod test {
    use ntest::timeout;

    use crate::{renderer::Renderer, terminal::Terminal};

    use super::Text;

    fn default<'a>() -> Text<'a> {
        Text::new("Question?")
    }

    macro_rules! text_test {
        ($name:ident,$input:expr,$output:expr) => {
            text_test! {$name, $input, $output, default()}
        };

        ($name:ident,$input:expr,$output:expr,$prompt:expr) => {
            #[test]
            #[timeout(100)]
            fn $name() {
                let mut read: &[u8] = $input.as_bytes();

                let mut write: Vec<u8> = Vec::new();
                let terminal = Terminal::new_with_io(&mut write, &mut read);
                let mut renderer = Renderer::new(terminal).unwrap();

                let ans = $prompt.prompt_with_renderer(&mut renderer).unwrap();

                assert_eq!($output, ans);
            }
        };
    }

    text_test!(empty, "\n", "");

    text_test!(single_letter, "b\n", "b");

    text_test!(letters_and_enter, "normal input\n", "normal input");

    text_test!(
        letters_and_enter_with_emoji,
        "with emoji 🧘🏻‍♂️, 🌍, 🍞, 🚗, 📞\n",
        "with emoji 🧘🏻‍♂️, 🌍, 🍞, 🚗, 📞"
    );

    text_test!(
        input_and_correction,
        "anor\x7F\x7F\x7F\x7Fnormal input\n",
        "normal input"
    );

    text_test!(
        input_and_excessive_correction,
        "anor\x7F\x7F\x7F\x7F\x7F\x7F\x7F\x7Fnormal input\n",
        "normal input"
    );

    text_test!(
        input_correction_after_validation,
        "1234567890\n\x7F\x7F\x7F\x7F\x7F\nyes\n",
        "12345yes",
        Text::new("").with_validator(&|ans| match ans.len() {
            len if len > 5 && len < 10 => Ok(()),
            _ => Err("Invalid".to_string()),
        })
    );
}
