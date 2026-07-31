use anyhow::Result;

pub fn execute(name: String) -> Result<()> {
    println!("Adding dependency: {}", name);

    // TODO:
    // - edit Cake.cman
    // - download manifest
    // - update lockfile

    Ok(())
}
