//! Type aliases and default implementations for parsers called in prompts
//! that need to parse user input, such as [Confirm](crate::Confirm) or
//! [`CustomType`](crate::CustomType).
//!
//! Parsers receive the user input to a given prompt and return either
//! a successful result ([Ok]) containing the parsed value or an empty [Err]
//! if a value could not be parsed.

/// Type alias for parsers used in [Confirm](crate::Confirm) prompts.
///
/// [`BoolParser`]s receive the user input to a given prompt and return either
/// a successful result ([Ok]) containing the parsed `bool` or an empty [Err]
/// if a value could not be parsed.
///
/// # Examples
///
/// ```
/// use inquire::parser::BoolParser;
///
/// let parser: BoolParser = &|ans| match ans {
///     "si" => Ok(true),
///     "no" => Ok(false),
///     _ => Err(()),
/// };
/// assert_eq!(Ok(true), parser("si"));
/// assert_eq!(Ok(false), parser("no"));
/// assert_eq!(Err(()), parser("yes"));
/// assert_eq!(Err(()), parser("não"));
/// ```
pub type BoolParser<'a> = &'a dyn Fn(&str) -> Result<bool, ()>;

/// Type alias for parsers used in [Confirm](crate::Confirm) prompts.
///
/// [`CustomTypeParser`]s receive the user input to a given prompt and return either
/// a successful result ([Ok]) containing the parsed `bool` or an empty [Err]
/// if a value could not be parsed.
///
/// # Examples
///
/// ```
/// use inquire::parser::CustomTypeParser;
///
/// let parser: CustomTypeParser<bool> = &|val| match val {
///     "si" => Ok(true),
///     "no" => Ok(false),
///     _ => Err(()),
/// };
/// assert_eq!(Ok(true), parser("si"));
/// assert_eq!(Ok(false), parser("no"));
/// assert_eq!(Err(()), parser("yes"));
/// assert_eq!(Err(()), parser("não"));
/// ```
pub type CustomTypeParser<'a, T> = &'a dyn Fn(&str) -> Result<T, ()>;

/// Bool formatter used  by default in [Confirm](crate::Confirm) prompts.
pub const DEFAULT_BOOL_PARSER: BoolParser<'_> = &|ans| {
    if ans.len() > 3 {
        return Err(());
    }

    let ans = ans.to_lowercase();

    match ans.as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        _ => Err(()),
    }
};

#[macro_export]
#[cfg(feature = "macros")]
/// Built-in parser creator that checks whether the answer is able to be successfully
/// parsed to a given type, such as `f64`.
/// [The given type must implement the FromStr trait.](https://doc.rust-lang.org/stable/std/primitive.str.html#method.parse)
///
/// # Arguments
///
/// * `$type` - Target type of the parsing operation.
///
/// # Examples
///
/// ```
/// use inquire::parse_type;
/// use inquire::parser::CustomTypeParser;
///
/// let parser: CustomTypeParser<f64> = parse_type!(f64);
/// assert_eq!(Ok(32.44f64), parser("32.44"));
/// assert_eq!(Ok(11e15f64), parser("11e15"));
/// assert_eq!(Err(()), parser("32f"));
/// assert_eq!(Err(()), parser("11^2"));
/// ```
macro_rules! parse_type {
    ($type:ty) => {{
        &|a| a.parse::<$type>().map_err(|_| ())
    }};
}
