use anyhow::Result;
use std::{fs, path::Path, process::Command};

use crate::manifest::{Manifest, Package};

pub fn generate_cmake(manifest: &Manifest, include_dirs: &[String]) -> Result<()> {
    let package = &manifest.package;

    let cmake = match package.package_type.as_deref() {
        Some("library") => generate_library_cmake(package, include_dirs),

        _ => generate_binary_cmake(package, include_dirs),
    };

    fs::write("CMakeLists.txt", cmake)?;

    Ok(())
}

fn source_extension(package: &Package) -> &str {
    match package.language.as_deref() {
        Some("c") => "c",
        _ => "cpp",
    }
}

fn cmake_language(package: &Package) -> &str {
    match package.language.as_deref() {
        Some("c") => "C",
        _ => "CXX",
    }
}

fn generate_library_cmake(package: &Package, include_dirs: &[String]) -> String {
    let includes = format_include_dirs(include_dirs);

    let ext = source_extension(package);

    let language = cmake_language(package);

    format!(
        r#"cmake_minimum_required(VERSION 3.20)

project({name}
    VERSION {version}
    LANGUAGES {language}
)

add_library(
    {name}
    src/{name}.{ext}
)

target_include_directories(
    {name}
    PUBLIC
{includes}
)
"#,
        name = package.name,
        version = package.version,
        language = language,
        ext = ext,
        includes = includes,
    )
}

fn generate_binary_cmake(package: &Package, include_dirs: &[String]) -> String {
    let includes = format_include_dirs(include_dirs);

    let ext = source_extension(package);

    let language = cmake_language(package);

    let source = match ext {
        "c" => "main.c",
        _ => "main.cpp",
    };

    format!(
        r#"cmake_minimum_required(VERSION 3.20)

project({name}
    VERSION {version}
    LANGUAGES {language}
)

add_executable(
    {name}
    src/{source}
)

target_include_directories(
    {name}
    PRIVATE
{includes}
)
"#,
        name = package.name,
        version = package.version,
        language = language,
        source = source,
        includes = includes,
    )
}

fn format_include_dirs(include_dirs: &[String]) -> String {
    if include_dirs.is_empty() {
        return String::new();
    }

    include_dirs
        .iter()
        .map(|dir| format!("    {}", dir))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn run_cmake(build_dir: &Path, release: bool) -> Result<()> {
    let build_dir = build_dir.to_string_lossy();

    let mut configure = Command::new("cmake");

    configure.args(["-S", ".", "-B", &build_dir]);

    if release {
        configure.args(["-DCMAKE_BUILD_TYPE=Release"]);
    }

    configure.status()?;

    Command::new("cmake")
        .args(["--build", &build_dir])
        .status()?;

    Ok(())
}
