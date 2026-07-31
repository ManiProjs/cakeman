use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

const LOCKFILE: &str = "Cake.lock";

#[derive(Debug, Serialize, Deserialize)]
pub struct Lockfile {
    pub version: u32,

    #[serde(default)]
    pub package: Vec<LockedPackage>,
}

impl Default for Lockfile {
    fn default() -> Self {
        Self {
            version: 1,
            package: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub repository: String,
}

impl Lockfile {
    pub fn load() -> Result<Self> {
        let path = Path::new(LOCKFILE);

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)?;

        if content.trim().is_empty() || content.starts_with("#") {
            return Ok(Self::default());
        }

        let lockfile: Lockfile = toml::from_str(&content)?;

        Ok(lockfile)
    }

    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(self)?;

        fs::write(LOCKFILE, content)?;

        Ok(())
    }

    pub fn add_package(&mut self, package: LockedPackage) {
        self.package.retain(|p| p.name != package.name);

        self.package.push(package);
    }

    pub fn remove_package(&mut self, name: &str) {
        self.package.retain(|p| p.name != name);
    }

    pub fn contains(&self, name: &str) -> bool {
        self.package.iter().any(|p| p.name == name)
    }
}
