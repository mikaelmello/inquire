use std::fmt::{Display, Formatter};

use inquire::error::InquireResult;
use inquire_derive::Selectable;

fn main() -> InquireResult<()> {
    println!("=== Currency Selection Example ===\n");

    // Example using single select
    println!("1. Select your primary currency:");
    let currency: Currency = Currency::select("Currency:").prompt()?;

    match currency {
        Currency::BRL | Currency::USD | Currency::CAD | Currency::EUR | Currency::GBP => {
            bank_transfer(&currency);
        }
        Currency::BTC | Currency::LTC => crypto_transfer(&currency),
    }

    // Example using multi_select with customization
    println!("\n2. Select multiple currencies for comparison:");
    let selected_currencies: Vec<Currency> = Currency::multi_select("Select multiple currencies:")
        .with_help_message("Use space to select, enter to confirm")
        .with_page_size(5)
        .prompt()?;

    if selected_currencies.is_empty() {
        println!("No currencies selected for comparison.");
    } else {
        println!(
            "You selected {} currencies for comparison:",
            selected_currencies.len()
        );
        for currency in selected_currencies {
            println!("  - {currency}");
        }
    }

    Ok(())
}

fn bank_transfer(currency: &Currency) {
    println!("Setting up bank transfer for {currency}...");
    println!("Bank transfer configured!");
}

fn crypto_transfer(currency: &Currency) {
    println!("Setting up crypto wallet for {currency}...");
    println!("Crypto wallet configured!");
}

#[derive(Debug, Copy, Clone, Selectable)]
#[allow(clippy::upper_case_acronyms)]
enum Currency {
    BRL,
    USD,
    CAD,
    EUR,
    GBP,
    BTC,
    LTC,
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Currency::BRL => write!(f, "🇧🇷 Brazilian Real (BRL)"),
            Currency::USD => write!(f, "🇺🇸 US Dollar (USD)"),
            Currency::CAD => write!(f, "🇨🇦 Canadian Dollar (CAD)"),
            Currency::EUR => write!(f, "🇪🇺 Euro (EUR)"),
            Currency::GBP => write!(f, "🇬🇧 British Pound (GBP)"),
            Currency::BTC => write!(f, "₿ Bitcoin (BTC)"),
            Currency::LTC => write!(f, "Ł Litecoin (LTC)"),
        }
    }
}
