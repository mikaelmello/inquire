use inquire::Confirm;

fn main() {
    println!("Standard confirm prompt (requires Enter after y/n):");
    let standard = Confirm::new("Do you want to continue?")
        .with_default(true)
        .prompt();

    match standard {
        Ok(true) => println!("You chose to continue!\n"),
        Ok(false) => println!("You chose not to continue!\n"),
        Err(_) => println!("Error occurred!\n"),
    }

    println!("Confirm with immediate input (y/n submits immediately):");
    let immediate = Confirm::new("Are you sure you want to delete this file?")
        .with_default(false)
        .with_confirm_on_input(true)
        .prompt();

    match immediate {
        Ok(true) => println!("File will be deleted!"),
        Ok(false) => println!("File deletion cancelled."),
        Err(_) => println!("Error occurred!"),
    }

    println!("\nTip: In the second prompt, just press 'y' or 'n' - no Enter needed!");
}
