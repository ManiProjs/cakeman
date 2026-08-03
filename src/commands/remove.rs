use anyhow::Result;
use std::path::Path;

use crate::{lockfile::Lockfile, manifest::Manifest, terminal};

pub fn execute(names: Vec<String>) -> Result<()> {
    let manifest_path = Path::new("Cake.toml");

    if !manifest_path.exists() {
        terminal::error("No Cake.toml found. Run `cakeman init` first.");

        return Ok(());
    }

    let mut manifest = Manifest::load(manifest_path)?;

    let mut lockfile = Lockfile::load()?;

    for name in names {
        if !manifest.dependencies.contains_key(&name) {
            terminal::warn(&format!("{} is not a dependency", name));
            continue;
        }

        terminal::info(&format!("Removing {}...", name));

        manifest.dependencies.remove(&name);

        lockfile.remove_package(&name);

        terminal::success(&format!("Removed {}", name));
    }

    manifest.save(manifest_path)?;

    lockfile.save()?;

    Ok(())
}
