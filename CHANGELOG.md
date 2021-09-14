# Changelog

<!-- next-header -->
## [Unreleased] - ReleaseDate

## [0.2.0] - 2021-09-14

### Features

- Add `inquire::set_global_render_config` method to set a global RenderConfig object to be used as the default one for all prompts created after the call.
- Add [KEY_BINDINGS.md](KEY_BINDINGS.md) to register all key bindings registered by `inquire` prompts.

### Breaking changes

- `RenderConfig` was made `Copy`-able and prompts now contain a `RenderConfig` field where it previously held a `&'a RenderConfig`. Consequently, `with_render_config()` methods now accept a `RenderConfig` argument instead of `&'a RenderConfig`.

## [0.1.0] - 2021-09-14

No changes in this version.

This is a bump to v0.1.0 as per @jam1garner's advice on the Virtual RustConf Discord server.

The library is already featureful enough to warrant a higher version number, bumping us to a minor release while we are still on our path to stabilization.

## [0.0.11] - 2021-09-06

### Features

- Add [`Editor`](https://docs.rs/inquire/0.0.11/inquire/prompts/editor/struct.Editor.html) prompt.
- Add support to use `console` or `termion` as the library to handle terminals while keeping `crossterm` as the default choice.
- Canceling the prompt by pressing `ESC` is now a different behavior than interrupting the prompt by pressing `Ctrl+C`.
  - If the prompt is canceled, the final prompt render indicates to the user that it was canceled via a `<canceled>` text, which is customizable via RenderConfig, and the prompt method returns `Err(InquireError::OperationCanceled)`.
  - If the prompt is interrupted, the only clean-up action done is restoring the cursor position, and the prompt method returns `Err(InquireError::OperationInterrupted)`.
- Add a `prompt_skippable` method for all prompts.
  - This method is intended for flows where the user skipping/cancelling the prompt - by pressing ESC - is considered normal behavior. In this case, it does not return `Err(InquireError::OperationCanceled)`, but `Ok(None)`. Meanwhile, if the user does submit an answer, the method wraps the return type with `Some`.

### Improvements

- Removed need to add `use inquire::validator::InquireLength` when using one of the length-related built-in validators.
- Cursor should not ficker anymore in wrong positions on ~~Windows~~ slower terminals.
- Documentation on the `Color` enum used for render configuration has been improved.

### Fixes

- Fix dependencies the crate had on macros provided by the `builtin_validators` feature, making it now compile when the feature is not turned on.

## [0.0.10] - 2021-08-29

### Features

- Use native terminal cursors in text inputs by default.
- Use native terminal cursor on date prompts when an optional style sheet for the selected cursor token was defined as `None`. The default behavior is still a custom style sheet which highlights the two columns pertaining to a date, instead of using a native cursor which can only highlight one column.
- Respect NO_COLOR environment variable when prompt uses the default render configuration.

### Fixes

- By using a new method to identify the length of the rendered prompt, we avoid possible rendering errors (edge cases) when a string can not render into a single line in the terminal due to a smaller width. Inner calculations could previously predict that the rendered string would fit, by considering that 1 grapheme = 1 column width, but this is not true for e.g. emojis. Now we use unicode_width to fix this behavior.
- Fixed case where Select/MultiSelect prompts were panicking when a user pressed the down arrow on an empty list, which happens when a filter input does not match any options. #30 
- Fixed incorrect indexes on the output of MultiSelect prompts, where the indexes inside a `ListOption` struct were relative to the output instead of the original input vector. #31 
- Fixed case where IO errors due to not finding a tty devices were not being catched and transformed to `InquireError::NotTTY`. #28 

## [0.0.9] - 2021-08-28

### General

- Improve docs on the differences between `raw_prompt` and `prompt` on Select and MultiSelect prompts.
- Bump version of `crossterm` dependency

### Fixes

- Regression introduced on v0.0.8 where select prompts were panicking when user pressed enter while no options were displayed (due to filter input). Tracked by #29 and tests were added for this to not happen again.

## [0.0.8] - 2021-08-25

### Features

- **Password display toggle**: By enabling this option in `Password` prompts via `with_display_toggle_enabled()`, the application user can choose to display the current text input for the password by pressing `Ctrl+R`, and hide it back by pressing the hotkey again. #18 
- **Render mask of password input**: `Password` prompts now support another render mode of the text input. Before, the only behavior was to not render anything about the current password input. Now, if the developer so chooses, they can activate the `Masked` mode where the current text input will be masked with special characters such as `'*'`. #19 
- **PageUp, PageDown, Home and End hotkeys**: PageUp and PageDown are now supported in `Select`, `MultiSelect` and `Text` (suggestions list) prompts, where they go a page up or down in the current list. Also, for `Select` and `MultiSelect` prompts, the Home and End keys were mapped to go to the start or end of the list, respectively. #17 
- **Indication that list is scrollable**: Now option lists, present in `Select`, `MultiSelect` and `Text` prompts, indicate whether there are more options other than the ones currently displayed. Little arrows are displayed at the top or bottom of the list indicating to which positions the user can scroll. #8 
- **Generic option types for Select and MultiSelect prompts**: Now, `Select` and `MultiSelect` prompts accept any type of options as input, allowing developers to pass a vector of owned objects and get back the full object picked by the user. #9 

### Fixes

- **Handling of new-line characters in user-provided strings**: When putting `\n` on strings such as prompt messages, the library previously did not render it very well and did not account for it when cleaning the current prompt. This is fixed and you are free to create multi-line prompts! #15
- **Lines larger than terminal width broke rendering**: When lines that were larger than the terminal width were rendered, it caused the internal line counter (used to clean the prompt) to be off, leading to buggy behavior. This was fixed by retrieving the terminal size at the start of the prompt. #21

## [0.0.7] - 2021-08-20

### Features

- Add possibility to set custom rendering config, allowing users to set:
  - Custom colors
  - Custom prefixes for several prompts
  - Custom checkboxes
- Add "placeholder" feature for prompts with text input

## [0.0.6] - 2021-07-26

- Add [previously non-existing] documentation.
- Add [CustomType](https://github.com/mikaelmello/inquire#customtype) prompt
- Add revamped auto-completion support for Text prompts

## [0.0.5] - 2021-07-19

- All function arguments now accept closures by having their type changed to `&dyn Fn`.
- Improved input UX
  - Cursor added for better editing experience
  - Features/shortcuts added: Ctrl+Left, Ctrl+Right, Home, End, Delete, Ctrl+Delete

## [0.0.4] - 2021-07-14

- Add a custom error enum `InquireError`, improving error handling for library users.
- Improve support for validator functions, allowing the use of closures.
- Change the terminal back-end from termion to crossterm, adding Windows support for this library.

## [0.0.3] - 2021-07-07

- Reduce package footprint
- Add custom parser option to Confirm prompt
- Add DateSelect prompt

<!-- next-url -->
[Unreleased]: https://github.com/mikaelmello/inquire/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/mikaelmello/inquire/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/mikaelmello/inquire/compare/v0.0.11...v0.1.0
[0.0.11]: https://github.com/mikaelmello/inquire/compare/v0.0.10...v0.0.11
[0.0.10]: https://github.com/mikaelmello/inquire/compare/v0.0.9...v0.0.10
[0.0.9]: https://github.com/mikaelmello/inquire/compare/v0.0.8...v0.0.9
[0.0.8]: https://github.com/mikaelmello/inquire/compare/v0.0.7...v0.0.8
[0.0.7]: https://github.com/mikaelmello/inquire/compare/v0.0.6...v0.0.7
[0.0.6]: https://github.com/mikaelmello/inquire/compare/v0.0.5...v0.0.6
[0.0.5]: https://github.com/mikaelmello/inquire/compare/v0.0.4...v0.0.5
[0.0.4]: https://github.com/mikaelmello/inquire/compare/v0.0.3...v0.0.4