use thiserror::Error;

use crate::model::AliasRecord;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("TOML serialization error: {0}")]
    Serialize(#[from] toml::ser::Error),

    #[error("TOML deserialization error: {0}")]
    Deserialize(#[from] toml::de::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AliasStore {
    pub aliases: Vec<AliasRecord>,
}

impl AliasStore {
    pub fn to_toml(&self) -> Result<String, StoreError> {
        let output = toml::to_string_pretty(self)?;
        Ok(output)
    }

    pub fn from_toml(input: &str) -> Result<Self, StoreError> {
        let store = toml::from_str(input)?;
        Ok(store)
    }
}
