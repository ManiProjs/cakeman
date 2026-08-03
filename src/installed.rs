use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

const INSTALLED_FILE: &str = "installed.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Installed {
    pub version: u32,

    #[serde(default)]
    pub package: Vec<InstalledPackage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,

    #[serde(default)]
    pub dependencies: Vec<String>,
}

impl Default for Installed {
    fn default() -> Self {
        Self {
            version: 1,
            package: Vec::new(),
        }
    }
}

impl Installed {
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)?;

        Ok(toml::from_str(&content)?)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;

        fs::write(path, content)?;

        Ok(())
    }

    pub fn add(&mut self, package: InstalledPackage) {
        self.package.retain(|p| p.name != package.name);

        self.package.push(package);
    }

    pub fn remove(&mut self, name: &str) {
        self.package.retain(|p| p.name != name);
    }

    pub fn dependents(&self, name: &str) -> Vec<String> {
        self.package
            .iter()
            .filter(|p| p.dependencies.contains(&name.to_string()))
            .map(|p| p.name.clone())
            .collect()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.package.iter().any(|package| package.name == name)
    }
}
