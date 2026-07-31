use anyhow::Result;
use std::path::PathBuf;

use crate::{compiler, dependency, lockfile::Lockfile, manifest::Manifest, terminal};

pub fn execute() -> Result<()> {
    let manifest = Manifest::load("Cake.toml")?;

    terminal::info(&format!(
        "Generating build files for {}...",
        manifest.package.name
    ));

    let lockfile = Lockfile::load()?;

    let includes = dependency::prepare_dependencies(&lockfile)?;

    let include_dirs: Vec<String> = includes
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    compiler::generate_cmake(&manifest.package, &include_dirs)?;

    compiler::run_cmake()?;

    terminal::success("Build completed successfully!");

    Ok(())
}
