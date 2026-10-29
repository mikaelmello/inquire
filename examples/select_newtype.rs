use std::fmt;

use inquire::{formatter::MultiOptionFormatter, formatter::OptionFormatter, MultiSelect, Select};

/// A product with a price in cents.
struct Product {
    name: &'static str,
    cents: u32,
}

/// Newtype that controls how a `Product` is listed while the prompt is open.
///
/// `Select` and `MultiSelect` render each option through its `Display` implementation, so wrapping
/// the value in a newtype is the way to change how the options look in the list.
struct Listed(Product);

impl fmt::Display for Listed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:<12} ${}.{:02}",
            self.0.name,
            self.0.cents / 100,
            self.0.cents % 100
        )
    }
}

fn products() -> Vec<Listed> {
    vec![
        Listed(Product {
            name: "Notebook",
            cents: 450,
        }),
        Listed(Product {
            name: "Pen",
            cents: 150,
        }),
        Listed(Product {
            name: "Backpack",
            cents: 3999,
        }),
    ]
}

fn main() {
    // The formatter does not change the list above. It only changes how the answer is
    // printed after the prompt is submitted.
    let formatter: OptionFormatter<'_, Listed> =
        &|opt| format!("{} (#{})", opt.value.0.name, opt.index);

    let one = Select::new("Which product do you want?", products())
        .with_formatter(formatter)
        .prompt();

    let multi_formatter: MultiOptionFormatter<'_, Listed> = &|opts| {
        let total: u32 = opts.iter().map(|o| o.value.0.cents).sum();
        format!(
            "{} products, ${}.{:02} in total",
            opts.len(),
            total / 100,
            total % 100
        )
    };

    let many = MultiSelect::new("Which products go in the cart?", products())
        .with_formatter(multi_formatter)
        .prompt();

    match (one, many) {
        (Ok(one), Ok(many)) => println!("Picked {} and {} more", one.0.name, many.len()),
        _ => println!("There was an error, please try again"),
    }
}
