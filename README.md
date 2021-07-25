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

Almost all prompts provide an API to set custom validators.

The validators provided to a given prompt are called whenever the user submits their input. These validators vary by prompt type, receiving different types of variables as arguments, such as `&str`, `&[OptionAnswer]`, or `NaiveDate`, but their return type are always the same: `Result<(), String>`.

If the input provided by the user is invalid, your validator should return `Ok(())`.

If the input is not valid, your validator should return `Err(String)`, where the content of `Err` is a string whose content will be displayed to the user as an error message. It is recommended that this value gives a helpful feedback to the user, e.g. "This field should contain at least 5 characters".

The validators are typed as a reference to `dyn Fn`. This allows both functions and closures to be used as validators, but it also means that the functions can not hold any mutable references.

Finally, `inquire` has a feature called `builtin_validators` that is included by default. When the feature is on, several built-in validators are exported at the root-level of the library in the form of macros, check their documentation to see more details.

The docs provide full-featured examples.

### Formatting

Formatting is the process of transforming the user input into a readable output displayed after the user submits their response. By default, this is in some cases just echoing back the input itself, such as in Text prompts. Other prompts have different formatting rules by default, for example DateSelect which formats the selected date into something like "August 5, 2021".

All prompts provide an API to set custom formatters. By setting a formatter, you can customize how the user's response is displayed to them. For example, you might want to format a selected date into a new format such as "05/08/2021".

Custom formatters receive the input as an argument, with varying types such as `&str`, `chrono::NaiveDate`, and return a `String` containing the output to be displayed to the user. Check the docs for specific examples.

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

The Select prompt does not support custom validators because of the nature of the prompt. A submission always selects exactly one of the options. If this option was not supposed to be selected or is invalid in some way, it probably should not be included in the options list.

### MultiSelect

### Password

### Confirm

The Confirm prompt does not support custom validators because of the nature of the prompt. The user input is always parsed to true or false. If one of the two alternatives is invalid, a Confirm prompt that only allows yes or no answers does not make a lot of sense to me, but if someone provides a clear use-case I will reconsider.
