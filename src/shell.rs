use std::path::{Path, PathBuf};

/// The kind of shell being managed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellKind {
    Zsh,
    Bash,
}

/// Result of shell detection when the shell cannot be unambiguously determined.
#[derive(Debug, PartialEq, Eq)]
pub enum DetectResult {
    /// Shell and config file identified.
    Found(ShellKind, PathBuf),
    /// No shell signals found — user must choose.
    Ambiguous,
}

/// Detect the shell kind from a shell executable path (e.g., the value of `$SHELL`).
pub fn detect_shell_from_path(shell_path: &str) -> Option<ShellKind> {
    let filename = Path::new(shell_path).file_name().and_then(|f| f.to_str());

    match filename {
        Some("zsh") => Some(ShellKind::Zsh),
        Some("bash") => Some(ShellKind::Bash),
        _ => None,
    }
}

/// Select the appropriate shell config file given a shell kind and an injectable home directory.
///
/// - zsh → `.zshrc`
/// - bash → `.bash_profile` if it exists, otherwise `.bashrc`
pub fn select_shell_config(home: &Path, kind: ShellKind) -> Option<PathBuf> {
    match kind {
        ShellKind::Zsh => {
            let path = home.join(".zshrc");
            if path.exists() {
                Some(path)
            } else {
                Some(path) // return path even if it doesn't exist yet — init will create it
            }
        }
        ShellKind::Bash => {
            let profile = home.join(".bash_profile");
            if profile.exists() {
                Some(profile)
            } else {
                let rc = home.join(".bashrc");
                Some(rc)
            }
        }
    }
}

/// Attempt full detection: try env shell path first, then fall back to existing config files.
///
/// `home` is the injectable home directory root for tests.
/// `shell_env` is the value of `$SHELL` (or empty string if unset).
pub fn detect_shell_and_config(home: &Path, shell_env: &str) -> DetectResult {
    // 1. Try $SHELL
    if !shell_env.is_empty() {
        if let Some(kind) = detect_shell_from_path(shell_env) {
            if let Some(config) = select_shell_config(home, kind) {
                return DetectResult::Found(kind, config);
            }
        }
    }

    // 2. Fall back to existing config files
    let zshrc = home.join(".zshrc");
    let bash_profile = home.join(".bash_profile");
    let bashrc = home.join(".bashrc");

    let zsh_exists = zshrc.exists();
    let bash_profile_exists = bash_profile.exists();
    let bashrc_exists = bashrc.exists();

    let shell_signals_count = [zsh_exists, bash_profile_exists, bashrc_exists]
        .iter()
        .filter(|b| **b)
        .count();

    match shell_signals_count {
        0 => DetectResult::Ambiguous,
        1 => {
            if zsh_exists {
                DetectResult::Found(ShellKind::Zsh, zshrc)
            } else if bash_profile_exists {
                DetectResult::Found(ShellKind::Bash, bash_profile)
            } else {
                DetectResult::Found(ShellKind::Bash, bashrc)
            }
        }
        _ => DetectResult::Ambiguous, // conflicting signals
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn detects_zsh_from_path() {
        assert_eq!(detect_shell_from_path("/bin/zsh"), Some(ShellKind::Zsh));
        assert_eq!(
            detect_shell_from_path("/usr/local/bin/zsh"),
            Some(ShellKind::Zsh)
        );
    }

    #[test]
    fn detects_bash_from_path() {
        assert_eq!(detect_shell_from_path("/bin/bash"), Some(ShellKind::Bash));
    }

    #[test]
    fn unknown_shell_returns_none() {
        assert_eq!(detect_shell_from_path("/bin/fish"), None);
        assert_eq!(detect_shell_from_path(""), None);
    }

    #[test]
    fn selects_zshrc_for_zsh() {
        let tmp = tempfile::tempdir().unwrap();
        let config = select_shell_config(tmp.path(), ShellKind::Zsh);
        assert!(config.is_some());
        assert_eq!(config.unwrap().file_name().unwrap(), ".zshrc");
    }

    #[test]
    fn prefers_bash_profile_for_bash_when_exists() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join(".bash_profile"), "# existing\n").unwrap();
        fs::write(tmp.path().join(".bashrc"), "# existing\n").unwrap();

        let config = select_shell_config(tmp.path(), ShellKind::Bash).unwrap();
        assert_eq!(config.file_name().unwrap(), ".bash_profile");
    }

    #[test]
    fn falls_back_to_bashrc_when_no_bash_profile() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join(".bashrc"), "# existing\n").unwrap();

        let config = select_shell_config(tmp.path(), ShellKind::Bash).unwrap();
        assert_eq!(config.file_name().unwrap(), ".bashrc");
    }

    #[test]
    fn detect_with_zsh_shell_env() {
        let tmp = tempfile::tempdir().unwrap();
        let result = detect_shell_and_config(tmp.path(), "/bin/zsh");
        assert_eq!(
            result,
            DetectResult::Found(ShellKind::Zsh, tmp.path().join(".zshrc"))
        );
    }

    #[test]
    fn detect_with_bash_shell_env() {
        let tmp = tempfile::tempdir().unwrap();
        let result = detect_shell_and_config(tmp.path(), "/bin/bash");
        // No .bash_profile exists, so it picks .bashrc
        assert_eq!(
            result,
            DetectResult::Found(ShellKind::Bash, tmp.path().join(".bashrc"))
        );
    }

    #[test]
    fn detect_fallback_to_existing_zshrc() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join(".zshrc"), "# content\n").unwrap();
        let result = detect_shell_and_config(tmp.path(), "");
        assert_eq!(
            result,
            DetectResult::Found(ShellKind::Zsh, tmp.path().join(".zshrc"))
        );
    }

    #[test]
    fn detect_ambiguous_when_multiple_configs_exist() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join(".zshrc"), "# zsh\n").unwrap();
        fs::write(tmp.path().join(".bashrc"), "# bash\n").unwrap();
        let result = detect_shell_and_config(tmp.path(), "");
        assert_eq!(result, DetectResult::Ambiguous);
    }

    #[test]
    fn detect_ambiguous_when_no_config_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let result = detect_shell_and_config(tmp.path(), "");
        assert_eq!(result, DetectResult::Ambiguous);
    }
}
