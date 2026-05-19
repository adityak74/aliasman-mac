use std::collections::HashMap;
use std::path::Path;

use crate::model::{AliasRecord, AliasShell, AliasSource};
use crate::shell::ShellKind;
use crate::store::AliasStore;
use crate::validation::{is_protected_name, validate_alias_name};

/// Managed block markers inserted into shell config files.
pub const MANAGED_BLOCK_START: &str = "# >>> aliasman >>>";
pub const MANAGED_BLOCK_END: &str = "# <<< aliasman <<<";

/// The body of the managed source block.
fn managed_block_body() -> String {
    "[ -f \"$HOME/.aliases\" ] && source \"$HOME/.aliases\"".to_string()
}

/// A record of a skipped alias during import, with the reason it was skipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedAlias {
    pub name: String,
    pub reason: String,
}

/// Parse simple alias lines from shell config file content.
///
/// Only extracts lines matching `alias NAME='VALUE'` or `alias NAME="VALUE"` or `alias NAME=VALUE`.
/// Complex syntax (conditional aliases, functions, global aliases) are ignored.
pub fn parse_alias_lines(content: &str) -> Vec<(String, String)> {
    let mut aliases = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Must start with "alias "
        if !trimmed.starts_with("alias ") {
            continue;
        }

        let after_prefix = &trimmed[6..]; // strip "alias "

        // Find the = sign
        if let Some(eq_pos) = after_prefix.find('=') {
            let name = after_prefix[..eq_pos].trim().to_string();
            let raw_value = after_prefix[eq_pos + 1..].trim();

            // Skip if name is empty or invalid syntax
            if name.is_empty() || validate_alias_name(&name).is_err() {
                continue;
            }

            // Strip surrounding quotes from value
            let value = strip_quotes(raw_value);

            aliases.push((name, value));
        }
    }

    aliases
}

/// Strip surrounding single or double quotes from an alias value.
fn strip_quotes(s: &str) -> String {
    let bytes = s.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0] as char;
        let last = bytes[bytes.len() - 1] as char;

        if (first == '\'' && last == '\'') || (first == '"' && last == '"') {
            return String::from(&s[1..s.len() - 1]);
        }
    }
    s.to_string()
}

/// Merge imported aliases into an existing AliasStore, deduplicating by name.
///
/// Returns a `(imported_count, skipped)` tuple where `skipped` contains aliases
/// that were skipped during import along with the reason.
pub fn merge_imported_aliases(
    store: &AliasStore,
    imported: Vec<(String, String)>,
) -> (usize, Vec<SkippedAlias>) {
    let mut existing_names: HashMap<String, &AliasRecord> = HashMap::new();
    for record in &store.aliases {
        existing_names.insert(record.name.clone(), record);
    }

    let mut imported_count = 0;
    let mut skipped: Vec<SkippedAlias> = Vec::new();

    for (name, _command) in imported {
        // Skip protected names
        if is_protected_name(&name) {
            skipped.push(SkippedAlias {
                name: name.clone(),
                reason: "protected command name".to_string(),
            });
            continue;
        }

        // Skip if alias already exists in store
        if existing_names.contains_key(&name) {
            continue;
        }

        imported_count += 1;
    }

    (imported_count, skipped)
}

/// Build new AliasRecord entries from imported aliases that are not already in the store.
pub fn build_imported_records(
    store: &AliasStore,
    imported: Vec<(String, String)>,
    shell: AliasShell,
) -> (Vec<AliasRecord>, Vec<SkippedAlias>) {
    let (_, skipped) = merge_imported_aliases(store, imported.clone());

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut existing_names: HashMap<String, bool> = HashMap::new();
    for record in &store.aliases {
        existing_names.insert(record.name.clone(), true);
    }

    let mut records = Vec::new();
    for (name, command) in imported {
        if is_protected_name(&name) {
            continue;
        }
        if existing_names.contains_key(&name) {
            continue;
        }

        records.push(AliasRecord {
            name,
            command,
            description: None,
            tags: vec![],
            shell: shell.clone(),
            source: AliasSource::Imported,
            created_at: now,
            updated_at: now,
    modified_by_user: false,
        });
    }

    (records, skipped)
}

/// Check if the managed aliasman block already exists in config content.
pub fn has_managed_block(content: &str) -> bool {
    content.contains(MANAGED_BLOCK_START) && content.contains(MANAGED_BLOCK_END)
}

/// Insert the managed aliasman source block into shell config content.
///
/// If the block already exists, returns the content unchanged.
/// Otherwise appends the block at the end.
pub fn ensure_managed_block(content: &str) -> String {
    if has_managed_block(content) {
        return content.to_string();
    }

    let mut result = content.to_string();
    if !result.ends_with('\n') {
        result.push('\n');
    }
    result.push_str(&format!(
        "\n{start}\n{body}\n{end}\n",
        start = MANAGED_BLOCK_START,
        body = managed_block_body(),
        end = MANAGED_BLOCK_END
    ));

    result
}

/// Generate the managed block as a standalone string (for preview purposes).
pub fn render_managed_block() -> String {
    format!(
        "{start}\n{body}\n{end}",
        start = MANAGED_BLOCK_START,
        body = managed_block_body(),
        end = MANAGED_BLOCK_END
    )
}

/// Get the shell-specific reload hint for a given config file path.
pub fn get_reload_hint(config_path: &Path) -> String {
    let path_str = config_path.to_string_lossy();
    format!("source {}\n\nOr open a new terminal.", path_str)
}

/// Get the reload hint for the managed aliases file specifically.
pub fn get_aliases_reload_hint() -> String {
    "source $HOME/.aliases\n\nOr open a new terminal.".to_string()
}

/// Map ShellKind to AliasShell for record creation.
pub fn shell_kind_to_alias_shell(kind: ShellKind) -> AliasShell {
    match kind {
        ShellKind::Zsh => AliasShell::Zsh,
        ShellKind::Bash => AliasShell::Bash,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_aliases() {
        let content = r#"alias gs='git status'
alias ll='ls -la'
"#;
        let aliases = parse_alias_lines(content);
        assert_eq!(aliases.len(), 2);
        assert_eq!(aliases[0], ("gs".to_string(), "git status".to_string()));
        assert_eq!(aliases[1], ("ll".to_string(), "ls -la".to_string()));
    }

    #[test]
    fn parses_double_quoted_aliases() {
        let content = r#"alias gs="git status""#;
        let aliases = parse_alias_lines(content);
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].0, "gs");
        assert_eq!(aliases[0].1, "git status");
    }

    #[test]
    fn skips_non_alias_lines() {
        let content = r#"export PATH="/usr/bin:$PATH"
# This is a comment
alias gs='git status'

function my_func { echo hi; }
"#;
        let aliases = parse_alias_lines(content);
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].0, "gs");
    }

    #[test]
    fn skips_invalid_alias_names() {
        let content = r#"alias 1bad='echo one'
alias gs='git status'
"#;
        let aliases = parse_alias_lines(content);
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].0, "gs");
    }

    #[test]
    fn skips_protected_names_during_import() {
        let store = AliasStore::default();
        let imported = vec![
            ("gs".to_string(), "git status".to_string()),
            ("rm".to_string(), "rm -i".to_string()),
            ("sudo".to_string(), "sudo -A".to_string()),
        ];

        let (_, skipped) = merge_imported_aliases(&store, imported);
        assert_eq!(skipped.len(), 2);
        assert_eq!(skipped[0].name, "rm");
        assert_eq!(skipped[1].name, "sudo");
    }

    #[test]
    fn does_not_duplicate_existing_aliases() {
        let store = AliasStore {
            aliases: vec![AliasRecord {
                name: "gs".to_string(),
                command: "git status".to_string(),
                description: None,
                tags: vec![],
                shell: AliasShell::All,
                source: AliasSource::User,
                created_at: 100,
                updated_at: 100,
    modified_by_user: false,
            }],
        };

        let imported = vec![("gs".to_string(), "git status".to_string())];
        let (count, _) = merge_imported_aliases(&store, imported);
        assert_eq!(count, 0);
    }

    #[test]
    fn managed_block_insertion_is_idempotent() {
        let content = "# my shell config\nexport PATH=\"/usr/bin:$PATH\"\n";

        let once = ensure_managed_block(content);
        assert!(has_managed_block(&once));

        let twice = ensure_managed_block(&once);
        // Should be identical — block not duplicated
        assert_eq!(once, twice);
    }

    #[test]
    fn managed_block_not_added_if_already_present() {
        let content = format!(
            "# existing\n{start}\n{body}\n{end}\n",
            start = MANAGED_BLOCK_START,
            body = managed_block_body(),
            end = MANAGED_BLOCK_END
        );

        let result = ensure_managed_block(&content);
        assert_eq!(result, content);
    }

    #[test]
    fn render_managed_block_contains_source_line() {
        let block = render_managed_block();
        assert!(block.contains("source \"$HOME/.aliases\""));
        assert!(block.contains(MANAGED_BLOCK_START));
        assert!(block.contains(MANAGED_BLOCK_END));
    }

    #[test]
    fn reload_hint_contains_source_command() {
        let tmp = Path::new("/tmp/home");
        let hint = get_reload_hint(&tmp.join(".zshrc"));
        assert!(hint.contains("source /tmp/home/.zshrc"));
        assert!(hint.contains("new terminal"));
    }

    #[test]
    fn aliases_reload_hint() {
        let hint = get_aliases_reload_hint();
        assert!(hint.contains("source $HOME/.aliases"));
    }
}
