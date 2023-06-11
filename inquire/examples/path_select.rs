//! Path picker example
use inquire::{
    PathSelect,
    PathSelectionMode
};

fn main() {
    let start_path = std::env::current_dir().expect("must get current dir");

    let toml_extension = "toml";
    let rs_extension = "rs";
    let selection_mode = PathSelectionMode::Multiple(vec![
        PathSelectionMode::File(Some(toml_extension)),
        PathSelectionMode::File(Some(rs_extension)),
    ]);

    let ans = PathSelect::new(
        &format!("pick an .{toml_extension} or .{rs_extension} file"),
        Some(start_path),
    )
    .with_select_multiple(true)
    .with_selection_mode(selection_mode)
    .prompt();

    match ans {
        Ok(entries) => {
            let l = entries.len();
            println!(
                "\nYou picked {l} items{}",
                (!entries.is_empty())
                    .then(|| {
                        entries
                            .iter()
                            .enumerate()
                            .map(|(i, entry)| format!("\n{i}: {entry}"))
                            .collect::<String>()
                    })
                    .unwrap_or_default()
            )
        }
        Err(err) => eprintln!("Your choices were wretched:\n{err:#?}"),
    }
}
