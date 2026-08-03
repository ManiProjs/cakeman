use anyhow::{Result, anyhow};
use std::{
    collections::HashSet,
    fs,
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    process::Command,
};

use crate::{
    compiler,
    installed::{Installed, InstalledPackage},
    manifest::Manifest,
    registry, terminal,
};

pub struct Installer {
    installed: HashSet<String>,
}

impl Installer {
    pub fn new() -> Self {
        Self {
            installed: HashSet::new(),
        }
    }

    pub fn install<'a>(
        &'a mut self,
        name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            if self.installed.contains(name) {
                return Ok(());
            }

            self.installed.insert(name.to_string());

            let installed_path = cakeman_dir()?.join("installed.toml");
            let installed = Installed::load(&installed_path)?;

            if installed.contains(name) {
                terminal::info(&format!("{} is already installed", name));
                return Ok(());
            }

            terminal::info(&format!("Installing {}...", name));

            let registry_manifest = registry::get_cake_manifest(name).await?;

            let manifest: Manifest = toml::from_str(&registry_manifest)?;

            // Install dependencies first
            for (dependency, _) in &manifest.dependencies {
                self.install(dependency).await?;
            }

            let package_dir = packages_dir()?.join(name);

            if package_dir.exists() {
                terminal::info(&format!("Using cached {}", name));
            } else {
                clone_package(&manifest.package.repository, &package_dir)?;
            }

            generate_cmake(&package_dir)?;

            build_package(&package_dir)?;

            record_install(
                &manifest.package.name,
                &manifest.package.version,
                manifest.dependencies.keys().cloned().collect(),
            )?;

            Ok(())
        })
    }
}

pub async fn execute(name: String) -> Result<()> {
    let mut installer = Installer::new();

    installer.install(&name).await?;

    terminal::success(&format!("Installed {}", name));

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

fn clone_package(repository: &str, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination.parent().unwrap())?;

    terminal::info(&format!("Downloading {}", repository));

    let status = Command::new("git")
        .args(["clone", repository, destination.to_str().unwrap()])
        .status()?;

    if !status.success() {
        return Err(anyhow!("Failed to clone {}", repository));
    }

    Ok(())
}

fn generate_cmake(path: &Path) -> Result<()> {
    let manifest_path = path.join("Cake.toml");

    if !manifest_path.exists() {
        return Err(anyhow!("Cake.toml not found"));
    }

    let manifest = Manifest::load(&manifest_path)?;

    terminal::info(&format!(
        "Generating CMakeLists for {} ({:?})",
        manifest.package.name, manifest.package.package_type
    ));

    let cmake = compiler::generate_dependency_cmake(&manifest)?;

    fs::write(path.join("CMakeLists.txt"), cmake)?;

    Ok(())
}

fn build_package(path: &Path) -> Result<()> {
    terminal::info("Building package...");

    let status = Command::new("cmake")
        .args([
            "-S",
            ".",
            "-B",
            "build",
            "-G",
            "Ninja",
            "-DCMAKE_BUILD_TYPE=Release",
        ])
        .current_dir(path)
        .status()?;

    if !status.success() {
        return Err(anyhow!("CMake configuration failed"));
    }

    let status = Command::new("cmake")
        .args(["--build", "build"])
        .current_dir(path)
        .status()?;

    if !status.success() {
        return Err(anyhow!("Build failed"));
    }

    terminal::info("Installing package...");

    let prefix = cakeman_dir()?;

    let status = Command::new("cmake")
        .args(["--install", "build", "--prefix", prefix.to_str().unwrap()])
        .current_dir(path)
        .status()?;

    if !status.success() {
        return Err(anyhow!("Installation failed"));
    }

    Ok(())
}

fn record_install(name: &str, version: &str, dependencies: Vec<String>) -> Result<()> {
    let path = cakeman_dir()?.join("installed.toml");

    let mut installed = Installed::load(&path)?;

    installed.add(InstalledPackage {
        name: name.to_string(),
        version: version.to_string(),
        dependencies,
    });

    installed.save(&path)?;

    Ok(())
}
