use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Detect the history file path from environment or defaults.
pub fn detect_history_file() -> Option<PathBuf> {
    // Try $HISTFILE first
    if let Ok(histfile) = std::env::var("HISTFILE") {
        if !histfile.is_empty() {
            return Some(PathBuf::from(histfile));
          }
      }

    // Fall back to $SHELL-based defaults
    if let Ok(shell) = std::env::var("SHELL") {
        if shell.contains("zsh") {
            if let Ok(home) = std::env::var("HOME") {
                let path = PathBuf::from(home).join(".zsh_history");
                if path.exists() {
                    return Some(path);
                  }
              }
          } else if shell.contains("bash") {
            if let Ok(home) = std::env::var("HOME") {
                let path = PathBuf::from(home).join(".bash_history");
                if path.exists() {
                    return Some(path);
                  }
              }
          }
      }

    None
}

/// Parse a zsh extended history line.
///
/// Format: `: <epoch>:<duration>;<command>`
/// Returns the command portion, or None if the line doesn't match.
pub fn parse_zsh_extended(line: &str) -> Option<String> {
    // Match pattern: ": <timestamp>:<duration>;<command>"
    if !line.starts_with(": ") {
        return None;
      }

    // Find the semicolon that separates the timestamp from the command
    let after_prefix = &line[2..]; // skip ": "
    if let Some(semi_pos) = after_prefix.find(";") {
        let command = after_prefix[semi_pos + 1..].trim().to_string();
        if !command.is_empty() {
            return Some(command);
          }
      }

    None
}

/// Parse a single history line, handling zsh extended format, bash timestamps, and plain commands.
pub fn parse_history_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
      }

    // Try zsh extended format first
    if let Some(cmd) = parse_zsh_extended(trimmed) {
        return Some(cmd);
      }

    // Skip bash timestamp lines like "#1715300000"
    if trimmed.starts_with('#') {
        return None;
      }

    // Plain command line
    Some(trimmed.to_string())
}

/// Read a history file and return parsed commands.
pub fn read_history_file(path: &Path) -> std::io::Result<Vec<String>> {
    let content = fs::read_to_string(path)?;
    let mut commands = Vec::new();

    for line in content.lines() {
        if let Some(cmd) = parse_history_line(line) {
            commands.push(cmd);
          }
      }

    Ok(commands)
}

/// Compute command frequencies from a list of commands.
/// Returns a vector of (command, count) sorted by descending count, then command text.
pub fn command_frequencies(commands: &[String]) -> Vec<(String, usize)> {
    let mut freq: HashMap<String, usize> = HashMap::new();

    for cmd in commands {
        *freq.entry(cmd.clone()).or_insert(0) += 1;
      }

    let mut result: Vec<(String, usize)> = freq.into_iter().collect();
    result.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    result
}

/// Check if a command contains risky shell metacharacters.
pub fn is_risky_history_command(cmd: &str) -> bool {
    let risky_patterns = [
        "$(",      // command substitution
        "`",       // backtick substitution
        "<(",      // process substitution
        "|",       // pipe
        ";",       // command separator
        "&&",      // and-chain
        "||",      // or-chain
    ];

    risky_patterns.iter().any(|p| cmd.contains(p))
}

/// Generate a suggested alias name from a command.
/// Extracts the first executable name and creates a short alias.
pub fn suggest_alias_name(cmd: &str) -> String {
    // Extract the first word (executable/command)
    let first_word = cmd.split_whitespace().next().unwrap_or(cmd);
    let base = first_word
         .split('/')
         .last()
         .unwrap_or(first_word);

    // Take first syllables/characters for short alias
    if base.len() <= 4 {
        base.to_lowercase()
     } else {
        // Take first 2-3 chars, handling common patterns
        let chars: Vec<char> = base.chars().take(3).collect();
        chars.iter().collect()
      }
}

/// A suggestion for creating an alias from a frequent command.
#[derive(Debug)]
pub struct Suggestion {
    pub alias_name: String,
    pub command: String,
    pub count: usize,
    pub is_risky: bool,
    pub reason: String,
}

/// Generate alias suggestions from command frequencies.
/// Only suggests for commands that are long enough or frequent enough.
pub fn generate_suggestions(
    frequencies: &[(String, usize)],
    min_length: usize,
    min_count: usize,
    existing_aliases: &[String],
) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();

    for (cmd, count) in frequencies {
        // Skip if command is too short or not frequent enough
        if cmd.len() < min_length && *count < min_count {
            continue;
          }

        // Skip if alias already exists
        let name = suggest_alias_name(cmd);
        if existing_aliases.iter().any(|a| a == &name) {
            continue;
          }

        let is_risky = is_risky_history_command(cmd);
        let reason = format!("Used {} times. Command length: {} chars.", count, cmd.len());

        suggestions.push(Suggestion {
            alias_name: name,
            command: cmd.clone(),
            count: *count,
            is_risky,
            reason,
          });
      }

    suggestions.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.alias_name.cmp(&b.alias_name)));
    suggestions
}

/// Format verbose stats output with percentages and grouping.
pub fn format_verbose_stats(frequencies: &[(String, usize)]) -> String {
    let total: usize = frequencies.iter().map(|(_, c)| c).sum();
    let mut output = String::new();

    output.push_str(&format!(
         "\nTotal commands analyzed: {}\n\n",
        total
     ));

    // Group by executable
    let mut groups: HashMap<String, Vec<(&String, &usize)>> = HashMap::new();
    for (cmd, count) in frequencies {
        let first_word = cmd.split_whitespace().next().unwrap_or(cmd);
        let base = first_word.split('/').last().unwrap_or(first_word).to_string();
        groups.entry(base).or_default().push((cmd, count));
      }

    let mut sorted_groups: Vec<_> = groups.into_iter().collect();
    sorted_groups.sort_by(|a, b| {
        let sum_a: usize = a.1.iter().map(|(_, c)| *c).sum();
        let sum_b: usize = b.1.iter().map(|(_, c)| *c).sum();
        sum_b.cmp(&sum_a)
      });

    output.push_str(&format!("{:<20} {:>8} {:>10} {:<40}\n", "Tool", "Count", "%", "Top Command"));
    output.push_str(&format!("{:-<20} {:-<8} {:-<10} {:-<40}\n", "", "", "", ""));

    for (tool, entries) in &sorted_groups {
        let group_total: usize = entries.iter().map(|(_, c)| *c).sum();
        let pct = if total > 0 {
            (group_total as f64 / total as f64 * 100.0) as i32
          } else {
            0
          };

        let top_cmd = entries
             .iter()
             .max_by_key(|(_, c)| *c)
             .map(|(c, _)| c.as_str())
             .unwrap_or("");

        output.push_str(&format!(
              "{:<20} {:>8} {:>9}% {:<40}\n",
            tool,
            group_total,
            pct,
            truncate_str(top_cmd, 40)
          ));
      }

    output
}

/// Format the default stats output as a simple Count/Command table.
pub fn format_stats(frequencies: &[(String, usize)]) -> String {
    let mut output = String::new();
    output.push_str(&format!("{:>8}  {:<}\n", "Count", "Command"));
    output.push_str(&format!("{:-<8}  {:-<}\n", "", ""));

    for (cmd, count) in frequencies {
        output.push_str(&format!("{:>8}  {:<}\n", count, cmd));
      }

    output
}

fn truncate_str(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
     } else {
        format!("{}...", &s[..max - 3])
      }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_zsh_extended_history_line() {
        let line = ": 1715300000:0;git status";
        let result = parse_zsh_extended(line);
        assert_eq!(result, Some("git status".to_string()));
    }

    #[test]
    fn rejects_non_extended_lines() {
        let result = parse_zsh_extended("git status");
        assert_eq!(result, None);
    }

    #[test]
    fn bash_timestamp_not_counted_as_command() {
        let result = parse_history_line("#1715300000");
        assert_eq!(result, None);
    }

    #[test]
    fn plain_command_parsed_directly() {
        let result = parse_history_line("git status");
        assert_eq!(result, Some("git status".to_string()));
    }

    #[test]
    fn command_frequencies_sorted_by_count() {
        let commands = vec![
            "git status".to_string(),
            "ls".to_string(),
            "git status".to_string(),
            "ls".to_string(),
            "ls".to_string(),
          ];
        let freq = command_frequencies(&commands);
        assert_eq!(freq[0], ("ls".to_string(), 3));
        assert_eq!(freq[1], ("git status".to_string(), 2));
    }

    #[test]
    fn flags_command_substitution_as_risky() {
        assert!(is_risky_history_command("echo $(date)"));
    }

    #[test]
    fn flags_shell_chaining_as_risky() {
        assert!(is_risky_history_command("ls && rm -rf /tmp/test"));
        assert!(is_risky_history_command("cat file | grep foo"));
        assert!(is_risky_history_command("ls; echo done"));
    }

    #[test]
    fn simple_command_not_risky() {
        assert!(!is_risky_history_command("git status"));
        assert!(!is_risky_history_command("ls -la"));
    }

    #[test]
    fn suggestions_are_display_only_by_default() {
        let commands = vec![
            "cargo build --release".to_string(),
            "cargo build --release".to_string(),
            "cargo build --release".to_string(),
            "ls".to_string(),
          ];
        let freq = command_frequencies(&commands);
        let suggestions = generate_suggestions(&freq, 15, 2, &[]);

           // Should have suggestions but not modify anything
        assert!(!suggestions.is_empty());
        for s in &suggestions {
            assert_eq!(s.command, "cargo build --release");
          }
    }

    #[test]
    fn risky_suggestion_flagged() {
        let commands = vec![
            "echo $(pwd) && ls".to_string(),
            "echo $(pwd) && ls".to_string(),
          ];
        let freq = command_frequencies(&commands);
        let suggestions = generate_suggestions(&freq, 5, 1, &[]);

        assert!(!suggestions.is_empty());
        assert!(suggestions[0].is_risky);
    }

    #[test]
    fn suggest_alias_name_extraction() {
        let name = suggest_alias_name("cargo build --release");
        assert_eq!(name, "car");

        let name = suggest_alias_name("git status -s");
        assert_eq!(name, "git");
    }
}
