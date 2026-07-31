use anyhow::Result;
use std::path::Path;

use crate::{manifest::Manifest, registry, terminal};

pub fn execute(name: String, version: Option<String>) -> Result<()> {
    let manifest_path = Path::new("Cake.toml");

    if !manifest_path.exists() {
        terminal::error("No Cake.toml found. Run `cakeman init` first.");

        return Ok(());
    }

    terminal::info(&format!("Checking {} in registry...", name));

    match registry::get_cake_manifest(&name) {
        Ok(_) => {
            terminal::success("Package found in registry");
        }

        Err(err) => {
            terminal::error(&err.to_string());
            return Ok(());
        }
    }

    let mut manifest = Manifest::load(manifest_path)?;

    if manifest.dependencies.contains_key(&name) {
        terminal::warn(&format!("{} is already installed", name));

        return Ok(());
    }

    let version = version.unwrap_or_else(|| "latest".to_string());

    manifest.dependencies.insert(name.clone(), version.clone());

    manifest.save(manifest_path)?;

    terminal::success(&format!("Added {} {}", name, version));

    Ok(())
}
