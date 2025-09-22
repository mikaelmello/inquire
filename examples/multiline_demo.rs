use inquire::{MultiSelect, Select};

fn main() {
    println!("=== Multiline Select Demo ===");
    
    let people = vec![
        "Mr. Anderson\nSesame st. 10, NY\n90210",
        "Mrs. Anderson\nSesame st. 10, NY\n90210", 
        "Jr. Anderson\nSesame st. 10, NY\n90210",
    ];

    println!("\n--- Select Demo ---");
    let select_result = Select::new("Choose a person:", people.clone()).prompt();
    match select_result {
        Ok(choice) => println!("You selected:\n{}", choice),
        Err(_) => println!("Selection was cancelled"),
    }

    println!("\n--- MultiSelect Demo ---");
    let multiselect_result = MultiSelect::new("Choose people:", people).prompt();
    match multiselect_result {
        Ok(choices) => {
            println!("You selected {} people:", choices.len());
            for choice in choices {
                println!("- {}", choice);
            }
        },
        Err(_) => println!("Selection was cancelled"),
    }
}