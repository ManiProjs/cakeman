use anyhow::{Result, anyhow};
use reqwest::blocking::Client;

const REGISTRY_URL: &str = "https://raw.githubusercontent.com/beaglesoftware/cakes/main/manifests";

pub fn get_cake_manifest(name: &str) -> Result<String> {
    let first_char = name
        .chars()
        .next()
        .ok_or_else(|| anyhow!("Package name cannot be empty"))?;

    let url = format!(
        "{}/{}/{}.cman",
        REGISTRY_URL,
        first_char.to_ascii_lowercase(),
        name
    );

    let response = Client::new().get(&url).send()?;

    if !response.status().is_success() {
        return Err(anyhow!("Package '{}' was not found in registry", name));
    }

    Ok(response.text()?)
}
