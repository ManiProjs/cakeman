use anyhow::Result;

pub fn execute(package_type: String) -> Result<()> {
    println!("Adding dependency: {}", package_type);

    // TODO:
    // - edit Cake.cman
    // - download manifest
    // - update lockfile

    Ok(())
}
