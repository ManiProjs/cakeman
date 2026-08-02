use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub package: Package,

    #[serde(default)]
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub repository: String,

    #[serde(rename = "type")]
    pub package_type: Option<String>,

    pub language: Option<String>,
}

impl Manifest {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(path)?;

        let manifest = toml::from_str(&content)?;

        Ok(manifest)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = toml::to_string_pretty(self)?;

        fs::write(path, content)?;

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if self.package.repository.trim().is_empty() {
            return Err(anyhow!("Package repository is required"));
        }

        Ok(())
    }
}
