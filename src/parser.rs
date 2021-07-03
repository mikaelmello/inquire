pub type BoolParser = fn(answer: &str) -> Result<bool, String>;

pub(in crate) const DEFAULT_BOOL_PARSER: BoolParser = |ans| {
    static ERROR_MESSAGE: &str = "Invalid answer, try typing 'y' for yes or 'n' for no";

    if ans.len() > 3 {
        return Err(ERROR_MESSAGE.into());
    }

    let ans = ans.to_lowercase();

    match ans.as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        _ => Err(ERROR_MESSAGE.into()),
    }
};
