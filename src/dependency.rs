use anyhow::{Result, anyhow};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{compiler, terminal};
use crate::{lockfile::Lockfile, manifest::Manifest};

const PACKAGE_DIR: &str = ".cake/packages";

pub struct Dependency {
    pub name: String,
    pub path: PathBuf,
}

fn generate_dependency_cmake(path: &Path) -> Result<()> {
    let manifest_path = path.join("Cake.toml");

    if !manifest_path.exists() {
        return Err(anyhow!("Package does not contain Cake.toml"));
    }

    let manifest = Manifest::load(&manifest_path)?;

    let cmake_path = path.join("CMakeLists.txt");

    if cmake_path.exists() {
        return Ok(());
    }

    let cmake = compiler::generate_dependency_cmake(&manifest)?;

    fs::write(cmake_path, cmake)?;

    Ok(())
}

pub fn prepare_dependencies(lockfile: &Lockfile) -> Result<Vec<Dependency>> {
    let mut dependencies = Vec::new();

    fs::create_dir_all(PACKAGE_DIR)?;

    for package in &lockfile.package {
        let package_path = Path::new(PACKAGE_DIR).join(&package.name);

        if !package_path.exists() {
            clone_package(&package.repository, &package.version, &package_path)?;
        } else {
            terminal::info(&format!("Using cached {}", package.name));
        }

        generate_dependency_cmake(&package_path)?;

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
        .arg("clone")
        .arg(repository)
        .arg(destination)
        .status()?;

    if !status.success() {
        return Err(anyhow!("Failed to clone {}", repository));
    }

    let status = Command::new("git")
        .current_dir(destination)
        .args(["checkout", &format!("v{}", version)])
        .status()?;

    if !status.success() {
        let status = Command::new("git")
            .current_dir(destination)
            .args(["checkout", version])
            .status()?;

        if !status.success() {
            return Err(anyhow!("Version {} not found", version));
        }
    }

    if !status.success() {
        return Err(anyhow!("Failed to checkout {}", version));
    }

    Ok(())
}
