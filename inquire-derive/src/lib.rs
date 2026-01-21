mod codegen;
mod utils;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// Derive macro for making enums selectable in inquire prompts.
///
/// This macro generates `select()` and `multi_select()` methods for enums,
/// allowing them to be used directly with inquire's Select and MultiSelect prompts.
/// The methods return the prompt builders, allowing for further customization.
///
/// ## Display Implementation
///
/// The macro automatically implements `Display` for enums **if and only if** any variant
/// has a doc comment. If doc comments are present on some variants:
/// - Variants with doc comments use the comment text as their display value
/// - Variants without doc comments fall back to their variant name
///
/// If no variants have doc comments, you must manually implement `Display` before
/// using the `Selectable` macro, otherwise compilation will fail.
///
/// The enum must also implement `Debug`, `Copy`, `Clone`, and be `'static`.
///
/// ## Examples
///
/// ### With Doc Comments (Auto Display)
///
/// ```ignore
/// use inquire_derive::Selectable;
///
/// #[derive(Debug, Copy, Clone, Selectable)]
/// enum Color {
///     /// Bright red color
///     Red,
///     /// Vibrant green color  
///     Green,
///     /// Deep blue color
///     Blue,
///     // This variant will display as "Yellow" (variant name)
///     Yellow,
/// }
///
/// // Usage:
/// let color: Result<Color, inquire::InquireError> = Color::select("Choose a color:").prompt();
/// ```
///
/// ### Without Doc Comments (Manual Display)
///
/// ```ignore
/// use inquire_derive::Selectable;
/// use std::fmt::{Display, Formatter, Result as FmtResult};
///
/// #[derive(Debug, Copy, Clone, Selectable)]
/// enum Priority {
///     Low,
///     Medium,
///     High,
/// }
///
/// impl Display for Priority {
///     fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
///         let display_str: &str = match self {
///             Priority::Low => "Low Priority",
///             Priority::Medium => "Medium Priority",
///             Priority::High => "High Priority",
///         };
///         write!(f, "{}", display_str)
///     }
/// }
///
/// // Usage:
/// let priority: Result<Priority, inquire::InquireError> = Priority::select("Select priority:").prompt();
/// ```
///
/// ### With Customization
///
/// ```ignore
/// let color: Result<Color, inquire::InquireError> = Color::select("Choose a color:")
///     .with_help_message("Use arrow keys to navigate")
///     .with_page_size(5)
///     .prompt();
///
/// let colors: Result<Vec<Color>, inquire::InquireError> = Color::multi_select("Choose multiple colors:")
///     .with_default(&[Color::Red, Color::Blue])
///     .prompt();
/// ```
#[proc_macro_derive(Selectable, attributes(desc))]
pub fn derive_selectable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    codegen::generate_selectable_impl(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selectable_derive_exists() {
        // This test ensures the derive macro function exists and can be called
        // We can't actually test the TokenStream output without a full compile,
        // but we can test that the function signature is correct
        let _derive_fn = derive_selectable;
    }
}
