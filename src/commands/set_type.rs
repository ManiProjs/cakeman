use anyhow::{Result, anyhow};
use std::fs;

pub fn execute(package_type: String) -> Result<()> {
    if package_type != "library" && package_type != "binary" {
        return Err(anyhow!("Type must be library or binary"));
    }

    let content = fs::read_to_string("Cake.toml")?;

    let mut doc = content.parse::<toml_edit::DocumentMut>()?;

    doc["package"]["type"] = toml_edit::value(package_type);

    fs::write("Cake.toml", doc.to_string())?;

    println!("Updated package type");

    Ok(())
}
