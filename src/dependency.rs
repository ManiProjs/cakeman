use anyhow::{Result, anyhow};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::compiler::generate_dependency_cmake;

use crate::{lockfile::Lockfile, manifest::Manifest, terminal};

const PACKAGE_DIR: &str = ".cake/packages";

pub struct Dependency {
    pub name: String,
    pub path: PathBuf,
}

pub fn prepare_dependencies(lockfile: &Lockfile, root_package: &str) -> Result<Vec<Dependency>> {
    let mut dependencies = Vec::new();

    std::fs::create_dir_all(PACKAGE_DIR)?;

    for package in &lockfile.package {
        // The package we're building is already in the current directory.
        if package.name == root_package {
            continue;
        }

        let package_path = Path::new(PACKAGE_DIR).join(&package.name);

        if !package_path.exists() {
            clone_package(&package.repository, &package.version, &package_path)?;
        } else {
            terminal::info(&format!("Using cached {}", package.name));
        }

        let dependency_manifest = Manifest::load(package_path.join("Cake.toml"))?;

        let cmake = generate_dependency_cmake(&dependency_manifest)?;

        std::fs::write(package_path.join("CMakeLists.txt"), cmake)?;

        dependencies.push(Dependency {
            name: package.name.clone(),
            path: package_path,
        });
    }

    Ok(dependencies)
}

fn clone_package(repository: &str, version: &str, destination: &Path) -> Result<()> {
    terminal::info(&format!("Downloading {}...", repository));

    let status = Command::new("git")
        .args(["clone", repository, destination.to_str().unwrap()])
        .status()?;

    if !status.success() {
        return Err(anyhow!("Failed to clone {}", repository));
    }

    let version_tag = format!("v{}", version);

    let status = Command::new("git")
        .current_dir(destination)
        .args(["checkout", &version_tag])
        .status()?;

    if status.success() {
        return Ok(());
    }

    let status = Command::new("git")
        .current_dir(destination)
        .args(["checkout", version])
        .status()?;

    if !status.success() {
        return Err(anyhow!("Version {} not found", version));
    }

    Ok(())
}
