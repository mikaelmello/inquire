//! Restores the terminal when the process receives SIGTERM while a prompt is open.
//!
//! A signal like SIGTERM ends the process without running destructors, so
//! inquire never gets the chance to restore the terminal. This example
//! registers a handler that restores it and then exits.
//!
//! Try it: run the example, then send `kill -TERM <pid>` from another terminal.

#[cfg(unix)]
fn main() {
    use std::io::stderr;

    use crossterm::{cursor, event, execute, terminal};
    use signal_hook::{consts::SIGTERM, iterator::Signals};

    let mut signals = Signals::new([SIGTERM]).expect("failed to register SIGTERM handler");

    std::thread::spawn(move || {
        if let Some(signal) = signals.forever().next() {
            // Undo what inquire's crossterm backend changes while a prompt is
            // active. These calls are harmless if no prompt is open.
            let _ = execute!(stderr(), event::DisableBracketedPaste, cursor::Show);
            let _ = terminal::disable_raw_mode();
            eprintln!();

            // Conventional exit code for "terminated by signal N".
            std::process::exit(128 + signal);
        }
    });

    let name = inquire::Text::new("What is your name?")
        .with_help_message("Send SIGTERM to this process to see the terminal restored")
        .prompt();

    match name {
        Ok(name) => println!("Hello {name}"),
        Err(err) => println!("Error: {err}"),
    }
}

#[cfg(not(unix))]
fn main() {
    println!("This example uses Unix signals and only runs on Unix-like systems.");
}
