use termion::color::{self, Color};

use crate::terminal::{Style, Terminal};

#[derive(Default)]
pub struct Renderer {
    cur_line: usize,
}

pub struct Token<'a> {
    pub content: &'a str,
    pub fg: Option<Box<dyn Color>>,
    pub bg: Option<Box<dyn Color>>,
    pub style: Option<Style>,
}

impl<'a> Token<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            fg: None,
            bg: None,
            style: None,
        }
    }

    pub fn empty() -> Self {
        Self::new("")
    }

    pub fn with_fg<C: 'static + Color + Clone>(mut self, fg: C) -> Self {
        self.fg = Some(Box::new(fg.clone()));
        self
    }

    pub fn with_bg<C: 'static + Color + Clone>(mut self, fg: C) -> Self {
        self.bg = Some(Box::new(fg.clone()));
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    pub fn print(&self, terminal: &mut Terminal) {
        if self.content.is_empty() {
            return;
        }

        if let Some(color) = self.fg.as_ref() {
            terminal.set_fg_color(color.as_ref());
        }
        if let Some(color) = self.bg.as_ref() {
            terminal.set_bg_color(color.as_ref());
        }
        if let Some(style) = &self.style {
            terminal.set_style(style.clone());
        }

        print!("{}", self.content);

        if let Some(_) = self.fg.as_ref() {
            terminal.undo_fg_color();
        }
        if let Some(_) = self.bg.as_ref() {
            terminal.undo_bg_color();
        }
        if let Some(_) = &self.style {
            terminal.undo_style();
        }
    }
}

impl Renderer {
    pub fn reset_prompt(&mut self, terminal: &Terminal) {
        for _ in 0..self.cur_line {
            terminal.cursor_up();
            terminal.cursor_horizontal_reset();
            terminal.clear_current_line();
        }

        self.cur_line = 0;
    }

    pub fn print_tokens(&mut self, terminal: &mut Terminal, tokens: &[Token]) {
        tokens.iter().for_each(|t| t.print(terminal));
    }

    pub fn print_prompt_answer(&mut self, terminal: &mut Terminal, prompt: &str, answer: &str) {
        self.print_tokens(
            terminal,
            &vec![
                Token::new("? ").with_fg(color::Green),
                Token::new(prompt),
                Token::new(&format!(" {}", answer)).with_fg(color::Cyan),
            ],
        );
        self.new_line();
    }

    pub fn print_prompt_filter(&mut self, terminal: &mut Terminal, prompt: &str, filter: &str) {
        self.print_tokens(
            terminal,
            &vec![
                Token::new("? ").with_fg(color::Green),
                Token::new(prompt),
                Token::new(&format!(" {}", filter)).with_style(Style::Bold),
            ],
        );
        self.new_line();
    }

    pub fn print_prompt(&mut self, terminal: &mut Terminal, prompt: &str) {
        self.print_tokens(
            terminal,
            &vec![Token::new("? ").with_fg(color::Green), Token::new(prompt)],
        );
        self.new_line();
    }

    pub fn print_help(&mut self, terminal: &mut Terminal, message: &str) {
        Token::new(&format!("[{}]", message))
            .with_fg(color::Cyan)
            .print(terminal);
        self.new_line();
    }

    pub fn print_multi_option(
        &mut self,
        terminal: &mut Terminal,
        cursor: bool,
        checked: bool,
        content: &str,
    ) {
        self.print_tokens(
            terminal,
            &vec![
                match cursor {
                    true => Token::new("> ").with_fg(color::Cyan),
                    false => Token::new("  "),
                },
                match checked {
                    true => Token::new("[x] ").with_fg(color::Green),
                    false => Token::new("[ ] "),
                },
                Token::new(content),
            ],
        );

        self.new_line();
    }

    fn new_line(&mut self) {
        print!("\n\r");
        self.cur_line = self.cur_line.saturating_add(1);
    }
}
