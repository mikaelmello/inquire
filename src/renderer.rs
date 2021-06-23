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

    #[allow(unused)]
    pub fn empty() -> Self {
        Self::new("")
    }

    pub fn with_fg<C: 'static + Color + Clone>(mut self, fg: C) -> Self {
        self.fg = Some(Box::new(fg.clone()));
        self
    }

    #[allow(unused)]
    pub fn with_bg<C: 'static + Color + Clone>(mut self, fg: C) -> Self {
        self.bg = Some(Box::new(fg.clone()));
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    pub fn print(&self, terminal: &mut Terminal) -> Result<(), std::io::Error> {
        if self.content.is_empty() {
            return Ok(());
        }

        if let Some(color) = self.fg.as_ref() {
            terminal.set_fg_color(color.as_ref())?;
        }
        if let Some(color) = self.bg.as_ref() {
            terminal.set_bg_color(color.as_ref())?;
        }
        if let Some(style) = &self.style {
            terminal.set_style(style.clone())?;
        }

        terminal.write(self.content)?;

        if let Some(_) = self.fg.as_ref() {
            terminal.reset_fg_color()?;
        }
        if let Some(_) = self.bg.as_ref() {
            terminal.reset_bg_color()?;
        }
        if let Some(_) = &self.style {
            terminal.reset_style()?;
        }

        Ok(())
    }
}

impl Renderer {
    pub fn reset_prompt(&mut self, terminal: &mut Terminal) -> Result<(), std::io::Error> {
        for _ in 0..self.cur_line {
            terminal.cursor_up()?;
            terminal.cursor_horizontal_reset()?;
            terminal.clear_current_line()?;
        }

        self.cur_line = 0;
        Ok(())
    }

    pub fn print_tokens(
        &mut self,
        terminal: &mut Terminal,
        tokens: &[Token],
    ) -> Result<(), std::io::Error> {
        for t in tokens {
            t.print(terminal)?;
        }
        Ok(())
    }

    pub fn print_error_message(
        &mut self,
        terminal: &mut Terminal,
        message: &str,
    ) -> Result<(), std::io::Error> {
        Token::new(&format!("# {}", message))
            .with_fg(color::Red)
            .print(terminal)?;

        self.new_line(terminal)?;

        Ok(())
    }

    pub fn print_prompt_answer(
        &mut self,
        terminal: &mut Terminal,
        prompt: &str,
        answer: &str,
    ) -> Result<(), std::io::Error> {
        self.print_tokens(
            terminal,
            &vec![
                Token::new("? ").with_fg(color::Green),
                Token::new(prompt),
                Token::new(&format!(" {}", answer)).with_fg(color::Cyan),
            ],
        )?;
        self.new_line(terminal)?;

        Ok(())
    }

    pub fn print_prompt(
        &mut self,
        terminal: &mut Terminal,
        prompt: &str,
        default: Option<&str>,
        content: Option<&str>,
    ) -> Result<(), std::io::Error> {
        Token::new("? ").with_fg(color::Green).print(terminal)?;
        Token::new(prompt).print(terminal)?;

        if let Some(default) = default {
            Token::new(&format!(" ({})", default)).print(terminal)?;
        }

        match content {
            Some(content) if !content.is_empty() => Token::new(&format!(" {}", content))
                .with_style(Style::Bold)
                .print(terminal)?,
            _ => {}
        }

        self.new_line(terminal)?;

        Ok(())
    }

    pub fn print_help(
        &mut self,
        terminal: &mut Terminal,
        message: &str,
    ) -> Result<(), std::io::Error> {
        Token::new(&format!("[{}]", message))
            .with_fg(color::Cyan)
            .print(terminal)?;
        self.new_line(terminal)?;

        Ok(())
    }

    pub fn print_option(
        &mut self,
        terminal: &mut Terminal,
        cursor: bool,
        content: &str,
    ) -> Result<(), std::io::Error> {
        match cursor {
            true => Token::new(&format!("> {}", content))
                .with_fg(color::Cyan)
                .print(terminal),
            false => Token::new(&format!("  {}", content)).print(terminal),
        }?;

        self.new_line(terminal)?;

        Ok(())
    }

    pub fn print_multi_option(
        &mut self,
        terminal: &mut Terminal,
        cursor: bool,
        checked: bool,
        content: &str,
    ) -> Result<(), std::io::Error> {
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
        )?;

        self.new_line(terminal)?;

        Ok(())
    }

    fn new_line(&mut self, terminal: &mut Terminal) -> Result<(), std::io::Error> {
        terminal.cursor_horizontal_reset()?;
        terminal.write("\n")?;
        self.cur_line = self.cur_line.saturating_add(1);

        Ok(())
    }
}
