# inquire

[![Latest Version]][crates.io] ![Build] ![Platforms] ![License]

[crates.io]: https://crates.io/crates/inquire
[Latest Version]: https://img.shields.io/crates/v/inquire.svg
[Build]: https://github.com/mikaelmello/inquire/actions/workflows/test.yml/badge.svg
[Platforms]: https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-success
[License]: https://img.shields.io/crates/l/inquire.svg

`inquire` is a library for building interactive prompts on terminals, inspired by [survey](https://github.com/AlecAivazis/survey).

## Demo

![Animated GIF making a demonstration of a questionnaire created with this library. You can replay this recording in your terminal with asciinema play command - asciinema play ./assets/expense_tracker.cast](assets/expense_tracker.gif)
[Source](examples/expense_tracker.rs)

## Examples

Examples can be found in the `examples` directory. Run them to see basic behavior:

```
$ cargo run --example expense_tracker --features date
```

## Features

- Cross-platform, supporting UNIX and Windows terminals (thanks to [crossterm](https://github.com/crossterm-rs/crossterm)).
- Five (5) kinds of prompts and their own custom features (to see the full list of features check the full documentation below):
  - Text input, with auto-completion support.
  - Date picker, with support to limit allowed date ranges.
  - Selection inputs, with support for single and multiple selections, as well as custom filtering based on user input.
  - Confirm prompt, with support for custom parsing of input.
  - Password prompt, where user input is not echoed back to the terminal.
- Standardized error handling (thanks to [thiserror](https://github.com/dtolnay/thiserror))
- Support for customized help messages.
- Support for default values.
- Support for validation of user input.
- Support for custom formatting of user input after it is submitted and echoed back to the terminal.
- Fine-grained configuration, e.g. page size of option list, vim mode for navigation, etc.

## Cross-cutting concerns

There are several features that are shared among different types of prompts. This section will give an overview on each of them.

### Validation

### Formatting

### Parsing

### Filtering

### Error handling


## Prompts

Currently, there are 5 different prompt types supported.

### Text

Text is the standard kind of prompt you would expect from a library like this one. It displays a message to the user, prompting them to type something back. The user's input is then stored in a `String` and returned to the prompt caller.

```rust
// ./examples/text_simple.rs

let name = Text::new("What is your name?").prompt();

match name {
    Ok(name) => println!("Hello {}", name),
    Err(_) => println!("An error happened when asking for your name, try again later."),
}
```

![Animated GIF making a demonstration of a simple prompt with Text created with this library. You can replay this recording in your terminal with asciinema play command using the file ./assets/text_simple.cast](assets/text_simple.gif)

With Text, you can customize several aspects:

- Help message: Display a helpful message at the line below the prompt.
- Default value: Set a default value for when the user just presses enter without any text input.


### DateSelect

### Select

### MultiSelect

### Password

### Confirm
