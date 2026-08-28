use anyhow::{Result, anyhow};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{compiler, manifest::Manifest, terminal};

pub fn execute() -> Result<()> {
    terminal::info("Packing CakePak...");

    let current = std::env::current_dir()?;

    let manifest_path = current.join("Cake.toml");

    if !manifest_path.exists() {
        return Err(anyhow!("Cake.toml not found"));
    }

    let manifest = Manifest::load(&manifest_path)?;

    let name = &manifest.package.name;
    let version = &manifest.package.version;

    let output = current.join(format!("{name}-{version}.cmpak"));

    let temp_dir = create_temp_dir()?;

    let result = build_cakepak(&current, &temp_dir, &output, &manifest);

    // Always clean up the temporary directory.
    let cleanup_result = fs::remove_dir_all(&temp_dir);

    if let Err(error) = result {
        let _ = cleanup_result;
        return Err(error);
    }

    cleanup_result?;

    terminal::success(&format!("Created {}", output.display()));

    Ok(())
}

fn build_cakepak(
    project_dir: &Path,
    temp_dir: &Path,
    output: &Path,
    manifest: &Manifest,
) -> Result<()> {
    let source_dir = temp_dir.join("source");
    let binary_dir = temp_dir.join("binary");

    fs::create_dir_all(&source_dir)?;
    fs::create_dir_all(&binary_dir)?;

    terminal::info("Copying source...");

    copy_source(project_dir, &source_dir)?;

    fs::copy(project_dir.join("Cake.toml"), temp_dir.join("Cake.toml"))?;

    let lockfile = project_dir.join("Cake.lock");

    if lockfile.exists() {
        fs::copy(lockfile, temp_dir.join("Cake.lock"))?;
    }

    terminal::info("Building package...");

    build_package(project_dir, &binary_dir)?;

    terminal::info("Compressing CakePak...");

    compress(temp_dir, output)?;

    Ok(())
}

fn create_temp_dir() -> Result<PathBuf> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();

    let pid = std::process::id();

    let path = std::env::temp_dir().join(format!("cman-cakepack-{pid}-{timestamp}"));

    fs::create_dir_all(&path)?;

    Ok(path)
}

fn copy_source(src: &Path, dst: &Path) -> Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();

        let file_name = entry.file_name();

        if should_skip(&file_name) {
            continue;
        }

        let target = dst.join(&file_name);

        if path.is_dir() {
            fs::create_dir_all(&target)?;
            copy_source(&path, &target)?;
        } else {
            fs::copy(&path, &target)?;
        }
    }

    Ok(())
}

fn should_skip(name: &std::ffi::OsStr) -> bool {
    matches!(
        name.to_str(),
        Some(".git" | "build" | ".cake" | ".cakepak" | "target" | ".DS_Store")
    )
}

fn build_package(project_dir: &Path, install_prefix: &Path) -> Result<()> {
    let build_dir = project_dir.join("build-cakepak");

    if build_dir.exists() {
        fs::remove_dir_all(&build_dir)?;
    }

    fs::create_dir_all(install_prefix)?;

    let generated_cmake =
        compiler::generate_dependency_cmake(&Manifest::load(&project_dir.join("Cake.toml"))?)?;

    fs::write(project_dir.join("CMakeLists.txt"), generated_cmake)?;

    let status = Command::new("cmake")
        .args([
            "-S",
            ".",
            "-B",
            "build-cakepak",
            "-G",
            "Ninja",
            "-DCMAKE_BUILD_TYPE=Release",
        ])
        .current_dir(project_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if !status.success() {
        return Err(anyhow!("CMake configuration failed"));
    }

    let status = Command::new("cmake")
        .args(["--build", "build-cakepak"])
        .current_dir(project_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if !status.success() {
        return Err(anyhow!("Build failed"));
    }

    let prefix = install_prefix
        .to_str()
        .ok_or_else(|| anyhow!("Invalid install prefix"))?;

    let status = Command::new("cmake")
        .args(["--install", "build-cakepak", "--prefix", prefix])
        .current_dir(project_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if !status.success() {
        return Err(anyhow!("Installation failed"));
    }

    fs::remove_dir_all(&build_dir)?;

    Ok(())
}

fn compress(staging_dir: &Path, output: &Path) -> Result<()> {
    let output_str = output
        .to_str()
        .ok_or_else(|| anyhow!("Invalid output path"))?;

    let status = Command::new("7z")
        .args(["a", "-t7z", output_str, "."])
        .current_dir(staging_dir)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .status()?;

    if !status.success() {
        return Err(anyhow!("Failed to compress CakePak"));
    }

    Ok(())
}
