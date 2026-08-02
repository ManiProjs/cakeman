use anyhow::Result;
use std::path::Path;

use crate::{compiler, dependency, lockfile::Lockfile, manifest::Manifest, terminal};

pub fn execute(release: bool) -> Result<()> {
    let manifest = Manifest::load("Cake.toml")?;

    manifest.validate()?;

    if manifest.package.package_type.as_deref() == Some("library") {
        terminal::error("Cannot build a library package directly.");

        terminal::hint("Create a binary package that depends on this library.");

        return Ok(());
    }

    let profile = if release { "release" } else { "debug" };

    terminal::info(&format!(
        "Building {} ({})...",
        manifest.package.name, profile
    ));

    let lockfile = Lockfile::load()?;

    terminal::info("Preparing dependencies...");

    let includes = dependency::prepare_dependencies(&lockfile)?;

    let include_dirs: Vec<String> = includes
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    let build_dir = Path::new(".cake").join("build").join(profile);

    if !build_dir.exists() {
        std::fs::create_dir_all(&build_dir)?;
    }

    terminal::info("Generating build files...");

    compiler::generate_cmake(&manifest, &include_dirs)?;

    compiler::run_cmake(&build_dir, release)?;

    terminal::success("Build completed successfully!");

    terminal::hint(&format!("Output directory: {}", build_dir.display()));

    Ok(())
}
