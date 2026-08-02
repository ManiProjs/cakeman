use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::{collections::HashSet, future::Future, pin::Pin};

use crate::{
    lockfile::{LockedPackage, Lockfile},
    registry,
};

#[derive(Debug, Deserialize)]
struct CakeManifest {
    package: Package,

    #[serde(default)]
    dependencies: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    version: String,
    repository: String,
}

pub struct Resolver {
    visited: HashSet<String>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            visited: HashSet::new(),
        }
    }

    pub fn resolve<'a>(
        &'a mut self,
        name: &'a str,
        version: Option<&'a str>,
        lockfile: &'a mut Lockfile,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
        Box::pin(async move {
            if self.visited.contains(name) {
                return Ok(());
            }

            self.visited.insert(name.to_string());

            let content = registry::get_cake_manifest(name).await?;

            let manifest: CakeManifest = toml::from_str(&content)
                .map_err(|e| anyhow!("Invalid manifest for {}: {}", name, e))?;

            let resolved_version = match version {
                Some(v) if v != "latest" => v.to_string(),

                _ => manifest.package.version.clone(),
            };

            if !lockfile.contains(name) {
                lockfile.add_package(LockedPackage {
                    name: manifest.package.name.clone(),
                    version: resolved_version,
                    repository: manifest.package.repository.clone(),
                });
            }

            for (dependency, dep_version) in manifest.dependencies {
                self.resolve(&dependency, Some(&dep_version), lockfile)
                    .await?;
            }

            Ok(())
        })
    }
}
