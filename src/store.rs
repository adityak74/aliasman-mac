use std::fs;
use std::io::Write;
use std::path::Path;

use tempfile::NamedTempFile;
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

    #[error("Atomic persist error: {0}")]
    Persist(#[from] tempfile::PersistError),
}

pub const MANAGED_ALIASES_PATH: &str = "~/.aliases";

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

/// Render the alias store into a deterministic shell-compatible alias file.
///
/// Output is sorted by alias name and single-quotes are escaped
/// with the standard shell `'\''` sequence.
pub fn render_aliases_file(store: &AliasStore) -> String {
    let mut output = String::new();
    output.push_str("# aliasman managed - do not edit manually\n");
    output.push_str("# Run `aliasman list` to view aliases once CRUD commands are available.\n");

    let mut sorted = store.aliases.clone();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));

    for record in sorted {
        let escaped_command = escape_single_quotes(&record.command);
        output.push_str(&format!("alias {}='{}'\n", record.name, escaped_command));
    }

    output
}

/// Escape single quotes inside a shell alias command value.
///
/// Uses the standard shell sequence: close quote, escaped quote, reopen quote.
/// e.g. `echo 'hi'` becomes `echo '\''hi'\''`
fn escape_single_quotes(cmd: &str) -> String {
    cmd.replace('\'', "'\\''")
}

/// Write `contents` to `path` atomically.
///
/// Creates the parent directory if needed, writes to a same-directory
/// temporary file, flushes, and then persists (renames) over the target.
pub fn write_atomic(path: &Path, contents: &str) -> Result<(), StoreError> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create temp file in the same directory as the target for atomic rename
    let temp_file = NamedTempFile::new_in(path.parent().unwrap_or_else(|| Path::new(".")))?;

    // Write all contents
    {
        let mut handle = temp_file.as_file();
        handle.write_all(contents.as_bytes())?;
        handle.flush()?;
        handle.sync_all()?;
    }

    // Persist (rename) the temp file over the target
    temp_file.persist(path)?;

    Ok(())
}

/// Render the alias store and write it atomically to the given path.
pub fn write_managed_aliases(path: &Path, store: &AliasStore) -> Result<(), StoreError> {
    let rendered = render_aliases_file(store);
    write_atomic(path, &rendered)
}

/// Create a timestamped backup of a file before modifying it.
///
/// The backup is placed alongside the original with the pattern:
/// `{filename}.aliasman-backup-{YYYY-MM-DDTHH-MM-SS}`
///
/// Returns the backup file path.
pub fn backup_file(path: &Path) -> Result<std::path::PathBuf, StoreError> {
    let timestamp = chrono::Local::now().format("%Y-%m-%dT%H-%M-%S").to_string();

    let backup_name = format!(
        "{}.aliasman-backup-{}",
        path.file_name()
            .map(|f| f.to_string_lossy())
            .unwrap_or_default(),
        timestamp
    );

    let backup_path = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(backup_name);

    fs::copy(path, &backup_path)?;
    Ok(backup_path)
}

/// Prune old aliasman backups, keeping only the `keep_count` most recent.
///
/// Scans the parent directory for files matching `{base}.aliasman-backup-*`.
pub fn prune_backups(path: &Path, keep_count: usize) -> Result<(), StoreError> {
    let parent = path.parent().ok_or_else(|| {
        StoreError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "path has no parent directory",
        ))
    })?;

    let base_name = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .ok_or_else(|| {
            StoreError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "path has no file name",
            ))
        })?;

    let prefix = format!("{}.aliasman-backup-", base_name);

    let mut backups: Vec<std::path::PathBuf> = Vec::new();
    for entry in fs::read_dir(parent)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();
        if name_str.starts_with(&prefix) {
            backups.push(entry.path());
        }
    }

    // Sort by name (which includes timestamp) so oldest come first
    backups.sort();

    // Remove all but the last `keep_count`
    if backups.len() > keep_count {
        let to_remove = backups.split_at(backups.len() - keep_count).0;
        for old in to_remove {
            fs::remove_file(old)?;
        }
    }

    Ok(())
}

/// Add a new alias to the store. Returns error if alias already exists.
pub fn store_add_alias(
    store: &mut AliasStore,
    name: String,
    command: String,
    description: Option<String>,
    tags: Vec<String>,
    shell: crate::model::AliasShell,
) -> Result<(), String> {
    if store.aliases.iter().any(|a| a.name == name) {
        return Err(format!(
            "Alias '{}' already exists. Use `aliasman update --name {} --command \"...\"` to update it.",
            name, name
        ));
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    store.aliases.push(AliasRecord {
        name,
        command,
        description,
        tags,
        shell,
        source: crate::model::AliasSource::User,
        created_at: now,
        updated_at: now,
    });

    Ok(())
}

/// Update an existing alias. Returns error if alias doesn't exist.
pub fn store_update_alias(
    store: &mut AliasStore,
    name: &str,
    command: Option<String>,
    description: Option<Option<String>>,
    tags: Option<Vec<String>>,
) -> Result<(), String> {
    let record = store
        .aliases
        .iter_mut()
        .find(|a| a.name == name)
        .ok_or_else(|| format!("Alias '{}' not found. Use `aliasman list` to see existing aliases.", name))?;

    if let Some(cmd) = command {
        record.command = cmd;
    }
    if let Some(desc) = description {
        record.description = desc;
    }
    if let Some(t) = tags {
        record.tags = t;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    record.updated_at = now;

    Ok(())
}

/// Delete an alias by name. Returns error if alias doesn't exist.
pub fn store_delete_alias(store: &mut AliasStore, name: &str) -> Result<(), String> {
    let len_before = store.aliases.len();
    store.aliases.retain(|a| a.name != name);

    if store.aliases.len() == len_before {
        return Err(format!(
            "Alias '{}' not found. Use `aliasman list` to see existing aliases.",
            name
        ));
    }

    Ok(())
}

/// List all aliases, optionally filtered by shell kind.
pub fn store_list_aliases(
    store: &AliasStore,
    shell_filter: Option<crate::model::AliasShell>,
) -> Vec<&AliasRecord> {
    store
        .aliases
        .iter()
        .filter(|a| {
            shell_filter.map_or(true, |sf| {
                a.shell == crate::model::AliasShell::All || a.shell == sf
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AliasShell, AliasSource};

    fn make_sample_record() -> AliasRecord {
        AliasRecord {
            name: "gs".to_string(),
            command: "git status".to_string(),
            description: Some("Quick git status".to_string()),
            tags: vec!["git".to_string()],
            shell: AliasShell::All,
            source: AliasSource::User,
            created_at: 1715300000,
            updated_at: 1715300000,
        }
    }

    #[test]
    fn serializes_and_deserializes_alias_store() {
        let store = AliasStore {
            aliases: vec![make_sample_record()],
        };

        let toml_str = store.to_toml().expect("serialization should succeed");
        let deserialized =
            AliasStore::from_toml(&toml_str).expect("deserialization should succeed");

        assert_eq!(store, deserialized);
    }

    #[test]
    fn renders_aliases_in_name_order() {
        let store = AliasStore {
            aliases: vec![
                AliasRecord {
                    name: "z_last".to_string(),
                    command: "echo z".to_string(),
                    description: None,
                    tags: vec![],
                    shell: AliasShell::All,
                    source: AliasSource::User,
                    created_at: 0,
                    updated_at: 0,
                },
                AliasRecord {
                    name: "a_first".to_string(),
                    command: "echo a".to_string(),
                    description: None,
                    tags: vec![],
                    shell: AliasShell::All,
                    source: AliasSource::User,
                    created_at: 0,
                    updated_at: 0,
                },
            ],
        };

        let rendered = render_aliases_file(&store);
        let a_pos = rendered.find("alias a_first='echo a'").unwrap();
        let z_pos = rendered.find("alias z_last='echo z'").unwrap();

        assert!(a_pos < z_pos, "aliases should be sorted by name");
    }

    #[test]
    fn escapes_single_quotes_in_alias_commands() {
        let store = AliasStore {
            aliases: vec![AliasRecord {
                name: "x".to_string(),
                command: "echo 'hi'".to_string(),
                description: None,
                tags: vec![],
                shell: AliasShell::All,
                source: AliasSource::User,
                created_at: 0,
                updated_at: 0,
            }],
        };

        let rendered = render_aliases_file(&store);
        assert!(rendered.contains("alias x='echo '\\''hi'\\'''"));
    }

    #[test]
    fn store_add_alias_works() {
        let mut store = AliasStore::default();
        store_add_alias(
            &mut store,
            "gs".to_string(),
            "git status".to_string(),
            None,
            vec![],
            AliasShell::All,
        ).unwrap();
        assert_eq!(store.aliases.len(), 1);
        assert_eq!(store.aliases[0].name, "gs");
    }

    #[test]
    fn store_add_duplicate_fails() {
        let mut store = AliasStore {
            aliases: vec![make_sample_record()],
        };
        let result = store_add_alias(
            &mut store,
            "gs".to_string(),
            "git log".to_string(),
            None,
            vec![],
            AliasShell::All,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("already exists"));
        assert!(err.contains("aliasman update --name"));
    }

    #[test]
    fn store_update_existing_alias() {
        let mut store = AliasStore {
            aliases: vec![make_sample_record()],
        };
        store_update_alias(
            &mut store,
            "gs",
            Some("git status -s".to_string()),
            None,
            None,
        ).unwrap();
        assert_eq!(store.aliases[0].command, "git status -s");
    }

    #[test]
    fn store_update_missing_fails() {
        let mut store = AliasStore::default();
        let result = store_update_alias(&mut store, "missing", None, None, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn store_delete_existing() {
        let mut store = AliasStore {
            aliases: vec![make_sample_record()],
        };
        store_delete_alias(&mut store, "gs").unwrap();
        assert_eq!(store.aliases.len(), 0);
    }

    #[test]
    fn store_delete_missing_fails() {
        let mut store = AliasStore::default();
        let result = store_delete_alias(&mut store, "missing");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn store_list_filters_by_shell() {
        let store = AliasStore {
            aliases: vec![
                AliasRecord {
                    name: "all_alias".to_string(),
                    command: "echo all".to_string(),
                    description: None,
                    tags: vec![],
                    shell: AliasShell::All,
                    source: AliasSource::User,
                    created_at: 0,
                    updated_at: 0,
                },
                AliasRecord {
                    name: "zsh_only".to_string(),
                    command: "echo zsh".to_string(),
                    description: None,
                    tags: vec![],
                    shell: AliasShell::Zsh,
                    source: AliasSource::User,
                    created_at: 0,
                    updated_at: 0,
                },
            ],
        };
        let all = store_list_aliases(&store, None);
        assert_eq!(all.len(), 2);
        let zsh = store_list_aliases(&store, Some(AliasShell::Zsh));
        assert_eq!(zsh.len(), 2); // All + Zsh
        let bash = store_list_aliases(&store, Some(AliasShell::Bash));
        assert_eq!(bash.len(), 1); // Only All
    }
}
