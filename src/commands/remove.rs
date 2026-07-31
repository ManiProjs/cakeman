use anyhow::Result;
use std::path::Path;

use crate::{manifest::Manifest, terminal};

pub fn execute(name: String) -> Result<()> {
    let manifest_path = Path::new("Cake.toml");

    if !manifest_path.exists() {
        terminal::error("No Cake.toml found. Run `cakeman init` first.");

        return Ok(());
    }

    let mut manifest = Manifest::load(manifest_path)?;

    if !manifest.dependencies.contains_key(&name) {
        terminal::warn(&format!("{} is not a dependency", name));

        return Ok(());
    }

    terminal::info(&format!("Removing {}...", name));

    manifest.dependencies.remove(&name);

    manifest.save(manifest_path)?;

    terminal::success(&format!("Removed {}", name));

    Ok(())
}
