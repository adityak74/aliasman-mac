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
