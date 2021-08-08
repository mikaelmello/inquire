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

## Usage

Put this line in your `Cargo.toml`, under `[dependencies]`.

```
inquire = "0.0.6"
```

\* If you'd like to use the date-related features, enable the `date` feature:

```
inquire = { version = "0.0.6", features = ["date"] }
```

## Examples

Examples can be found in the `examples` directory. Run them to see basic behavior:

```
$ cargo run --example expense_tracker --features date
```

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

# Cross-cutting concerns

There are several features that are shared among different types of prompts. This section will give an overview on each of them.

## Validation

Almost all prompts provide an API to set custom validators.

The validators provided to a given prompt are called whenever the user submits their input. These validators vary by prompt type, receiving different types of variables as arguments, such as `&str`, `&[OptionAnswer]`, or `NaiveDate`, but their return type are always the same: `Result<(), String>`.

If the input provided by the user is invalid, your validator should return `Ok(())`.

If the input is not valid, your validator should return `Err(String)`, where the content of `Err` is a string whose content will be displayed to the user as an error message. It is recommended that this value gives a helpful feedback to the user, e.g. "This field should contain at least 5 characters".

The validators are typed as a reference to `dyn Fn`. This allows both functions and closures to be used as validators, but it also means that the functions can not hold any mutable references.

Finally, `inquire` has a feature called `builtin_validators` that is included by default. When the feature is on, several built-in validators are exported at the root-level of the library in the form of macros, check their documentation to see more details.

The docs provide full-featured examples.

In the [demo](#Demo) you can see the behavior of an input not passing the requirements in the _amount_ prompt, when the error message "Please type a valid number" is displayed. _Full disclosure, this error message was displayed due to a parsing, not validation, error, but the user experience is the same for both cases._

## Formatting

Formatting is the process of transforming the user input into a readable output displayed after the user submits their response. By default, this is in some cases just echoing back the input itself, such as in Text prompts. Other prompts have different formatting rules by default, for example DateSelect which formats the selected date into something like "August 5, 2021".

All prompts provide an API to set custom formatters. By setting a formatter, you can customize how the user's response is displayed to them. For example, you might want to format a selected date into a new format such as "05/08/2021".

Custom formatters receive the input as an argument, with varying types such as `&str`, `chrono::NaiveDate`, and return a `String` containing the output to be displayed to the user. Check the docs for specific examples.

In the [demo](#Demo) you can see this behavior in action with the _amount_ (CustomType) prompt, where a custom formatter adds a '$' character preffix to the input.

## Parsing

Parsing features are related to two prompts: [`Confirm`] and [`CustomType`]. They return to you a value (of types `bool` or any custom type you might want) parsed from the user's text input. In both cases, you can either use default parsers that are already built-in or provide custom ones adhering to the function signatures.

The default `bool` parser returns `true` if the input is either `"y"` or `"yes"`, in a case-insensitive comparison. Similarly, the parser returns `false` if the input is either `"n"` or `"no"`.

The default parser for [`CustomType`] prompts calls the `parse::<T>()` method on the input string. This means that if you want to create a [`CustomType`] with default settings, the wanted return type must implement the `FromStr` trait.

In the [demo](#Demo) you can see this behavior in action with the _amount_ (CustomType) prompt.

## Filtering

Filtering is applicable to two prompts: [`Select`] and [`MultiSelect`]. They provide the user the ability to filter the options based on their text input. This is specially useful when there are a lot of options for the user to choose from, allowing them to quickly find their expected options.

Filter functions receive three arguments: the current user input, the option string value and the option index. They must return a `bool` value indicating whether the option should be part of the results or not.

The default filter function does a naive case-insensitive comparison between the option string value and the current user input, returning `true` if the option string value contains the user input as a substring.

In the [demo](#Demo) you can see this behavior in action with the *account* (Select) and *tags* (MultiSelect) prompts. 

## Error handling

Error handling when using `inquire` is pretty simple. Instantiating prompt structs is not fallible by design, in order to avoid requiring chaining of `map` and `and_then` methods to subsequent configuration method calls such as `with_help_message()`. All fallible operations are exposable only when you call `prompt()` on the instantiated prompt struct.

`prompt` calls return a `Result` containing either your expected response value or an `Err` of type `InquireError`. An `InquireError` has the following variants:

- **NotTTY**: The input device is not a TTY, which means that enabling raw mode on the terminal in order to listen to input events is not possible. I currently do not know if it is possible to make the library work even if that's the case.
- **InvalidConfiguration(String)**: Some aspects of the prompt configuration were considered to be invalid, with more details given in the value string.
  - This error is only possible in [`Select`], [`MultiSelect`] and [`DateSelect`] prompts, where specific settings might be incompatible. All other prompts always have valid configurations by design.
- **IO(io::Error)**: There was an error when performing IO operations. IO errors are not handled inside `inquire` to keep the library simple.
- **OperationCanceled**: The user canceled the prompt before submitting a response. The user might cancel the operation by pressing `Ctrl-C` or `ESC`.

# Prompts

Currently, there are 5 different prompt types supported.

## Text

`Text` is the standard kind of prompt you would expect from a library like this one. It displays a message to the user, prompting them to type something back. The user's input is then stored in a `String` and returned to the prompt caller.

```rust
let name = Text::new("What is your name?").prompt();

match name {
    Ok(name) => println!("Hello {}", name),
    Err(_) => println!("An error happened when asking for your name, try again later."),
}
```

![Animated GIF making a demonstration of a simple prompt with Text created with this library. You can replay this recording in your terminal with asciinema play command using the file ./assets/text_simple.cast](./assets/text_simple.gif)

With `Text`, you can customize several aspects:

- **Prompt message**: Main message when prompting the user for input, `"What is your name?"` in the example above.
- **Help message**: Message displayed at the line below the prompt.
- **Default value**: Default value returned when the user submits an empty response.
- **Validators**: Custom validators to the user's input, displaying an error message if the input does not pass the requirements.
- **Formatter**: Custom formatter in case you need to pre-process the user input before showing it as the final answer.

### Autocomplete

With `Text` inputs, it is also possible to set-up an auto-completion system to provide a better UX when necessary.

You can set-up a custom `Suggester` function, which receives the current input as the only argument and should return a vector of strings, all of the suggested values.

The user is then able to select one of them by moving up and down the list, possibly further modifying a selected suggestion.

In the demo on the top of this README, you can see this behavior in action with the _payee_ prompt.

### Default behaviors

Default behaviors for each one of `Text` configuration options:

- The input formatter just echoes back the given input.
- No validators are called, accepting any sort of input including empty ones.
- No default values or help messages.
- No auto-completion features set-up.
- Prompt messages are always required when instantiating via `new()`.

## DateSelect

![Animated GIF making a demonstration of a DateSelect prompt created with this library. You can replay this recording in your terminal with asciinema play command using the file ./assets/date_complete.cast](./assets/date_complete.gif)

```rust
let date = DateSelect::new("When do you want to travel?")
    .with_default(chrono::NaiveDate::from_ymd(2021, 8, 1))
    .with_min_date(chrono::NaiveDate::from_ymd(2021, 8, 1)) 
    .with_max_date(chrono::NaiveDate::from_ymd(2021, 12, 31))
    .with_week_start(chrono::Weekday::Mon)
    .with_help_message("Possible flights will be displayed according to the selected date")
    .prompt();

match date {
    Ok(_) => println!("No flights available for this date."),
    Err(_) => println!("There was an error in the system."),
}
```

DateSelect prompts allows user to select a date (time not supported) from an interactive calendar. This prompt is only available when including the `date` feature in the dependency, as it brings an additional module (`chrono`) in your dependency tree.

DateSelect prompts provide several options of configuration:

- **Prompt message**: Required when creating the prompt.
- **Default value**: Default value selected when the calendar is displayed and the one select if the user submits without any previous actions. Current date by default.
- **Help message**: Message displayed at the line below the prompt.
- **Formatter**: Custom formatter in case you need to pre-process the user input before showing it as the final answer.
  - Formats to "Month Day, Year" by default.
- **Validators**: Custom validators to the user's selected date, displaying an error message if the date does not pass the requirements.
- **Week start**: Which day of the week should be displayed in the first column of the calendar.
  - Sunday by default.
- **Min and max date**: Inclusive boundaries of allowed dates in the interactive calendar.
  - None by default.
- **Vim mode**: Allows the user to navigate using hjkl keys, off by default.

## Select

![Animated GIF making a demonstration of a simple Select prompt created with this library. You can replay this recording in your terminal with asciinema play command using the file ./assets/select.cast](./assets/select.gif)

```rust
let options = vec!["Banana", "Apple", "Strawberry", "Grapes",
    "Lemon", "Tangerine", "Watermelon", "Orange", "Pear", "Avocado", "Pineapple",
];

let ans = Select::new("What's your favorite fruit?", &options).prompt();

match ans {
    Ok(choice) => println!("I also love {}!", choice.value),
    Err(_) => println!("There was an error, please try again"),
}
```

The `Select` prompt is created with a prompt message and a non-empty list of options. It is suitable for when you need the user to select one option among many.

The `Select` prompt does not support custom validators because of the nature of the prompt. A submission always selects exactly one of the options. If this option was not supposed to be selected or is invalid in some way, it probably should not be included in the options list.

The options are paginated in order to provide a smooth experience to the user, with the default page size being 7. The user can move from the options and the pages will be updated accordingly, including moving from the last to the first options (or vice-versa).

The user can submit their choice by pressing either space or enter.

Like all others, this prompt also allows you to customize several aspects of it:

- **Prompt message**: Required when creating the prompt.
- **Options list**: Options displayed to the user.
- **Starting cursor**: Index of the cursor when the prompt is first rendered. Default is 0 (first option).
- **Help message**: Message displayed at the line below the prompt.
- **Formatter**: Custom formatter in case you need to pre-process the user input before showing it as the final answer.
  - Prints the selected option string value by default.
- **Vim mode**: Allows the user to navigate using hjkl keys, off by default.
- **Page size**: Number of options displayed at once, 7 by default.
- **Filter function**: Function that defines if an option is displayed or not based on the current filter input.

## MultiSelect

![Animated GIF making a demonstration of a simple MultiSelect prompt created with this library. You can replay this recording in your terminal with asciinema play command using the file ./assets/multiselect.cast](./assets/multiselect.gif)

The source is too long, find it [here](./examples/multiselect.rs).

The `MultiSelect` prompt is created with a prompt message and a non-empty list of options. It is suitable for when you need the user to select many options (including none if applicable) among a list of them.

The options are paginated in order to provide a smooth experience to the user, with the default page size being 7. The user can move from the options and the pages will be updated accordingly, including moving from the last to the first options (or vice-versa).

The user can pick the current selection by pressing space, cleaning all selections by pressing the left arrow and selecting all options by pressing the right arrow.

Like all others, this prompt also allows you to customize several aspects of it:

- **Prompt message**: Required when creating the prompt.
- **Options list**: Options displayed to the user.
- **Default selections**: Options that are selected by default when the prompt is first rendered. The user can unselect them.
- **Starting cursor**: Index of the cursor when the prompt is first rendered. Default is 0 (first option).
- **Help message**: Message displayed at the line below the prompt.
- **Formatter**: Custom formatter in case you need to pre-process the user input before showing it as the final answer.
  - Prints the selected options string value, joined using a comma as the separator, by default.
- **Validator**: Custom validator to make sure a given submitted input pass the specified requirements, e.g. not allowing 0 selected options or limiting the number of options that the user is allowed to select.
  - No validators are on by default.
- **Vim mode**: Allows the user to navigate using hjkl keys, off by default.
- **Page size**: Number of options displayed at once, 7 by default.
- **Filter function**: Function that defines if an option is displayed or not based on the current filter input.
- **Keep filter flag**: Whether the current filter input should be cleared or not after a selection is made. Defaults to true.

## Password

![Animated GIF making a demonstration of a simple Password prompt created with this library. You can replay this recording in your terminal with asciinema play command using the file ./assets/password_simple.cast](./assets/password_simple.gif)

```rust
let name = Password::new("Encryption key:").prompt();

match name {
    Ok(_) => println!("This doesn't look like a key."),
    Err(_) => println!("An error happened when asking for your key, try again later."),
}
```

`Password` prompts are basically a less-featured version of Text prompts. Differences being:

- User input is not echoed back to the terminal while typing.
- User input is formatted to "\*\*\*\*\*\*\*\*" (eight star characters) by default.
- No support for default values.
- No support for auto-completion, obviously.

However, it is still possible to customize error messages, formatters and validators.

## CustomType

![Animated GIF making a demonstration of a simple CustomType prompt created with this library. You can replay this recording in your terminal with asciinema play command using the file ./assets/custom_type.cast](./assets/custom_type.gif)

```rust
let amount = CustomType::<f64>::new("How much do you want to donate?")
    .with_formatter(&|i| format!("${:.2}", i))
    .with_error_message("Please type a valid number")
    .with_help_message("Type the amount in US dollars using a decimal point as a separator")
    .prompt();

match amount {
    Ok(_) => println!("Thanks a lot for donating that much money!"),
    Err(_) => println!("We could not process your donation"),
}
```

`CustomType` prompts are generic prompts suitable for when you need to parse the user input into a specific type, for example an `f64` or a `rust_decimal`, maybe even an `uuid`.

This prompt has all of the validation, parsing and error handling features built-in to reduce as much boilerplaste as possible from your prompts. Its defaults are necessarily very simple in order to cover a large range of generic cases, for example a "Invalid input" error message.

You can customize as many aspects of this prompt as you like: prompt message, help message, default value, value parser and value formatter.

**Behavior**

When initializing this prompt via the `new()` method, some constraints on the return type `T` are added to make sure we can apply a default parser and formatter to the prompt.

The default parser calls the [`str.parse`](https://doc.rust-lang.org/stable/std/primitive.str.html#method.parse) method, which means that `T` must implement the `FromStr` trait. When the parsing fails for any reason, a default error message "Invalid input" is displayed to the user.

After the user submits, the prompt handler tries to parse the input into the expected type. If the operation succeeds, the value is returned to the prompt caller. If it fails, the message defined in `error_message` is displayed to the user.

The default formatter simply calls `to_string()` on the parsed value, which means that `T` must implement the `ToString` trait, which normally happens implicitly when you implement the `Display` trait.

If your type `T` does not satisfy these constraints, you can always manually instantiate the entire struct yourself like this:

```rust
let amount_prompt: CustomType<chrono::NaiveDate> = CustomType {
    message: "When will you travel?",
    formatter: &|val| val.format("%d/%m/%Y").to_string(),
    default: None,
    error_message: "Please type a valid date in the expected format.".into(),
    help_message: "The date should be in the dd/mm/yyyy format.".into(),
    parser: &|i| match chrono::NaiveDate::parse_from_str(i, "%d/%m/%Y") {
        Ok(val) => Ok(val),
        Err(_) => Err(()),
    },
};
```

## Confirm

![Animated GIF making a demonstration of a simple Confirm prompt created with this library. You can replay this recording in your terminal with asciinema play command using the file ./assets/confirm_simple.cast](./assets/confirm_simple.gif)

```rust
let ans = Confirm::new("Do you live in Brazil?")
    .with_default(false)
    .with_help_message("This data is stored for good reasons")
    .prompt();

match ans {
    Ok(true) => println!("That's awesome!"),
    Ok(false) => println!("That's too bad, I've heard great things about it."),
    Err(_) => println!("Error with questionnaire, try again later"),
}
```

`Confirm` is a prompt to ask the user for simple yes/no questions, commonly known by asking the user displaying the `(y/n)` text.

This prompt is basically a wrapper around the behavior of [`CustomType`] prompts, providing a sensible set of defaults to ask for simple `true/false` questions, such as confirming an action.

Default values are formatted with the given value in uppercase, e.g. `(Y/n)` or `(y/N)`. The `bool` parser accepts by default only the following inputs (case-insensitive): `y`, `n`, `yes` and `no`. If the user input does not match any of them, the following error message is displayed by default:
- `# Invalid answer, try typing 'y' for yes or 'n' for no`.

Finally, once the answer is submitted, `Confirm` prompts display the bool value formatted as either "Yes", if a `true` value was parsed, or "No" otherwise.

The Confirm prompt does not support custom validators because of the nature of the prompt. The user input is always parsed to true or false. If one of the two alternatives is invalid, a Confirm prompt that only allows yes or no answers does not make a lot of sense to me, but if someone provides a clear use-case I will reconsider.

Confirm prompts provide several options of configuration:

- **Prompt message**: Required when creating the prompt.
- **Default value**: Default value returned when the user submits an empty response.
- **Help message**: Message displayed at the line below the prompt.
- **Formatter**: Custom formatter in case you need to pre-process the user input before showing it as the final answer.
  - Formats `true` to "Yes" and `false` to "No", by default.
- **Parser**: Custom parser for user inputs.
  - The default `bool` parser returns `true` if the input is either `"y"` or `"yes"`, in a case-insensitive comparison. Similarly, the parser returns `false` if the input is either `"n"` or `"no"`.
- **Default value formatter**: Function that formats how the default value is displayed to the user.
  - By default, displays "y/n" with the default value capitalized, e.g. "y/N".
- **Error message**: Error message to display when a value could not be parsed from the input.
  - Set to "Invalid answer, try typing 'y' for yes or 'n' for no" by default.

[`Text`]: #Text
[`DateSelect`]: #DateSelect
[`Select`]: #Select
[`MultiSelect`]: #MultiSelect
[`Confirm`]: #Confirm
[`CustomType`]: #CustomType
[`Password`]: #Password