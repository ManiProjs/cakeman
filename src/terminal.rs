use owo_colors::OwoColorize;
use std::io::{self, Write};

pub fn info(message: &str) {
    println!("{} {}", "→".cyan().bold(), message);
}

pub fn success(message: &str) {
    println!("{} {}", "✓".green().bold(), message);
}

pub fn warn(message: &str) {
    println!("{} {}", "!".yellow().bold(), message);
}

pub fn error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message);
}

pub fn hint(message: &str) {
    println!("{} {}", "💡".cyan(), message);
}

pub fn debug(message: &str) {
    println!("{} {}", "DEBUG".magenta().bold(), message.dimmed());
}

pub fn title(message: &str) {
    println!("\n{}\n", message.bold().underline());
}

pub fn ask(prompt: &str) -> String {
    print!("{} ", prompt.blue().bold());

    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    input.trim().to_string()
}

pub fn banner() {
    println!(
        "{}",
        r#"
   ____      _                                  _
  / ___|__ _| | _____ _ __ ___   __ _ _ __      | |
 | |   / _` | |/ / _ \ '_ ` _ \ / _` | '_ \  _  | |
 | |__| (_| |   <  __/ | | | | | (_| | | | || |_| |
  \____\__,_|_|\_\___|_| |_| |_|\__,_|_| |_(_)___/

        C/C++ package manager
"#
        .bright_yellow()
    );
}

pub fn divider() {
    println!("{}", "────────────────────────────────────────".dimmed());
}
