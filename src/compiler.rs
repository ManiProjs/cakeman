use anyhow::{Result, anyhow};
use std::{fs, path::Path, process::Command};

use crate::manifest::Package;

pub fn generate_cmake(package: &Package, include_dirs: &[String]) -> Result<()> {
    let mut cmake = format!(
        r#"cmake_minimum_required(VERSION 3.20)

project({})

set(CMAKE_C_STANDARD 11)

add_executable(
    {}
    {}
)
"#,
        package.name, package.name, package.main
    );

    if !include_dirs.is_empty() {
        cmake.push_str("\n");

        cmake.push_str(&format!(
            r#"target_include_directories(
    {}
    PRIVATE
"#,
            package.name
        ));

        for dir in include_dirs {
            cmake.push_str(&format!("    {}\n", dir));
        }

        cmake.push_str(")\n");
    }

    fs::write("CMakeLists.txt", cmake)?;

    Ok(())
}

pub fn run_cmake() -> Result<()> {
    let configure = Command::new("cmake")
        .args(["-B", "target/build"])
        .status()?;

    if !configure.success() {
        return Err(anyhow!("CMake configuration failed"));
    }

    let build = Command::new("cmake")
        .args(["--build", "target/build"])
        .status()?;

    if !build.success() {
        return Err(anyhow!("CMake build failed"));
    }

    Ok(())
}
