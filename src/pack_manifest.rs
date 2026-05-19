use std::fs;
use std::path::Path;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("Invalid pack name: {0}")]
    InvalidName(String),

    #[error("Invalid version: {0}")]
    InvalidVersion(#[from] semver::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

     #[error("TOML deserialize error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

pub const DEFAULT_FORMAT_VERSION: &str = "0.1";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PackManifest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub format_version: String,
}

impl PackManifest {
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            description: None,
            author: None,
            format_version: DEFAULT_FORMAT_VERSION.to_string(),
        }
    }

    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.name.is_empty() {
            return Err(ManifestError::InvalidName(
                "Pack name cannot be empty".to_string(),
            ));
        }

        // Validate name contains only alphanumeric, hyphens, underscores
        if !self.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(ManifestError::InvalidName(
                "Pack name must contain only alphanumeric characters, hyphens, and underscores"
                    .to_string(),
            ));
        }

        // Validate version parses as semver
        semver::Version::parse(&self.version)?;

        Ok(())
    }

    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn from_toml(input: &str) -> Result<Self, ManifestError> {
        let manifest: PackManifest = toml::from_str(input)?;
        manifest.validate()?;
        Ok(manifest)
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), ManifestError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let toml_str = self.to_toml()?;
        fs::write(path, toml_str)?;
        Ok(())
    }

    pub fn load_from_path(path: &Path) -> Result<Self, ManifestError> {
        let content = fs::read_to_string(path)?;
        Self::from_toml(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_valid_manifest() {
        let m = PackManifest::new("k8s".to_string(), "1.0.0".to_string());
        assert!(m.validate().is_ok());
        assert_eq!(m.format_version, DEFAULT_FORMAT_VERSION);
    }

    #[test]
    fn rejects_empty_name() {
        let m = PackManifest::new("".to_string(), "1.0.0".to_string());
        let err = m.validate().unwrap_err();
        assert!(err.to_string().contains("cannot be empty"));
    }

    #[test]
    fn rejects_invalid_version() {
        let m = PackManifest::new("test".to_string(), "not-a-version".to_string());
        assert!(m.validate().is_err());
    }

    #[test]
    fn roundtrips_through_toml() {
        let m = PackManifest::new("docker".to_string(), "0.1.0".to_string());
        let serialized = m.to_toml().unwrap();
        let deserialized = PackManifest::from_toml(&serialized).unwrap();
        assert_eq!(m, deserialized);
    }
}
