pub type BoolParser<'a> = &'a dyn Fn(&str) -> Result<bool, ()>;
pub type CustomTypeParser<'a, T> = &'a dyn Fn(&str) -> Result<T, ()>;

pub(in crate) const DEFAULT_BOOL_PARSER: BoolParser = &|ans| {
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
#[cfg(feature = "builtin_validators")]
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
