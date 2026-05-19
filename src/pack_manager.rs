use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::model::{AliasRecord, AliasShell, AliasSource};
use crate::pack_manifest::{ManifestError, PackManifest};

#[derive(Debug, Error)]
pub enum PackError {
     #[error("Pack '{0}' already exists")]
    PackExists(String),

     #[error("Pack '{0}' not found")]
    PackNotFound(String),

     #[error("Alias '{0}' already exists in pack '{1}'")]
    AliasExistsInPack(String, String),

     #[error("Alias '{0}' not found in pack '{1}'")]
    AliasNotFoundInPack(String, String),

     #[error("Manifest error: {0}")]
    Manifest(#[from] ManifestError),

     #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

     #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

     #[error("TOML deserialize error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),
}

/// Get the base directory for all packs: ~/.config/aliasman/packs/
pub fn get_packs_dir() -> Result<PathBuf, PackError> {
    let config_dir = dirs::config_dir()
         .ok_or_else(|| PackError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find config directory",
         )))?
        .join("aliasman")
        .join("packs");
    Ok(config_dir)
}

/// Get the directory for a specific pack
pub fn get_pack_dir(pack_name: &str) -> Result<PathBuf, PackError> {
    Ok(get_packs_dir()?
         .join(pack_name))
}

/// Get the path to a pack's manifest file
pub fn get_pack_manifest_path(pack_name: &str) -> Result<PathBuf, PackError> {
    Ok(get_pack_dir(pack_name)?
         .join("pack.toml"))
}

/// Get the path to a pack's aliases file
pub fn get_pack_aliases_path(pack_name: &str) -> Result<PathBuf, PackError> {
    Ok(get_pack_dir(pack_name)?
         .join("aliases.toml"))
}

/// Check if a pack directory exists
pub fn pack_exists(pack_name: &str) -> bool {
    get_pack_dir(pack_name).map(|p| p.exists()).unwrap_or(false)
}

/// Create a new pack with a default manifest.
///
/// Creates the pack directory and writes an initial pack.toml.
pub fn create_pack(
    name: String,
    version: String,
    description: Option<String>,
    author: Option<String>,
) -> Result<PathBuf, PackError> {
    if pack_exists(&name) {
        return Err(PackError::PackExists(name.clone()));
     }

    let pack_dir = get_pack_dir(&name)?;
    fs::create_dir_all(&pack_dir)?;

     // Create and save manifest
    let mut manifest = PackManifest::new(name.clone(), version);
    manifest.description = description;
    manifest.author = author;
    manifest.save_to_path(&pack_dir.join("pack.toml"))?;

     // Create empty aliases file
    let aliases_content = "aliases = []\n";
    fs::write(get_pack_aliases_path(&name)?, aliases_content)?;

    Ok(pack_dir)
}

/// Load a pack's manifest
pub fn load_pack_manifest(pack_name: &str) -> Result<PackManifest, PackError> {
    Ok(PackManifest::load_from_path(&get_pack_manifest_path(pack_name)?)?)
}

/// Load all aliases from a pack
pub fn load_pack_aliases(pack_name: &str) -> Result<Vec<AliasRecord>, PackError> {
    let aliases_path = get_pack_aliases_path(pack_name)?;

    if !aliases_path.exists() {
        return Ok(Vec::new());
     }

    let content = fs::read_to_string(&aliases_path)?;
    let data: PackAliasesFile = toml::from_str(&content)?;
    let mut aliases = data.aliases;

     // Ensure all aliases are marked as coming from this pack
    for alias in &mut aliases {
        alias.source = AliasSource::Pack(pack_name.to_string());
     }

    Ok(aliases)
}

/// Add an alias to an existing pack
pub fn add_alias_to_pack(
    pack_name: &str,
    name: String,
    command: String,
    description: Option<String>,
    tags: Vec<String>,
    shell: AliasShell,
) -> Result<(), PackError> {
    if !pack_exists(pack_name) {
        return Err(PackError::PackNotFound(pack_name.to_string()));
     }

    let mut aliases = load_pack_aliases(pack_name)?;

     // Check for duplicate name within this pack
    if aliases.iter().any(|a| a.name == name) {
        return Err(PackError::AliasExistsInPack(name, pack_name.to_string()));
     }

    let now = std::time::SystemTime::now()
         .duration_since(std::time::UNIX_EPOCH)
         .unwrap()
         .as_secs();

    aliases.push(AliasRecord {
        name,
        command,
        description,
        tags,
        shell,
        source: AliasSource::Pack(pack_name.to_string()),
        created_at: now,
        updated_at: now,
    modified_by_user: false,
     });

     // Persist back to aliases.toml
    save_pack_aliases(pack_name, &aliases)?;

    Ok(())
}

/// Remove an alias from a pack
pub fn remove_alias_from_pack(pack_name: &str, alias_name: &str) -> Result<(), PackError> {
    if !pack_exists(pack_name) {
        return Err(PackError::PackNotFound(pack_name.to_string()));
     }

    let mut aliases = load_pack_aliases(pack_name)?;
    let len_before = aliases.len();
    aliases.retain(|a| a.name != alias_name);

    if aliases.len() == len_before {
        return Err(PackError::AliasNotFoundInPack(
            alias_name.to_string(),
            pack_name.to_string(),
         ));
     }

    save_pack_aliases(pack_name, &aliases)?;

    Ok(())
}

/// Save aliases back to a pack's aliases.toml
pub fn save_pack_aliases(pack_name: &str, aliases: &[AliasRecord]) -> Result<(), PackError> {
    let data = PackAliasesFile {
        aliases: aliases.to_vec(),
     };
    let content = toml::to_string_pretty(&data)?;
    fs::write(get_pack_aliases_path(pack_name)?, content)?;
    Ok(())
}

/// Export a pack as a single shareable TOML file.
///
/// The output combines manifest metadata and all aliases into one file.
/// Returns the TOML content string.
pub fn export_pack(pack_name: &str) -> Result<String, PackError> {
    let manifest = load_pack_manifest(pack_name)?;
    let aliases = load_pack_aliases(pack_name)?;

    let export = PackExport {
        manifest,
        aliases,
     };

    toml::to_string_pretty(&export).map_err(PackError::TomlSerialize)
}

/// Write an exported pack TOML to a file
pub fn export_pack_to_file(pack_name: &str, output_path: &Path) -> Result<(), PackError> {
    let content = export_pack(pack_name)?;
    fs::write(output_path, content)?;
    Ok(())
}

/// Parse an exported pack TOML and return its manifest + aliases
pub fn parse_export_toml(content: &str) -> Result<PackExport, PackError> {
    let export: PackExport = toml::from_str(content)?;
    export.manifest.validate()?;
    Ok(export)
}

// -- Serde wrappers --

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PackAliasesFile {
    pub aliases: Vec<AliasRecord>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PackExport {
    pub manifest: PackManifest,
    pub aliases: Vec<AliasRecord>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::write_atomic;

     fn temp_packs_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("aliasman-packs-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
     }

     #[test]
    fn create_pack_creates_directory_and_manifest() {
        let pack_name = "test-pack";
        let pack_dir = get_pack_dir(pack_name).unwrap();

         // Clean up first
        let _ = fs::remove_dir_all(&pack_dir);

        let result = create_pack(
            pack_name.to_string(),
            "1.0.0".to_string(),
            Some("Test pack".to_string()),
            Some("Test Author".to_string()),
         );
        assert!(result.is_ok());
        assert!(pack_dir.exists());
        assert!(pack_dir.join("pack.toml").exists());
        assert!(pack_dir.join("aliases.toml").exists());

         // Verify manifest content
        let manifest = load_pack_manifest(pack_name).unwrap();
        assert_eq!(manifest.name, pack_name);
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.description, Some("Test pack".to_string()));

         // Cleanup
        let _ = fs::remove_dir_all(&pack_dir);
     }

     #[test]
    fn create_duplicate_pack_fails() {
        let pack_name = "dup-pack";
        let pack_dir = get_pack_dir(pack_name).unwrap();
        let _ = fs::remove_dir_all(&pack_dir);

        create_pack(pack_name.to_string(), "1.0.0".to_string(), None, None).unwrap();
        let result = create_pack(pack_name.to_string(), "1.0.0".to_string(), None, None);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));

        let _ = fs::remove_dir_all(&pack_dir);
     }

     #[test]
    fn add_and_load_aliases() {
        let pack_name = "alias-test-pack";
        let pack_dir = get_pack_dir(pack_name).unwrap();
        let _ = fs::remove_dir_all(&pack_dir);

        create_pack(pack_name.to_string(), "0.1.0".to_string(), None, None).unwrap();

        add_alias_to_pack(
            pack_name,
            "kget".to_string(),
            "kubectl get pods".to_string(),
            Some("Get pods".to_string()),
            vec!["k8s".to_string()],
            AliasShell::All,
         ).unwrap();

        let aliases = load_pack_aliases(pack_name).unwrap();
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].name, "kget");
        assert!(matches!(aliases[0].source, AliasSource::Pack(ref p) if p == pack_name));

        let _ = fs::remove_dir_all(&pack_dir);
     }

     #[test]
    fn export_pack_provides_combined_toml() {
        let pack_name = "export-test-pack";
        let pack_dir = get_pack_dir(pack_name).unwrap();
        let _ = fs::remove_dir_all(&pack_dir);

        create_pack(pack_name.to_string(), "1.0.0".to_string(),
            Some("Exportable".to_string()), None).unwrap();
        add_alias_to_pack(pack_name, "gs".to_string(), "git status".to_string(),
            None, vec![], AliasShell::All).unwrap();

        let exported = export_pack(pack_name).unwrap();
        assert!(exported.contains("name = \"export-test-pack\""));
        assert!(exported.contains("[aliases]"));

         // Verify roundtrip
        let parsed = parse_export_toml(&exported).unwrap();
        assert_eq!(parsed.manifest.name, pack_name);
        assert_eq!(parsed.aliases.len(), 1);

        let _ = fs::remove_dir_all(&pack_dir);
     }
}
