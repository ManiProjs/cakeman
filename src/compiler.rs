use anyhow::{Result, anyhow};
use std::{fs, path::Path, process::Command};

use crate::manifest::{Manifest, Package};

pub fn generate_cmake(
    manifest: &Manifest,
    include_dirs: &[String],
    dependencies: &[String],
) -> Result<()> {
    let package = &manifest.package;

    let cmake = match package.package_type.as_deref() {
        Some("library") => generate_library_cmake(package, include_dirs, dependencies),

        _ => generate_binary_cmake(package, include_dirs, dependencies),
    };

    fs::write("CMakeLists.txt", cmake)?;

    Ok(())
}

pub fn generate_dependency_cmake(manifest: &Manifest) -> Result<String> {
    let package = &manifest.package;

    let ext = source_extension(package);
    let language = cmake_language(package);

    let dependencies = manifest.dependencies.keys().cloned().collect::<Vec<_>>();

    let deps = format_dependencies(&dependencies);

    let link_section = if dependencies.is_empty() {
        String::new()
    } else {
        format!(
            r#"
target_link_libraries(
    {name}
    PUBLIC
{deps}
)
"#,
            name = package.name,
            deps = deps,
        )
    };

    Ok(format!(
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
    include
){link_section}
"#,
        name = package.name,
        version = package.version,
        language = language,
        ext = ext,
        link_section = link_section,
    ))
}

fn generate_library_cmake(
    package: &Package,
    include_dirs: &[String],
    dependencies: &[String],
) -> String {
    let includes = format_include_dirs(include_dirs);
    let deps = format_dependencies(dependencies);

    let link_section = if dependencies.is_empty() {
        String::new()
    } else {
        format!(
            r#"
target_link_libraries(
    {name}
    PRIVATE
{deps}
)
"#,
            name = package.name,
            deps = deps,
        )
    };

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

{link_section}
"#,
        name = package.name,
        version = package.version,
        language = language,
        ext = ext,
        includes = includes,
        link_section = link_section,
    )
}

fn generate_binary_cmake(
    package: &Package,
    include_dirs: &[String],
    dependencies: &[String],
) -> String {
    let includes = format_include_dirs(include_dirs);
    let deps = format_dependencies(dependencies);

    let link_section = if dependencies.is_empty() {
        String::new()
    } else {
        format!(
            r#"
target_link_libraries(
    {name}
    PRIVATE
{deps}
)
"#,
            name = package.name,
            deps = deps,
        )
    };

    let ext = source_extension(package);
    let language = cmake_language(package);

    let source = match ext {
        "c" => "main.c",
        _ => "main.cpp",
    };

    let subdirs = dependencies
        .iter()
        .map(|dep| format!("add_subdirectory(.cake/packages/{})", dep))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"cmake_minimum_required(VERSION 3.20)

project({name}
    VERSION {version}
    LANGUAGES {language}
)

{subdirs}

add_executable(
    {name}
    src/{source}
)

target_include_directories(
    {name}
    PRIVATE
{includes}
)

{link_section}
"#,
        name = package.name,
        version = package.version,
        language = language,
        source = source,
        includes = includes,
        link_section = link_section,
        subdirs = subdirs,
    )
}

fn format_dependencies(dependencies: &[String]) -> String {
    dependencies
        .iter()
        .map(|dep| format!("    {}", dep))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_include_dirs(include_dirs: &[String]) -> String {
    include_dirs
        .iter()
        .map(|dir| format!("    {}", dir))
        .collect::<Vec<_>>()
        .join("\n")
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

pub fn run_cmake(build_dir: &Path, release: bool) -> Result<()> {
    let build_dir = build_dir.to_string_lossy();

    let mut command = Command::new("cmake");

    command.args(["-S", ".", "-B", &build_dir]);

    if release {
        command.args(["-DCMAKE_BUILD_TYPE=Release"]);
    }

    let status = command.status()?;

    if !status.success() {
        return Err(anyhow!("CMake configuration failed"));
    }

    Command::new("cmake")
        .args(["--build", &build_dir])
        .status()?;

    Ok(())
}
