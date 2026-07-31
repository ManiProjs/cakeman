use anyhow::Result;
use std::io::{self, Write};

pub fn execute() -> Result<()> {
    let mut username = String::new();
    let mut password = String::new();

    print!("Username: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut username)?;

    print!("Password: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut password)?;

    let username = username.trim();
    let password = password.trim();

    // TODO:
    // Send credentials to Cakeman registry
    // Receive authentication token
    // Save token locally

    println!("Authenticated as {}", username);

    Ok(())
}
