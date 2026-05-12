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
}
