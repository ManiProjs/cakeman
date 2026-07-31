pub mod storage;

use anyhow::Result;
use std::io::{self, Write};

pub fn login() -> Result<()> {
    let mut username = String::new();
    let mut token = String::new();

    print!("Username: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut username)?;

    print!("Token: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut token)?;

    storage::save(username.trim(), token.trim())?;

    println!("Authenticated successfully!");

    Ok(())
}

pub fn authenticate() -> Result<()> {
    match storage::load()? {
        Some(auth) => {
            println!("Logged in as {}", auth.username);
            Ok(())
        }

        None => {
            println!("Not authenticated");
            Ok(())
        }
    }
}
