use anyhow::{Result, anyhow};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{
    fs,
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    process::Command,
};

use crate::{
    compiler,
    installed::{Installed, InstalledPackage},
    lockfile::Lockfile,
    manifest::Manifest,
    registry,
    resolver::Resolver,
    terminal,
};

pub struct Installer {
    multi: MultiProgress,
}

impl Installer {
    pub fn new(multi: MultiProgress) -> Self {
        Self { multi }
    }

    pub fn install<'a>(&'a self, name: &'a str) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            let installed_path = cakeman_dir()?.join("installed.toml");

            let installed = Installed::load(&installed_path)?;

            if installed.contains(name) {
                return Ok(());
            }

            let progress = self.multi.add(ProgressBar::new(100));

            progress.set_style(
                ProgressStyle::with_template(
                    "  {spinner:.cyan} {msg:<25} [{bar:30.cyan}] {percent}% {eta}",
                )?
                .progress_chars("━╸"),
            );

            progress.set_message(format!("Installing {}", name));

            let registry_manifest = registry::get_cake_manifest(name).await?;

            let manifest: Manifest = toml::from_str(&registry_manifest)?;

            let package_dir = packages_dir()?.join(name);

            if !package_dir.exists() {
                clone_package(&manifest.package.repository, &package_dir)?;
            }

            progress.set_message(format!("Building {}", name));
            progress.set_position(30);

            generate_cmake(&package_dir)?;

            build_package(&package_dir)?;

            progress.set_position(90);

            record_install(
                &manifest.package.name,
                &manifest.package.version,
                manifest.dependencies.keys().cloned().collect(),
            )?;

            progress.set_position(100);

            progress.finish_and_clear();

            terminal::info(&format!("✓ Installed {}", name));

            Ok(())
        })
    }
}

pub async fn execute(names: Vec<String>) -> Result<()> {
    terminal::info("Resolving dependencies...");

    let mut lockfile = Lockfile::load()?;

    let mut resolver = Resolver::new();

    for name in &names {
        resolver.resolve(name, None, &mut lockfile).await?;
    }

    lockfile.save()?;

    fs::create_dir_all(cakeman_dir()?)?;

    let multi = MultiProgress::new();

    let installer = Installer::new(multi.clone());

    for package in &lockfile.package {
        installer.install(&package.name).await?;
    }

    multi.clear()?;

    terminal::success("Installation complete");

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

    let status = Command::new("git")
        .args([
            "clone",
            "--quiet",
            repository,
            destination.to_str().unwrap(),
        ])
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

    let cmake = compiler::generate_dependency_cmake(&manifest)?;

    fs::write(path.join("CMakeLists.txt"), cmake)?;

    Ok(())
}

fn build_package(path: &Path) -> Result<()> {
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
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    if !status.success() {
        return Err(anyhow!("CMake configuration failed"));
    }

    let status = Command::new("cmake")
        .args(["--build", "build"])
        .current_dir(path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;

    if !status.success() {
        return Err(anyhow!("Build failed"));
    }

    let prefix = cakeman_dir()?;

    let status = Command::new("cmake")
        .args(["--install", "build", "--prefix", prefix.to_str().unwrap()])
        .current_dir(path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
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
