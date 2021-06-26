use crate::OptionAnswer;

pub type StringValidator = fn(answer: &str) -> Result<(), String>;

pub type MultiOptionValidator = fn(answer: &[OptionAnswer]) -> Result<(), String>;

#[macro_export]
#[cfg(feature = "builtin_validators")]
macro_rules! required {
    () => {
        $crate::required! {"A response is required."}
    };

    ($message:expr) => {
        |a| match a.is_empty() {
            true => Err(String::from($message)),
            false => Ok(()),
        }
    };
}

#[macro_export]
#[cfg(feature = "builtin_validators")]
macro_rules! max_length {
    ($length:expr) => {
        $crate::max_length! {$length, format!("The length of the response should be at most {}", $length)}
    };

    ($length:expr, $message:expr) => {
        {
            |a| match a.len() {
                _len if _len <= $length => Ok(()),
                _ => Err(String::from($message)),
            }

        }
    };
}

#[macro_export]
#[cfg(feature = "builtin_validators")]
macro_rules! min_length {
    ($length:expr) => {
        $crate::min_length! {$length, format!("The length of the response should be at least {}", $length)}
    };

    ($length:expr, $message:expr) => {
        {
            |a| match a.len() {
                _len if _len >= $length => Ok(()),
                _ => Err(String::from($message)),
            }
        }
    };
}

#[macro_export]
#[cfg(feature = "builtin_validators")]
macro_rules! length {
    ($length:expr) => {
        $crate::length! {$length, format!("The length of the response should be {}", $length)}
    };

    ($length:expr, $message:expr) => {{
        |a| match a.len() {
            _len if _len == $length => Ok(()),
            _ => Err(String::from($message)),
        }
    }};
}

#[macro_export]
#[cfg(feature = "builtin_validators")]
macro_rules! regex {
    ($regex:expr) => {
        $crate::regex! {$regex, format!("The response should match pattern {}", $regex)}
    };

    ($regex:expr,$message:expr) => {{
        |a| {
            use regex::Regex;
            let regex = Regex::new($regex).unwrap();
            if regex.is_match(a) {
                Ok(())
            } else {
                Err(String::from($message))
            }
        }
    }};
}

#[macro_export]
#[cfg(feature = "builtin_validators")]
macro_rules! parse_primitive {
    ($type:ty) => {
        $crate::parse_primitive! {$type, format!("Failure when parsing response to type {}", std::any::type_name::<$type>())}
    };

    ($type:ty, $message:expr) => {{
        |a| match a.parse::<$type>() {
            Ok(_) => Ok(()),
            Err(err) => Err(String::from($message)),
        }
    }};
}
