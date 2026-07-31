use anyhow::Result;

pub fn execute(release: bool) -> Result<()> {
    println!("Building project...");

    if release {
        println!("Release build");
    }

    // TODO:
    // - read Cake.cman
    // - resolve dependencies
    // - generate CMake
    // - run cmake

    Ok(())
}
