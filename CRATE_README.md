[![Latest Version]][crates.io] ![Build status] ![Supported platforms] ![License]

[crates.io]: https://crates.io/crates/inquire
[Latest Version]: https://img.shields.io/crates/v/inquire.svg
[Build status]: https://github.com/mikaelmello/inquire/actions/workflows/test.yml/badge.svg
[Supported platforms]: https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-success
[License]: https://img.shields.io/crates/l/inquire.svg

---

<p align="center">
  <img width="460" src="./assets/inquire.png">
  <br>
  <code>inquire</code> is a library for building interactive prompts on terminals.
</p>

It provides several different prompts in order to interactively ask the user for information via the CLI. With `inquire`, you can use:
- [`Text`] to get text input from the user, with _built-in auto-completion support_;
- [`DateSelect`]* to get a date input from the user, selected via an _interactive calendar_;
- [`Select`] to ask the user to select one option from a given list;
- [`MultiSelect`] to ask the user to select an arbitrary number of options from a given list;
- [`Confirm`] for simple yes/no confirmation prompts;
- [`CustomType`] for text prompts that you would like to parse to a custom type, such as numbers or UUIDs;
- [`Password`] for secretive text prompts.

---

## Demo

![Animated GIF making a demonstration of a questionnaire created with this library. You can replay this recording in your terminal with asciinema play command - asciinema play ./assets/expense_tracker.cast](./assets/expense_tracker.gif)
[Source](./examples/expense_tracker.rs)

## Features

- Cross-platform, supporting UNIX and Windows terminals (thanks to [crossterm](https://crates.io/crates/crossterm));
- Several kinds of prompts to suit your needs;
- Standardized error handling (thanks to [thiserror](https://crates.io/crates/thiserror));
- Support for fine-grained configuration for each prompt type, allowing you to customize:
  - Default values;
  - Input validators and formatters;
  - Help messages;
  - Auto-completion for [`Text`] prompts;
  - Custom list filters for Select and [`MultiSelect`] prompts;
  - Custom parsers for [`Confirm`] and [`CustomType`] prompts;
  - and many others!

## Usage

Put this line in your `Cargo.toml`, under `[dependencies]`.

```
inquire = "0.0.6"
```

\* If you'd like to use the date-related features, enable the `date` feature:

```
inquire = { version = "0.0.6", features = ["date"] }
```

[`Text`]: https://docs.rs/inquire/*/inquire/prompts/text/struct.Text.html
[`DateSelect`]: https://docs.rs/inquire/*/inquire/prompts/dateselect/struct.DateSelect.html
[`Select`]: https://docs.rs/inquire/*/inquire/prompts/select/struct.Select.html
[`MultiSelect`]: https://docs.rs/inquire/*/inquire/prompts/multiselect/struct.MultiSelect.html
[`Confirm`]: https://docs.rs/inquire/*/inquire/prompts/confirm/struct.Confirm.html
[`CustomType`]: https://docs.rs/inquire/*/inquire/prompts/customtype/struct.CustomType.html
[`Password`]: https://docs.rs/inquire/*/inquire/prompts/password/struct.Password.html