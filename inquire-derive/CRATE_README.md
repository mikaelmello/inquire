[![Latest Version]][crates.io] ![Build status] ![Supported platforms] ![License]

[crates.io]: https://crates.io/crates/inquire-derive
[latest version]: https://img.shields.io/crates/v/inquire-derive.svg
[build status]: https://github.com/mikaelmello/inquire/actions/workflows/build.yml/badge.svg
[supported platforms]: https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-success
[license]: https://img.shields.io/crates/l/inquire-derive.svg

---

# inquire-derive

Derive macros for the [inquire](https://crates.io/crates/inquire) crate.

## Usage

Put these lines in your `Cargo.toml`, under `[dependencies]`.

```toml
inquire = "0.9.1"
inquire-derive = "0.9.1"
```

Then use the `Selectable` derive macro on your enums:

```rust
use inquire_derive::Selectable;
use std::fmt::{Display, Formatter};

#[derive(Debug, Copy, Clone, Selectable)]
enum Color {
    Red,
    Green,
    Blue,
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Now you can use:
let color = Color::select("Choose a color:").prompt()?;
let colors = Color::multi_select("Choose colors:").prompt()?;
```

## Features

The `Selectable` derive macro generates two methods for your enum:

- `select(msg: &str)` - Returns a `Select` builder for single selection
- `multi_select(msg: &str)` - Returns a `MultiSelect` builder for multiple selection

Both methods return builders that can be customized before calling `.prompt()`:

```rust
let color = Color::select("Choose a color:")
    .with_help_message("Use arrow keys to navigate")
    .with_page_size(5)
    .prompt()?;

let colors = Color::multi_select("Choose colors:")
    .with_default(&[0, 1]) // Pre-select first two options
    .with_help_message("Space to select, Enter to confirm")
    .prompt()?;
```

## Examples

Run the examples to see the derive macro in action:

```bash
cargo run --example enum_select_derive
cargo run --example enum_comprehensive
```

## Requirements

Your enum must implement:
- `Display` - for showing options to the user
- `Debug` - required by inquire
- `Copy` and `Clone` - for efficient handling
- Be `'static` - for the generated code
