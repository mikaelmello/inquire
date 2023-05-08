//! Path picker example 
use inquire::{
    PathSelect, PathSelectionMode, 
};

fn main() {
    let start_path = std::env::current_dir().expect("must get current dir");

    let extension_filter = "toml";
    let selection_mode = PathSelectionMode::File(Some(extension_filter));

    let ans = PathSelect::new(
        &format!("pick an .{extension_filter} file"),
        Some(start_path)
    )
        .with_selection_mode(selection_mode)
        .prompt();

    match ans {
        Ok(f) => println!("You picked {f:#?}"),
        Err(err) => eprintln!("Your choices were wretched"),
    }
}