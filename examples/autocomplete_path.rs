use std::fs::DirEntry;

use inquire::{
    autocompletion::{AutoComplete, Replacement},
    CustomUserError, Text,
};

fn main() {
    let ans = Text::new("Profile picture:")
        .with_auto_completion(FilePathCompleter::default())
        .prompt();

    match ans {
        Ok(path) => println!("Path: {}", path),
        Err(error) => println!("Error with questionnaire, try again later: {:?}", error),
    }
}

#[derive(Clone, Default)]
pub struct FilePathCompleter {
    paths: Vec<String>,
}

impl AutoComplete for FilePathCompleter {
    fn update_input(&mut self, input: &str) -> Result<(), CustomUserError> {
        let root = input;
        self.paths.clear();

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
            return Ok(());
        }

        let mut entries: Vec<DirEntry> =
            std::fs::read_dir(input_path)?.collect::<Result<Vec<_>, _>>()?;

        let mut idx = 0;
        let limit = 15;

        while idx < entries.len() && self.paths.len() < limit {
            let entry = entries.get(idx).unwrap();

            let path = entry.path();
            let path_str = path.to_string_lossy();

            if path_str.starts_with(root) {
                let path = if path.is_dir() {
                    let mut subentries: Vec<DirEntry> =
                        std::fs::read_dir(path.clone())?.collect::<Result<Vec<_>, _>>()?;
                    entries.append(&mut subentries);

                    format!("{}/", path_str)
                } else {
                    path_str.to_string()
                };
                self.paths.push(path);
            }

            idx = idx.saturating_add(1);
        }

        self.paths.sort();

        Ok(())
    }

    fn get_suggestions(&self) -> Result<Vec<String>, CustomUserError> {
        Ok(self.paths.clone())
    }

    fn get_completion(
        &self,
        selected_suggestion: Option<(usize, &str)>,
    ) -> Result<inquire::autocompletion::Replacement, CustomUserError> {
        let completion = match selected_suggestion {
            None => {
                let lcp = longest_common_prefix(&self.paths)
                    .map(|bytes| String::from_utf8_lossy(&bytes).to_string());

                match lcp {
                    Some(lcp) => Replacement::Some(lcp),
                    None => Replacement::None,
                }
            }
            Some(suggestion) => Replacement::Some(suggestion.1.to_owned()),
        };

        Ok(completion)
    }
}

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
