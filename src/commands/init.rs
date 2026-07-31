use anyhow::Result;

pub fn execute(name: Option<String>) -> Result<()> {
    match name {
        Some(name) => println!("Creating project: {}", name),
        None => println!("Creating project in current directory"),
    }

    // TODO:
    // - write a Cake.cman starter template

    Ok(())
}
