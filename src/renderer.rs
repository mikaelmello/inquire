use termion::color;

use crate::terminal::Terminal;

#[derive(Default)]
pub struct Renderer {
    cur_line: usize,
}
pub enum Value<'a> {
    Answer(&'a str),
    Filter(&'a str),
    None,
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

    pub fn print_header(
        &mut self,
        terminal: &Terminal,
        message: &str,
        example: Option<&str>,
        value: Value,
    ) {
        terminal.set_fg_color(color::Green);
        print!("? ");
        terminal.reset_fg_color();
        print!("{}", message);

        if let Some(example) = example {
            terminal.set_fg_color(color::LightBlack);
            print!(" ({})", example);
            terminal.reset_fg_color();
        }

        match value {
            Value::Answer(answer) => {
                terminal.set_fg_color(color::Cyan);
                print!(" {}", answer);
                terminal.reset_fg_color();
            }
            Value::Filter(filter) => {
                terminal.set_bold_style();
                print!(" {}", filter);
                terminal.reset_style();
            }
            Value::None => {}
        }

        self.new_line();
    }

    pub fn print_multi_option(
        &mut self,
        terminal: &Terminal,
        cursor: bool,
        checked: bool,
        content: &str,
    ) {
        match cursor {
            true => {
                terminal.set_fg_color(color::Cyan);
                print!("> ");
                terminal.reset_fg_color();
            }
            false => print!("  "),
        }

        match checked {
            true => {
                terminal.set_fg_color(color::Green);
                print!("[x]");
                terminal.reset_fg_color();
            }
            false => print!("[ ]"),
        }

        print!(" {}", content);

        self.new_line();
    }

    fn new_line(&mut self) {
        print!("\n\r");
        self.cur_line = self.cur_line.saturating_add(1);
    }
}
