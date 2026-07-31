use anyhow::{Result, anyhow};
use std::fs;
use std::path::Path;

use crate::terminal;

pub fn execute(cake_name: String) -> Result<()> {
    let default_manifest = Path::new("Cake.toml");
    let filename = &format!("{}.toml", cake_name);
    let named_manifest = Path::new(filename);

    if default_manifest.exists() {
        fs::rename(default_manifest, named_manifest)?;

        terminal::success(&format!("Renamed Cake.cman to {}.cman", cake_name));

        return Ok(());
    }

    if named_manifest.exists() {
        fs::rename(named_manifest, default_manifest)?;

        terminal::success("Renamed project manifest to Cake.cman");

        return Ok(());
    }

    Err(anyhow!("No Cakeman manifest found"))
}
