use anyhow::{Result, anyhow};
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use tokio::sync::Mutex;

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
    progress: ProgressBar,
    installed: Arc<Mutex<Vec<String>>>,
}

impl Installer {
    pub fn new(progress: ProgressBar) -> Self {
        Self {
            progress,
            installed: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn download(&self, name: &str) -> Result<()> {
        self.progress.set_message(format!("Downloading {}", name));

        let content = registry::get_cake_manifest(name).await?;

        let manifest: Manifest = toml::from_str(&content)?;

        let package_dir = packages_dir()?.join(name);

        if !package_dir.exists() {
            clone_package(&manifest.package.repository, &package_dir)?;
        }

        self.progress.inc(1);

        Ok(())
    }

    pub async fn install(&self, name: &str) -> Result<()> {
        {
            let installed = self.installed.lock().await;

            if installed.contains(&name.to_string()) {
                return Ok(());
            }
        }

        let installed_path = cakeman_dir()?.join("installed.toml");

        let installed = Installed::load(&installed_path)?;

        if installed.contains(name) {
            self.progress.inc(1);
            return Ok(());
        }

        self.progress.set_message(format!("Installing {}", name));

        let package_dir = packages_dir()?.join(name);

        generate_cmake(&package_dir)?;

        build_package(&package_dir)?;

        let manifest = Manifest::load(&package_dir.join("Cake.toml"))?;

        record_install(
            &manifest.package.name,
            &manifest.package.version,
            manifest.dependencies.keys().cloned().collect(),
        )?;

        {
            let mut installed = self.installed.lock().await;

            installed.push(name.to_string());
        }

        self.progress.inc(1);

        terminal::info(&format!("✓ Installed {}", name));

        Ok(())
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

    let total = lockfile.package.len() as u64;

    let progress = ProgressBar::new(total);

    progress.set_style(
        ProgressStyle::with_template(
            "{spinner:.cyan} {msg:<30} [{bar:40.cyan}] {pos}/{len} {eta}",
        )?
        .progress_chars("━╸"),
    );

    progress.enable_steady_tick(std::time::Duration::from_millis(100));

    let installer = Installer::new(progress.clone());

    //
    // Download everything first
    //

    terminal::info("Downloading packages...");

    for package in &lockfile.package {
        installer.download(&package.name).await?;
    }

    progress.set_position(0);

    //
    // Install everything
    //

    terminal::info("Installing packages...");

    for package in &lockfile.package {
        installer.install(&package.name).await?;
    }

    progress.finish_and_clear();

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
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
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
