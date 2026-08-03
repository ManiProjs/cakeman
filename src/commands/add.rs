use anyhow::{Result, anyhow};
use std::path::Path;

use crate::{lockfile::Lockfile, manifest::Manifest, registry, resolver::Resolver, terminal};

pub async fn execute(names: Vec<String>, version: Option<String>) -> Result<()> {
    let manifest_path = Path::new("Cake.toml");

    if !manifest_path.exists() {
        terminal::error("No Cake.toml found. Run `cakeman init` first.");

        return Ok(());
    }

    let mut manifest = Manifest::load(manifest_path)?;

    let mut lockfile = Lockfile::load()?;

    let mut resolver = Resolver::new();

    for name in names {
        if manifest.dependencies.contains_key(&name) {
            terminal::warn(&format!("{} is already a dependency", name));
            continue;
        }

        terminal::info(&format!("Searching {} in registry...", name));

        let content = registry::get_cake_manifest(&name)
            .await
            .map_err(|_| anyhow!("Package '{}' was not found in registry", name))?;

        let registry_manifest: toml::Value = toml::from_str(&content)?;

        let latest_version = registry_manifest
            .get("package")
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Invalid registry manifest for {}", name))?
            .to_string();

        let resolved_version = match version.as_deref() {
            Some(v) if v != "latest" => v.to_string(),
            _ => latest_version,
        };

        terminal::info(&format!("Found {} {}", name, resolved_version));

        manifest
            .dependencies
            .insert(name.clone(), resolved_version.clone());

        resolver
            .resolve(&name, Some(&resolved_version), &mut lockfile)
            .await?;

        terminal::success(&format!("Added {}", name));
    }

    manifest.save(manifest_path)?;

    lockfile.save()?;

    Ok(())
}
