use inquire::{CustomUserError, Text};

fn main() {
    let ans = Text::new("Profile picture:")
        .with_suggester(&suggest_file_paths)
        .with_completer(&complete_file_path)
        .prompt();

    match ans {
        Ok(path) => println!("Path: {}", path),
        Err(error) => println!("Error with questionnaire, try again later: {:?}", error),
    }
}

fn suggest_file_paths(input: &str) -> Result<Vec<String>, CustomUserError> {
    Ok(list_paths(input)?)
}

fn complete_file_path(input: &str) -> Result<Option<String>, CustomUserError> {
    // Implementation from https://rosettacode.org/wiki/Longest_common_prefix#Rust
    fn longest_common_prefix<T: AsRef<[u8]>>(list: &[T]) -> Option<Vec<u8>> {
        if list.is_empty() {
            return None;
        }
        let mut ret = Vec::new();
        let mut i = 0;
        loop {
            let mut c = None;
            for word in list {
                let word = word.as_ref();
                if i == word.len() {
                    return Some(ret);
                }
                match c {
                    None => {
                        c = Some(word[i]);
                    }
                    Some(letter) if letter != word[i] => return Some(ret),
                    _ => continue,
                }
            }
            if let Some(letter) = c {
                ret.push(letter);
            }
            i += 1;
        }
    }

    Ok(longest_common_prefix(&list_paths(input)?)
        .map(|bytes| String::from_utf8_lossy(&bytes).to_string()))
}

fn list_paths(root: &str) -> std::io::Result<Vec<String>> {
    let mut suggestions = vec![];

    let mut input_path = std::path::PathBuf::from(root);
    if let Some(parent) = input_path.parent() {
        if !input_path.exists() || !input_path.is_dir() || !root.ends_with('/') {
            input_path = parent.to_path_buf();
        }
    }
    if root.is_empty() {
        input_path = std::env::current_dir()?;
    }
    if !input_path.exists() {
        return Ok(vec![]);
    }

    for entry in std::fs::read_dir(input_path)? {
        let path = entry?.path();
        let path_str = path.to_string_lossy();

        if path_str.starts_with(root) {
            let path = if path.is_dir() {
                format!("{}/", path_str)
            } else {
                path_str.to_string()
            };
            suggestions.push(path);
        }
    }

    Ok(suggestions)
}
