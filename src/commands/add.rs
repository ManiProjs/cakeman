use anyhow::{Result, anyhow};
use std::path::Path;

use crate::{
    lockfile::{LockedPackage, Lockfile},
    manifest::Manifest,
    registry, terminal,
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CakeRegistryManifest {
    package: Package,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    version: String,
    repository: String,
}

pub fn execute(name: String, version: Option<String>) -> Result<()> {
    let manifest_path = Path::new("Cake.toml");

    if !manifest_path.exists() {
        terminal::error("No Cake.toml found. Run `cakeman init` first.");

        return Ok(());
    }

    terminal::info(&format!("Searching {} in registry...", name));

    let registry_content = match registry::get_cake_manifest(&name) {
        Ok(content) => content,

        Err(err) => {
            terminal::error(&err.to_string());
            return Ok(());
        }
    };

    let registry_package: CakeRegistryManifest = toml::from_str(&registry_content)
        .map_err(|err| anyhow!("Invalid registry manifest: {}", err))?;

    let resolved_version = match version {
        Some(version) if version != "latest" => version,

        _ => registry_package.package.version.clone(),
    };

    terminal::success(&format!("Found {} {}", name, resolved_version));

    let mut manifest = Manifest::load(manifest_path)?;

    if manifest.dependencies.contains_key(&name) {
        terminal::warn(&format!("{} is already a dependency", name));

        return Ok(());
    }

    manifest
        .dependencies
        .insert(name.clone(), resolved_version.clone());

    manifest.save(manifest_path)?;

    let mut lockfile = Lockfile::load()?;

    lockfile.add_package(LockedPackage {
        name: name.clone(),
        version: resolved_version,
        repository: registry_package.package.repository,
    });

    lockfile.save()?;

    terminal::success(&format!("Added {}", name));

    Ok(())
}
