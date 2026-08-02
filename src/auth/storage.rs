use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub github_token: String,
}

fn auth_path() -> Result<std::path::PathBuf> {
    Ok(dirs::config_dir()
        .ok_or_else(|| anyhow!("Cannot find config directory"))?
        .join("cakeman")
        .join("auth.json"))
}

pub fn save(token: &str) -> Result<()> {
    let path = auth_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let config = AuthConfig {
        github_token: token.to_string(),
    };

    fs::write(path, serde_json::to_string_pretty(&config)?)?;

    Ok(())
}

pub fn load() -> Result<Option<AuthConfig>> {
    let path = auth_path()?;

    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(path)?;

    Ok(Some(serde_json::from_str(&content)?))
}

pub fn token() -> Result<String> {
    load()?
        .map(|auth| auth.github_token)
        .ok_or_else(|| anyhow!("Not authenticated. Run `cakeman auth` first."))
}
