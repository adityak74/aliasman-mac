use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("TOML deserialize error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),
}

pub const REGISTRY_PATH: &str = "~/.config/aliasman/registry.toml";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RegistryEntry {
    pub name: String,
    pub version: String,
    pub source: String,
    pub install_time: u64,
    pub alias_count: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PackRegistry {
    pub packs: Vec<RegistryEntry>,
}

impl PackRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_registry_path() -> PathBuf {
        if let Some(home) = dirs::home_dir() {
            home.join(".config").join("aliasman").join("registry.toml")
        } else {
            PathBuf::from("/tmp/aliasman-registry.toml")
        }
    }

    pub fn load() -> Result<Self, RegistryError> {
        let path = Self::get_registry_path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)?;
        let registry: PackRegistry = toml::from_str(&content)?;
        Ok(registry)
    }

    pub fn save(&self) -> Result<(), RegistryError> {
        let path = Self::get_registry_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn register_pack(&mut self, name: String, version: String, source: String, alias_count: usize) {
        // Remove existing entry if present
        self.packs.retain(|p| p.name != name);

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.packs.push(RegistryEntry {
            name,
            version,
            source,
            install_time: now,
            alias_count,
        });
    }

    pub fn unregister_pack(&mut self, name: &str) -> bool {
        let len_before = self.packs.len();
        self.packs.retain(|p| p.name != name);
        self.packs.len() < len_before
    }

    pub fn get_pack(&self, name: &str) -> Option<&RegistryEntry> {
        self.packs.iter().find(|p| p.name == name)
    }

    pub fn list_packs(&self) -> &[RegistryEntry] {
        &self.packs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_empty_registry() {
        let reg = PackRegistry::new();
        assert!(reg.packs.is_empty());
    }

    #[test]
    fn register_and_list_pack() {
        let mut reg = PackRegistry::new();
        reg.register_pack(
            "k8s".to_string(),
            "1.0.0".to_string(),
            "/tmp/k8s-pack.toml".to_string(),
            15,
        );

        let packs = reg.list_packs();
        assert_eq!(packs.len(), 1);
        assert_eq!(packs[0].name, "k8s");
        assert_eq!(packs[0].alias_count, 15);
    }

    #[test]
    fn unregister_pack() {
        let mut reg = PackRegistry::new();
        reg.register_pack("test".to_string(), "0.1.0".to_string(), "/tmp/test.toml".to_string(), 5);
        assert_eq!(reg.packs.len(), 1);

        let removed = reg.unregister_pack("test");
        assert!(removed);
        assert_eq!(reg.packs.len(), 0);
    }

    #[test]
    fn unregister_nonexistent_returns_false() {
        let mut reg = PackRegistry::new();
        let removed = reg.unregister_pack("nonexistent");
        assert!(!removed);
    }

    #[test]
    fn get_pack_returns_entry() {
        let mut reg = PackRegistry::new();
        reg.register_pack("docker".to_string(), "0.2.0".to_string(), "/tmp/docker.toml".to_string(), 12);

        let entry = reg.get_pack("docker");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().version, "0.2.0");
    }

    #[test]
    fn register_same_pack_updates_entry() {
        let mut reg = PackRegistry::new();
        reg.register_pack("k8s".to_string(), "1.0.0".to_string(), "/tmp/k8s.toml".to_string(), 15);
        reg.register_pack("k8s".to_string(), "1.1.0".to_string(), "/tmp/k8s-v2.toml".to_string(), 20);

        assert_eq!(reg.packs.len(), 1);
        assert_eq!(reg.packs[0].version, "1.1.0");
        assert_eq!(reg.packs[0].alias_count, 20);
    }
}
