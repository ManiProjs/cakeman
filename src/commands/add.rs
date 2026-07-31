use anyhow::Result;
use std::path::Path;

use crate::{lockfile::Lockfile, manifest::Manifest, resolver::Resolver, terminal};

pub fn execute(name: String, version: Option<String>) -> Result<()> {
    let manifest_path = Path::new("Cake.toml");

    if !manifest_path.exists() {
        terminal::error("No Cake.toml found. Run `cakeman init` first.");

        return Ok(());
    }

    let mut manifest = Manifest::load(manifest_path)?;

    if manifest.dependencies.contains_key(&name) {
        terminal::warn(&format!("{} is already a dependency", name));

        return Ok(());
    }

    let requested_version = version.unwrap_or_else(|| "latest".to_string());

    terminal::info(&format!("Adding {}...", name));

    // Add requested dependency to Cake.toml
    manifest
        .dependencies
        .insert(name.clone(), requested_version.clone());

    manifest.save(manifest_path)?;

    // Resolve dependency tree
    let mut lockfile = Lockfile::load()?;

    let mut resolver = Resolver::new();

    resolver.resolve(&name, Some(&requested_version), &mut lockfile)?;

    lockfile.save()?;

    terminal::success(&format!("Added {}", name));

    Ok(())
}
