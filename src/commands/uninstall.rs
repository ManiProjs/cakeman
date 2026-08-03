use anyhow::{Result, anyhow};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{installed::Installed, terminal};

pub fn execute(name: String) -> Result<()> {
    let installed_path = cakeman_dir()?.join("installed.toml");

    let mut installed = Installed::load(&installed_path)?;

    let dependents = installed.dependents(&name);

    if !dependents.is_empty() {
        return Err(anyhow!(
            "{} is required by: {}",
            name,
            dependents.join(", ")
        ));
    }

    terminal::info(&format!("Uninstalling {}...", name));

    let package_dir = packages_dir()?.join(&name);

    if !package_dir.exists() {
        return Err(anyhow!("Package '{}' is not installed", name));
    }

    let manifest = package_dir.join("build").join("install_manifest.txt");

    if manifest.exists() {
        remove_installed_files(&manifest)?;
    }

    fs::remove_dir_all(&package_dir)?;

    installed.remove(&name);

    installed.save(&installed_path)?;

    terminal::success(&format!("Removed {}", name));

    Ok(())
}

fn cakeman_dir() -> Result<PathBuf> {
    Ok(dirs::home_dir()
        .ok_or_else(|| anyhow!("Cannot find home directory"))?
        .join(".cakeman"))
}

fn packages_dir() -> Result<PathBuf> {
    Ok(cakeman_dir()?.join("packages"))
}

fn remove_installed_files(manifest: &Path) -> Result<()> {
    let content = fs::read_to_string(manifest)?;

    for line in content.lines() {
        let path = Path::new(line);

        if path.exists() {
            terminal::info(&format!("Removing {}", path.display()));

            fs::remove_file(path)?;
        }
    }

    Ok(())
}
