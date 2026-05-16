use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::model::{AliasRecord, AliasSource};
use crate::store::{backup_file, write_atomic, AliasStore};

/// The default token budget for hook-injected alias context.
pub const DEFAULT_MAX_TOKENS: usize = 500;

/// Approximate characters per token (4 chars ≈ 1 token).
const CHARS_PER_TOKEN: usize = 4;

/// Claude settings.json structure for merging.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaudeSettings {
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub other: HashMap<String, serde_json::Value>,
}

/// The hook entry we add to Claude settings.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HookEntry {
    pub command: String,
}

/// Preview of what hook install will do.
pub struct InstallPreview {
    pub settings_path: PathBuf,
    pub backup_path: Option<PathBuf>,
    pub hook_command: String,
    pub has_existing_hooks: bool,
    pub has_existing_settings: bool,
}

/// Check if settings.json already has the aliasman hook.
pub fn has_aliasman_hook(settings: &ClaudeSettings) -> bool {
    if let Some(hooks) = settings.other.get("hooks") {
        if let Some(session_start) = hooks.get("SessionStart") {
            if let Some(cmd) = session_start.get("command") {
                if let Some(cmd_str) = cmd.as_str() {
                    return cmd_str.contains("aliasman");
                   }
               }
           }
       }
    false
}

/// Merge the aliasman hook into Claude settings, preserving all other settings.
pub fn merge_aliasman_hook(
    settings: &ClaudeSettings,
    hook_command: &str,
) -> ClaudeSettings {
    let mut new_settings = settings.clone();

     // Ensure hooks object exists
    let hooks = new_settings
         .other
         .entry("hooks".to_string())
         .or_insert_with(|| serde_json::json!({}));

     // Set SessionStart hook
    *hooks = serde_json::json!({
        "SessionStart": {
            "command": hook_command,
            "success": true,
            "suppressOutput": true
        }
    });

    new_settings
}

/// Create an install preview without writing anything.
pub fn create_install_preview(
    settings_path: &Path,
    aliasman_bin: &Path,
) -> Result<InstallPreview, Box<dyn std::error::Error>> {
    let hook_command = format!(
          "{} hook --shell claude",
        aliasman_bin.display()
       );

    let has_existing = settings_path.exists();
    let backup_path = if has_existing {
        Some(backup_file(settings_path)?)
       } else {
        None
       };

    let has_existing_hooks = if has_existing {
        let content = fs::read_to_string(settings_path)?;
        if let Ok(settings) = serde_json::from_str::<ClaudeSettings>(&content) {
            settings.other.contains_key("hooks")
           } else {
            false
           }
       } else {
        false
       };

    Ok(InstallPreview {
        settings_path: settings_path.to_path_buf(),
        backup_path,
        hook_command,
        has_existing_hooks,
        has_existing_settings: has_existing,
       })
}

/// Install the Claude hook by merging into settings.json atomically.
pub fn install_claude_hook(
    settings_path: &Path,
    aliasman_bin: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let hook_command = format!("{} hook --shell claude", aliasman_bin.display());

     // Load existing settings or create empty
    let settings = if settings_path.exists() {
        let content = fs::read_to_string(settings_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| ClaudeSettings {
            other: HashMap::new(),
           })
       } else {
        ClaudeSettings { other: HashMap::new() }
       };

     // Backup if exists
    if settings_path.exists() {
        let bp = backup_file(settings_path)?;
        eprintln!("Backup created: {}", bp.display());
       }

     // Merge hook
    let merged = merge_aliasman_hook(&settings, &hook_command);

     // Ensure parent directory
    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent)?;
       }

     // Atomic write
    let json_out = serde_json::to_string_pretty(&merged)?;
    write_atomic(settings_path, &json_out)?;

    Ok(())
}

/// Scan project directory for context signals (e.g., Cargo.toml → git/cargo aliases).
pub fn detect_project_context(cwd: &Path) -> Vec<String> {
    let mut signals = Vec::new();

    let markers = [
         ("Cargo.toml", "rust"),
         ("Cargo.lock", "rust"),
         ("package.json", "node"),
         ("yarn.lock", "node"),
         ("pnpm-lock.yaml", "node"),
         ("Dockerfile", "docker"),
         ("docker-compose.yml", "docker"),
         ("docker-compose.yaml", "docker"),
         (".git", "git"),
         ("requirements.txt", "python"),
         ("pyproject.toml", "python"),
         ("Gemfile", "ruby"),
         ("go.mod", "go"),
         ("Makefile", "make"),
       ];

    for (file, tag) in &markers {
        if cwd.join(file).exists() {
            signals.push(tag.to_string());
           }
       }

    signals
}

/// Score an alias against project context signals. Higher = more relevant.
pub fn score_alias(record: &AliasRecord, signals: &[String]) -> f64 {
    let mut score = 1.0;

     // Tag matching
    for tag in &record.tags {
        for signal in signals {
            if tag.to_lowercase().contains(signal) || signal.contains(&tag.to_lowercase()) {
                score += 5.0;
               }
           }
       }

     // Command matching (check if command starts with a signal tool)
    let first_word = record.command.split_whitespace().next().unwrap_or("");
    for signal in signals {
        if first_word.to_lowercase().contains(signal) {
            score += 3.0;
           }
       }

     // Source bonus: user-created aliases score higher than imported/suggested
    match record.source {
        AliasSource::User => score += 2.0,
        AliasSource::Imported => score += 1.0,
        AliasSource::Suggested => {}
       }

    score
}

/// Filter and rank aliases by relevance to project context, respecting token budget.
pub fn get_relevant_aliases(
    store: &AliasStore,
    cwd: &Path,
    max_tokens: Option<usize>,
) -> Vec<(AliasRecord, f64)> {
    let budget = max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);
    let signals = detect_project_context(cwd);

     // Score all aliases
    let mut scored: Vec<(AliasRecord, f64)> = Vec::new();
    for record in &store.aliases {
        let score = score_alias(record, &signals);
        scored.push((record.clone(), score));
       }

     // Sort by score descending, then name for determinism
    scored.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
              .unwrap_or(std::cmp::Ordering::Equal)
              .then_with(|| a.0.name.cmp(&b.0.name))
       });

     // Apply token budget: estimate tokens from rendered context
    let mut result = Vec::new();
    let mut tokens_used: usize = 0;
     // Header takes ~20 tokens
    let header_cost = 20;

    for (record, score) in scored {
         // Estimate: each alias line ≈ (name.len() + command.len()) / CHARS_PER_TOKEN + 5
        let line_cost = (record.name.len() + record.command.len()) / CHARS_PER_TOKEN + 5;
        if header_cost + tokens_used + line_cost > budget {
            break;
           }
        tokens_used += line_cost;
        result.push((record, score));
       }

    result
}

/// Format alias context as markdown for Claude's additionalContext.
pub fn format_alias_context(aliases: &[(AliasRecord, f64)]) -> String {
    if aliases.is_empty() {
        return String::new();
       }

    let mut md = String::from("## Active Aliases\n\n");
    md.push_str("| Name | Command |\n");
    md.push_str("|------|--------|\n");

    for (record, _score) in aliases {
        md.push_str(&format!("| {} | `{}` |\n", record.name, truncate(&record.command, 50)));
       }

    md
}

/// The hook output JSON structure for Claude SessionStart.
#[derive(Debug, Serialize)]
pub struct HookOutput {
    pub continue: bool,
    pub suppress_output: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_context: Option<String>,
}

impl HookOutput {
    pub fn new(context: Option<String>) -> Self {
        HookOutput {
             continue: true,
            suppress_output: true,
            additional_context: context,
           }
    }
}

/// Run the Claude hook: read stdin, load aliases, filter, emit JSON.
///
/// On no aliases, no matches, or recoverable errors: exit 0 with no stdout.
pub fn run_claude_hook(
    store: &AliasStore,
    cwd: &Path,
    max_tokens: Option<usize>,
) -> Result<Option<HookOutput>, Box<dyn std::error::Error>> {
    if store.aliases.is_empty() {
        return Ok(None);
       }

    let aliases = get_relevant_aliases(store, cwd, max_tokens);
    if aliases.is_empty() {
        return Ok(None);
       }

    let context = format_alias_context(&aliases);
    if context.is_empty() {
        return Ok(None);
       }

    Ok(Some(HookOutput::new(Some(context))))
}

/// Read hook stdin JSON (for future extensibility — currently not used by Claude).
pub fn read_hook_stdin() -> Option<serde_json::Value> {
    let mut buf = String::new();
    if io::stdin().read_to_string(&mut buf).is_err() {
        return None;
       }
    serde_json::from_str(&buf).ok()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
       } else {
        format!("{}...", &s[..max - 3])
       }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AliasShell, AliasSource};

    fn make_record(name: &str, command: &str, source: AliasSource, tags: &[&str]) -> AliasRecord {
        AliasRecord {
             name: name.to_string(),
            command: command.to_string(),
            description: None,
            tags: tags.iter().map(|t| t.to_string()).collect(),
            shell: AliasShell::All,
            source,
            created_at: 1715300000,
            updated_at: 1715300000,
           }
    }

     #[test]
    fn detects_rust_project_context() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("Cargo.toml"), "[package]\n").unwrap();

        let signals = detect_project_context(tmp.path());
        assert!(signals.contains(&"rust".to_string()));
    }

     #[test]
    fn detects_node_project_context() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("package.json"), "{}\n").unwrap();

        let signals = detect_project_context(tmp.path());
        assert!(signals.contains(&"node".to_string()));
    }

     #[test]
    fn detects_docker_project_context() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("Dockerfile"), "FROM alpine\n").unwrap();

        let signals = detect_project_context(tmp.path());
        assert!(signals.contains(&"docker".to_string()));
    }

     #[test]
    fn detects_git_project_context() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join(".git")).unwrap();

        let signals = detect_project_context(tmp.path());
        assert!(signals.contains(&"git".to_string()));
    }

     #[test]
    fn cargo_project_includes_cargo_tagged_aliases() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("Cargo.toml"), "[package]\n").unwrap();

        let store = AliasStore {
            aliases: vec![
                make_record("cb", "cargo build", AliasSource::User, &["cargo", "rust"]),
                make_record("gs", "git status", AliasSource::User, &["git"]),
                make_record("ll", "ls -la", AliasSource::User, &[]),
               ],
           };

        let result = get_relevant_aliases(&store, tmp.path(), None);
         // cargo-tagged alias should rank higher than generic
        assert!(result[0].0.name == "cb" || result[0].1 > result.iter().find(|(r, _)| r.name == "ll").unwrap().1);
    }

     #[test]
    fn budget_limits_output_length() {
        let tmp = tempfile::tempdir().unwrap();

        let mut aliases = Vec::new();
        for i in 0..20 {
            aliases.push(make_record(
                 &format!("alias_{}", i),
                 &format!("echo this is a very long command number {}", i),
                AliasSource::User,
                &[],
               ));
           }

        let store = AliasStore { aliases };
        let result = get_relevant_aliases(&store, tmp.path(), Some(50));

        assert!(result.len() < 20, "budget should limit results, got {}", result.len());
    }

     #[test]
    fn empty_store_returns_no_hook_output() {
        let tmp = tempfile::tempdir().unwrap();
        let store = AliasStore::default();

        let result = run_claude_hook(&store, tmp.path(), None).unwrap();
        assert!(result.is_none());
    }

     #[test]
    fn hook_output_serializes_to_json() {
        let output = HookOutput::new(Some("test context".to_string()));
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("continue"));
        assert!(json.contains("suppressOutput"));
        assert!(json.contains("additionalContext"));
    }

     #[test]
    fn has_aliasman_hook_detects_existing() {
        let settings = ClaudeSettings {
            other: HashMap::from_iter(vec![(
                 "hooks".to_string(),
                serde_json::json!({"SessionStart": {"command": "/usr/local/bin/aliasman hook --shell claude"}}),
               )]),
           };
        assert!(has_aliasman_hook(&settings));
    }

     #[test]
    fn has_aliasman_hook_returns_false_for_other_hooks() {
        let settings = ClaudeSettings {
            other: HashMap::from_iter(vec![(
                 "hooks".to_string(),
                serde_json::json!({"SessionStart": {"command": "echo hello"}}),
               )]),
           };
        assert!(!has_aliasman_hook(&settings));
    }

     #[test]
    fn merge_preserves_unrelated_settings() {
        let mut other = HashMap::new();
        other.insert("theme".to_string(), serde_json::json!("dark"));
        other.insert("model".to_string(), serde_json::json!("sonnet"));

        let settings = ClaudeSettings { other };
        let merged = merge_aliasman_hook(&settings, "/path/to/aliasman hook --shell claude");

        assert_eq!(merged.other.get("theme").unwrap(), &serde_json::json!("dark"));
        assert_eq!(merged.other.get("model").unwrap(), &serde_json::json!("sonnet"));
        assert!(merged.other.contains_key("hooks"));
    }

     #[test]
    fn format_alias_context_returns_markdown_table() {
        let aliases = vec![(
             AliasRecord {
                name: "gs".to_string(),
                command: "git status".to_string(),
                description: None,
                tags: vec![],
                shell: AliasShell::All,
                source: AliasSource::User,
                created_at: 0,
                updated_at: 0,
               },
            5.0,
           )];

        let md = format_alias_context(&aliases);
        assert!(md.contains("| Name | Command |"));
        assert!(md.contains("| gs | `git status` |"));
    }

     #[test]
    fn install_does_not_duplicate_hook() {
        let tmp = tempfile::tempdir().unwrap();
        let settings_path = tmp.path().join(".claude").join("settings.json");
        fs::create_dir_all(tmp.path().join(".claude")).unwrap();

        // Write initial settings with hook
        let initial = ClaudeSettings {
            other: HashMap::from_iter(vec![(
                 "hooks".to_string(),
                serde_json::json!({"SessionStart": {"command": "/aliasman hook --shell claude"}}),
               )]),
           };
        fs::write(&settings_path, serde_json::to_string_pretty(&initial).unwrap()).unwrap();

        // Install again
        let fake_bin = PathBuf::from("/fake/aliasman");
        install_claude_hook(&settings_path, &fake_bin).unwrap();

        // Read back and check hook appears only once
        let content = fs::read_to_string(&settings_path).unwrap();
        assert_eq!(
             content.matches("aliasman"),
            1,
            "hook should appear exactly once: {}",
            content
           );
    }
}
