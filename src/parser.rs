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
macro_rules! parse_type {
    ($type:ty) => {
        $crate::parse_type! {$type, "Invalid input"}
    };

    ($type:ty,$message:expr) => {{
        &|a| a.parse::<$type>().map_err(|_| ())
    }};
}
