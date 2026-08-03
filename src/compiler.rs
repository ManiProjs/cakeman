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

        _ => generate_binary_cmake(package, include_dirs, dependencies, true),
    };

    fs::write("CMakeLists.txt", cmake)?;

    Ok(())
}

pub fn generate_dependency_cmake(manifest: &Manifest) -> Result<String> {
    let package = &manifest.package;

    let dependencies = manifest.dependencies.keys().cloned().collect::<Vec<_>>();

    let cakeman_include = dirs::home_dir()
        .ok_or_else(|| anyhow!("Cannot find home directory"))?
        .join(".cakeman/include")
        .to_string_lossy()
        .to_string();

    let includes = vec!["include".to_string(), cakeman_include];

    let cmake = match package.package_type.as_deref() {
        Some("library") => generate_library_cmake(package, &includes, &dependencies),

        _ => generate_binary_cmake(package, &includes, &dependencies, false),
    };

    Ok(cmake)
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

install(
    TARGETS {name}
    ARCHIVE DESTINATION lib
    LIBRARY DESTINATION lib
)

install(
    DIRECTORY include/
    DESTINATION include
)
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
    local_dependencies: bool,
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

    let link_dirs = if local_dependencies {
        String::new()
    } else {
        let cakeman_lib = dirs::home_dir()
            .unwrap()
            .join(".cakeman/lib")
            .to_string_lossy()
            .to_string();

        format!(
            r#"
target_link_directories(
    {name}
    PRIVATE
    {cakeman_lib}
)
"#,
            name = package.name,
            cakeman_lib = cakeman_lib,
        )
    };

    let rpath = if local_dependencies {
        String::new()
    } else {
        let cakeman_lib = dirs::home_dir()
            .unwrap()
            .join(".cakeman/lib")
            .to_string_lossy()
            .to_string();

        format!(
            r#"
set_target_properties(
    {name}
    PROPERTIES
    INSTALL_RPATH "{cakeman_lib}"
)
"#,
            name = package.name,
            cakeman_lib = cakeman_lib,
        )
    };

    let ext = source_extension(package);
    let language = cmake_language(package);

    let source = match ext {
        "c" => "main.c",
        _ => "main.cpp",
    };

    let subdirs = if local_dependencies {
        dependencies
            .iter()
            .map(|dep| format!("add_subdirectory(.cake/packages/{})", dep))
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        String::new()
    };

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

{link_dirs}

{rpath}

{link_section}

install(
    TARGETS {name}
    RUNTIME DESTINATION bin
)
"#,
        name = package.name,
        version = package.version,
        language = language,
        source = source,
        includes = includes,
        link_dirs = link_dirs,
        rpath = rpath,
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

    command.args(["-S", ".", "-B", &build_dir, "-G", "Ninja"]);

    if release {
        command.arg("-DCMAKE_BUILD_TYPE=Release");
    }

    let status = command.status()?;

    if !status.success() {
        return Err(anyhow!("CMake configuration failed"));
    }

    let status = Command::new("cmake")
        .args(["--build", &build_dir])
        .status()?;

    if !status.success() {
        return Err(anyhow!("Build failed"));
    }

    Ok(())
}
