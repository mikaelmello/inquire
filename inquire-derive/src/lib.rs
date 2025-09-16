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
/// The enum must implement `Display`, `Debug`, `Copy`, `Clone`, and be `'static`.
///
/// # Example
///
/// ```ignore
/// use inquire_derive::Selectable;
/// use std::fmt::{Display, Formatter};
///
/// #[derive(Debug, Copy, Clone, Selectable)]
/// enum Color {
///     Red,
///     Green,
///     Blue,
/// }
///
/// impl Display for Color {
///     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
///         write!(f, "{:?}", self)
///     }
/// }
///
/// // Usage:
/// // let color = Color::select("Choose a color:").prompt()?;
/// //
/// // With customization:
/// // let color = Color::select("Choose a color:")
/// //     .with_help_message("Use arrow keys to navigate")
/// //     .prompt()?;
/// //
/// // Multi-select:
/// // let colors = Color::multi_select("Choose colors:").prompt()?;
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
