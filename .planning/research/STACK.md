# Technology Stack: aliasman

**Project:** aliasman — Rust CLI alias manager for macOS/Linux
**Researched:** 2026-05-10
**Confidence:** HIGH (all crates verified via crates.io + Context7 docs; hooks format verified from live ~/.claude/settings.json)

---

## 1. CLI Framework

### Recommendation: clap 4.6.1 with the `derive` feature

**Why:** clap is the unambiguous standard for Rust CLIs in 2025. It provides derive macros that turn annotated structs and enums directly into a fully-featured CLI — subcommands, named flags, help text, shell completions, and colored output are all handled with no boilerplate. The derive API matches aliasman's "named flags" requirement (`aliasman add --name gs --command "git status"`) directly.

```toml
[dependencies]
clap = { version = "4.6", features = ["derive", "color", "wrap_help"] }
```

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aliasman", version, about = "Shell alias manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new alias
    Add {
        #[arg(long)]
        name: String,
        #[arg(long)]
        command: String,
    },
    /// List all aliases
    List,
    /// Delete an alias
    Delete {
        #[arg(long)]
        name: String,
    },
    /// Suggest aliases from shell history
    Suggest,
    /// Show history statistics
    Stats,
    /// Install or remove the Claude Code hook
    Hook {
        #[arg(long)]
        install: bool,
        #[arg(long)]
        uninstall: bool,
    },
}
```

**What NOT to use:**

- `argh` — Google-internal style, limited features, no subcommand nesting, almost no community adoption outside Google tooling.
- `lexopt` — Low-level, no derive, hand-rolls all parsing; only appropriate when binary size is an extreme constraint (it isn't here).
- `structopt` — Deprecated; absorbed into clap 3+ as the derive feature. Using structopt in 2025 means a dead dependency.
- `pico-args` — Minimal, no subcommands, no help generation. For embedded-style CLIs only.

**Version source:** crates.io API, clap 4.6.1 released 2026-04-15.

---

## 2. History Parsing

### Recommendation: Hand-written parser using `std::fs` + `regex 1.12.3`

**Why:** There is no mature, maintained crate specifically for parsing zsh/bash history in Rust as of 2025. The crates that exist (`pxh`, `histat`, `ristory`) are end-user tools, not parser libraries. The formats are simple enough that a custom parser is the right call — and it avoids adding a dependency on an unmaintained or niche crate.

**Zsh extended history format** (when `HISTFILE` uses `EXTENDED_HISTORY`):
```
: 1700000000:0;git status
: 1700000001:2;cargo build --release
```
Format: `: <timestamp>:<elapsed>;<command>`

**Bash history format** (when `HISTTIMEFORMAT` is set):
```
#1700000000
git status
```
Or plain (no timestamps):
```
git status
cargo build
```

**Parser approach:**
```toml
[dependencies]
regex = "1.12"
```

```rust
use regex::Regex;

// Zsh extended history line
static ZSH_EXTENDED: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^: (\d+):\d+;(.+)$").unwrap()
});

// Bash timestamped history line
static BASH_TIMESTAMP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^#(\d+)$").unwrap()
});
```

Parse the file line by line, accumulate `(timestamp: Option<u64>, command: String)` pairs, then aggregate by command to compute frequency counts.

**What NOT to use:**

- `nom` — Powerful but heavyweight for this use case. zsh/bash history is line-oriented text, not a grammar requiring combinator parsing. Adds compile-time and cognitive overhead with no benefit.
- Any of the existing history CLIs as libraries — `pxh`, `histat`, etc. are not designed for library use and have no documented public API.

**History file locations to handle:**
- Zsh: `$HISTFILE` env var, fallback to `~/.zsh_history`
- Bash: `$HISTFILE` env var, fallback to `~/.bash_history`

---

## 3. File Manipulation (Safe Shell Config Editing)

### Recommendation: `tempfile 3.27.0` for atomic writes + `regex 1.12.3` for pattern matching

**Why:** The key risk in editing `~/.zshrc` or `~/.aliases` is data loss if the process crashes mid-write or the user interrupts. The write-then-rename (atomic replace) pattern prevents this: write to a temp file in the same directory, then `rename()` it over the target. `tempfile::NamedTempFile::persist()` implements exactly this.

```toml
[dependencies]
tempfile = "3.27"
regex = "1.12"
```

**Pattern for safe shell config editing:**

```rust
use tempfile::NamedTempFile;
use std::io::Write;
use std::path::Path;

fn write_aliases_file(path: &Path, content: &str) -> anyhow::Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    let mut tmp = NamedTempFile::new_in(dir)?;
    tmp.write_all(content.as_bytes())?;
    tmp.flush()?;
    tmp.persist(path)?;
    Ok(())
}
```

**Pattern for sourcing line injection:**

```rust
use regex::Regex;

fn ensure_source_line(rc_content: &str, aliases_path: &str) -> String {
    let marker = format!("source {}", aliases_path);
    if rc_content.contains(&marker) {
        rc_content.to_string()
    } else {
        format!("{}\n# Added by aliasman\n{}\n", rc_content.rstrip(), marker)
    }
}
```

**What NOT to use:**

- AST/grammar-based shell parsers (e.g., `shlex`, bash-parser ports) — overkill for the specific task of adding/removing a single line. Shell is not safely parseable in the general case; avoiding full parsing is intentional.
- `std::fs::write()` directly — Not atomic. If the process dies between truncating and writing, the file is corrupted.
- In-place file editing (opening with `OpenOptions::append` or seeking back) — Error-prone and not atomic.

**Alias file format:** Plain text, one alias per line, bash/zsh compatible:
```sh
alias gs='git status'
alias gb='git branch'
```
Parsing is straightforward with `lines()` and prefix matching on `"alias "`.

---

## 4. Claude Code Hooks API

### Verified from live `~/.claude/settings.json` and hook scripts on this machine.

**Hook registration format** (`~/.claude/settings.json`):

```json
{
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "/path/to/aliasman-hook"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "/path/to/aliasman-hook",
            "timeout": 5
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "/path/to/aliasman-hook",
            "timeout": 5
          }
        ]
      }
    ]
  }
}
```

**Hook event types (confirmed):**
- `SessionStart` — Fires when a Claude Code session begins. No matcher. Used for context injection.
- `PreToolUse` — Fires before a tool executes. Has `matcher` (tool name regex). Can block with `{"decision": "block", "reason": "..."}` + exit 2.
- `PostToolUse` — Fires after a tool executes. Has `matcher`. Advisory only (cannot undo).

**Hook input (stdin):** JSON object containing:
```json
{
  "session_id": "abc123",
  "cwd": "/Users/alice/project",
  "tool_name": "Bash",
  "tool_input": { "command": "git status" }
}
```
For `SessionStart`, `tool_name` and `tool_input` are absent.

**Hook output (stdout):** JSON object:
```json
{
  "hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": "## Your Aliases\nalias gs='git status'\n..."
  }
}
```
- For `SessionStart`: `hookEventName` must be `"SessionStart"`, `additionalContext` is injected into the session.
- For `PostToolUse`: `hookEventName` is `"PostToolUse"`, `additionalContext` is advisory text.
- For `PreToolUse` blocking: emit `{"decision": "block", "reason": "..."}` to stdout and exit 2.

**Hook exit codes:**
- `0` — Success / no-op
- `2` — Block (PreToolUse only); must also output `{"decision": "block", ...}` to stdout

**Silence rule:** Hooks must exit 0 and produce no output when they have nothing to say. Hooks that crash or produce malformed output are silently ignored by Claude Code.

**aliasman hook strategy (SessionStart):**

The hook binary should:
1. Read `~/.aliases` (the aliasman-managed file)
2. Detect the current working directory's project context (language, tools in use)
3. Score each alias for relevance to the current project
4. Emit the top N relevant aliases (not the entire file) as `additionalContext`
5. Keep output under ~500 tokens to respect Claude Code's context budget

**Modifying settings.json safely:**

Use `serde_json` to read the existing JSON, merge the hook entry, then write atomically with `tempfile`. Never clobber other hooks already installed.

```toml
[dependencies]
serde_json = "1.0"
serde = { version = "1.0", features = ["derive"] }
tempfile = "3.27"
```

---

## 5. Distribution (Homebrew)

### Recommendation: `cargo-dist 0.31.0` + GitHub Actions + personal Homebrew tap

**Why:** `cargo-dist` (by Axo) is the current standard tool for distributing Rust binaries via Homebrew in 2025. It generates GitHub Actions workflows that cross-compile for macOS (x86_64 + aarch64), Linux, and Windows, produces SHA256-verified tarballs, creates GitHub Releases, and pushes a Homebrew formula to a user-controlled tap repository.

**Setup:**

```bash
cargo install cargo-dist
dist init --installer homebrew --ci github
```

This generates `.github/workflows/release.yml` and configures `dist-workspace.toml`.

**`dist-workspace.toml` (or `Cargo.toml [workspace.metadata.dist]`):**

```toml
[dist]
cargo-dist-version = "0.31.0"
ci = ["github"]
installers = ["shell", "homebrew"]
tap = "YOUR_GITHUB_USERNAME/homebrew-aliasman"
publish-jobs = ["homebrew"]
targets = [
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-unknown-linux-gnu",
]
checksum = "sha256"
```

**Homebrew tap setup:**
1. Create a GitHub repository named `homebrew-aliasman` under your account.
2. Grant the GitHub Actions token write access to that repo (via a fine-grained PAT stored as `HOMEBREW_TAP_TOKEN` secret).
3. cargo-dist pushes the formula on each tagged release.

**Release trigger:**
```bash
git tag v0.1.0
git push --tags
# GitHub Actions runs dist plan → dist build → dist host → dist publish
# Results in: GitHub Release + Homebrew formula update
```

**User install:**
```bash
brew tap YOUR_USERNAME/aliasman
brew install aliasman
```

**`cargo install` secondary path:**
```bash
cargo install aliasman
```

**What NOT to use:**

- Manual formula writing — cargo-dist handles SHA computation, formula templating, and tap push automatically. Manual formulas are error-prone and require updating on every release.
- `goreleaser` — Go-specific; does not handle Rust cross-compilation.
- Submitting to homebrew-core — Requires prebuilt bottles and significant review time; not appropriate until the tool has proven adoption. Use a personal tap for v1.

---

## 6. Cross-Platform File Paths

### Recommendation: `dirs 6.0.0` for home directory resolution + `std::path::PathBuf` for path construction

**Why:** `dirs` (the `directories` crate family) resolves `home_dir()` correctly on macOS, Linux, and Windows using OS-appropriate conventions, without relying on the `HOME` environment variable (which can be unset or wrong in certain execution contexts like sudo or CI). It is the canonical solution used by virtually every Rust CLI tool that needs platform-aware paths.

```toml
[dependencies]
dirs = "6.0"
```

```rust
use std::path::PathBuf;

fn aliases_file() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
    Ok(home.join(".aliases"))
}

fn zshrc_path() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
    Ok(home.join(".zshrc"))
}

fn claude_settings_path() -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine home directory"))?;
    Ok(home.join(".claude").join("settings.json"))
}

fn zsh_history_path() -> PathBuf {
    std::env::var("HISTFILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_default()
                .join(".zsh_history")
        })
}
```

**Shell config file detection priority:**
1. Check `$SHELL` environment variable to determine the active shell.
2. For zsh: `~/.zshrc` (interactive) or `~/.zprofile` (login-only)
3. For bash: `~/.bashrc` (Linux) or `~/.bash_profile` (macOS, login shell)
4. Fall back to asking the user on first run via `dialoguer`

**What NOT to use:**

- Hardcoding `/Users/username/` — breaks on Linux and for non-standard macOS setups.
- `std::env::var("HOME")` directly — can be empty or wrong in privilege-escalated contexts (sudo, launchd services); `dirs::home_dir()` uses OS APIs instead.
- `shellexpand` — Useful for expanding `~` in user-provided strings, but not needed for path resolution in internal code where you control construction. Keep it on the shelf unless you need to expand user-input paths.

---

## Complete Dependency List

```toml
[dependencies]
# CLI framework
clap = { version = "4.6", features = ["derive", "color", "wrap_help"] }

# Error handling
anyhow = "1.0"
thiserror = "2.0"

# Serialization (for settings.json manipulation)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Cross-platform paths
dirs = "6.0"

# Safe file writes (atomic replace)
tempfile = "3.27"

# History and alias file parsing
regex = "1.12"

# Terminal output
colored = "3.1"

[dev-dependencies]
# Testing
assert_cmd = "2"
predicates = "3"
tempdir = "0.3"
```

**Crates intentionally excluded:**

| Crate | Reason Excluded |
|-------|----------------|
| `nom` | Overkill for line-oriented text; regex is sufficient |
| `toml` | Not needed; aliasman uses JSON (settings.json) and plain text (aliases) |
| `dialoguer` | Deferred to v2; v1 uses flags not interactive prompts |
| `indicatif` | No long-running operations in v1 requiring progress display |
| `shellexpand` | Not needed for internal path construction |
| `console` | `colored` is simpler for the terminal output needs here |
| `crossterm` | No TUI; pure CLI output only |

---

## Sources

- clap: https://docs.rs/clap/latest/clap/ (Context7 verified) — version 4.6.1 confirmed via crates.io
- cargo-dist: https://github.com/axodotdev/cargo-dist (Context7 verified) — version 0.31.0 confirmed via crates.io
- dirs: https://github.com/dirs/directories-rs (Context7 verified) — version 6.0.0 confirmed via crates.io
- tempfile: https://docs.rs/tempfile (Context7 verified) — version 3.27.0 confirmed via crates.io
- serde_json: https://docs.rs/serde_json (Context7 verified) — version 1.0.149 confirmed via crates.io
- anyhow: https://docs.rs/anyhow (Context7 verified) — version 1.0.102 confirmed via crates.io
- regex: https://docs.rs/regex (Context7 verified) — version 1.12.3 confirmed via crates.io
- Claude Code hooks format: verified directly from ~/.claude/settings.json and ~/.claude/hooks/ scripts on this machine (HIGH confidence — primary source)
- Hook input/output protocol: verified from gsd-context-monitor.js, gsd-session-state.sh, gsd-validate-commit.sh source code
