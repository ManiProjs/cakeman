use anyhow::{Result, anyhow};
use std::{
    fs,
    io::Stdout,
    path::{Path, PathBuf},
    process::Command,
};

use crate::lockfile::Lockfile;

const PACKAGE_DIR: &str = ".cman/packages";

pub fn prepare_dependencies(lockfile: &Lockfile) -> Result<Vec<PathBuf>> {
    let mut includes = Vec::new();

    fs::create_dir_all(PACKAGE_DIR)?;

    for package in &lockfile.package {
        let package_path = Path::new(PACKAGE_DIR).join(&package.name);

        if !package_path.exists() {
            clone_package(&package.repository, &package_path)?;
        }

        let include_path = package_path.join("include");

        if include_path.exists() {
            includes.push(include_path);
        }
    }

    Ok(includes)
}

fn clone_package(repository: &str, destination: &Path) -> Result<()> {
    println!("Downloading {}...", repository);

    let status = Command::new("git")
        .arg("clone")
        .arg(repository)
        .arg(destination)
        .status()?;

    if !status.success() {
        return Err(anyhow!("Failed to clone {}", repository));
    }

    Ok(())
}
