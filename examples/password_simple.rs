use inquire::Password;

fn main() {
    let name = Password::new("RSA Encryption Key:").prompt();

    match name {
        Ok(_) => println!("This doesn't look like a key."),
        Err(_) => println!("An error happened when asking for your key, try again later."),
    }
}
