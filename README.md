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
- 


### Password

### Confirm

### Select

### MultiSelect

### DateSelect

## Prompt-generic concerns

### Validation

### Formatting

### Option Filtering