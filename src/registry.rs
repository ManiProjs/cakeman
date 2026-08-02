use anyhow::{Result, anyhow};
use reqwest::Client;

const REGISTRY_URL: &str = "https://raw.githubusercontent.com/beaglesoftware/cakes/main/manifests";

pub async fn get_cake_manifest(name: &str) -> Result<String> {
    let first_char = name
        .chars()
        .next()
        .ok_or_else(|| anyhow!("Package name cannot be empty"))?
        .to_ascii_lowercase();

    let url = format!("{}/{}/{}/Cake.toml", REGISTRY_URL, first_char, name);

    let response = Client::new().get(&url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!("Package '{}' was not found in registry", name));
    }

    Ok(response.text().await?)
}
