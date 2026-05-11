# Architecture Patterns — aliasman

**Domain:** Rust CLI tool — shell alias manager with Claude Code integration
**Researched:** 2026-05-10
**Confidence:** HIGH (verified via Context7 for tempfile/dirs/zoxide/atuin, direct inspection of Claude Code hook protocol, live shell environment)

---

## Component Boundary Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         aliasman binary                         │
│                                                                 │
│  ┌───────────┐   ┌──────────────┐   ┌────────────────────────┐ │
│  │  CLI      │   │  AliasStore  │   │  ShellIntegration      │ │
│  │  (clap)   │──▶│  (TOML r/w)  │   │  (read/write configs)  │ │
│  │           │   │  ~/.aliases  │   │  ~/.zshrc / ~/.bashrc  │ │
│  └─────┬─────┘   └──────────────┘   └───────────┬────────────┘ │
│        │                                         │             │
│        │         ┌──────────────┐   ┌────────────▼────────────┐ │
│        ├────────▶│  HistoryEngine│   │  ShellDetector         │ │
│        │         │  (parse zsh/ │   │  ($SHELL, config files) │ │
│        │         │   bash hist) │   └────────────────────────┘ │
│        │         └──────────────┘                              │
│        │                                                       │
│        │         ┌──────────────────────────────────────────┐  │
│        └────────▶│  HookRunner  (aliasman hook --shell claude│  │
│                  │  reads AliasStore + cwd context,          │  │
│                  │  outputs JSON {additionalContext: "..."}  │  │
│                  └──────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘

External surfaces:
  ~/.aliases              ← AliasStore owns this file (atomic writes)
  ~/.zshrc / ~/.bashrc    ← ShellIntegration appends ONE source line
  ~/.config/aliasman/     ← App config (TOML), managed by ConfigStore
  ~/.claude/settings.json ← HookRunner registers itself here on install
```

---

## Data Model

### The Alias Record

```toml
[[aliases]]
name        = "gs"
command     = "git status"
description = "Quick git status"          # optional, free text
tags        = ["git", "vcs"]              # optional, for filtering
shell       = "all"                       # "all" | "zsh" | "bash"
created_at  = 1715300000                  # Unix timestamp (u64)
updated_at  = 1715300000
source      = "user"                      # "user" | "imported" | "suggested"
```

**Field rationale:**

- `name` — the alias identifier, must match `[a-zA-Z_][a-zA-Z0-9_-]*`. Single string, no spaces.
- `command` — the expansion. Stored verbatim. May contain shell special chars; written to the aliases file with single-quote escaping.
- `description` — free text shown in `aliasman list`. Not written to the shell file. Kept in the TOML store only.
- `tags` — for smart filtering in the Claude hook. No enforced taxonomy in v1 — user-assigned strings.
- `shell` — default `"all"`. Shell-specific aliases (`"zsh"` only) are emitted only to the matching shell's source block. In v1 with a single `~/.aliases` file this is effectively unused but reserves the field for v2 multi-shell support.
- `created_at` / `updated_at` — Unix timestamps as u64. Enables `aliasman list --recent` and future analytics.
- `source` — provenance tracking. Imported aliases from first-run scan get `"imported"`. History suggestions get `"suggested"`. User-created get `"user"`.

### The AliasStore File

Location: `~/.aliases` (managed file, written by aliasman).

Written format:

```sh
# aliasman managed — do not edit manually
# Run `aliasman list` to view, `aliasman edit <name>` to change

alias gs='git status'
alias gco='git checkout'
alias ll='ls -la'
```

No per-alias metadata comments in the shell file. Metadata lives in `~/.config/aliasman/aliases.toml`. The shell file is derived output — regenerated on every write.

### Config/State File

Location: `~/.config/aliasman/config.toml`

```toml
[aliasman]
version = "1"
shell   = "zsh"          # detected on install, can be overridden

[aliases_file]
path = "/Users/alice/.aliases"   # default ~/.aliases, user can relocate

[hook]
enabled     = true
max_tokens  = 800        # approximate token budget for alias injection
filter_mode = "smart"    # "smart" | "all" | "tagged:<tag>"

[history]
zsh_histfile  = "/Users/alice/.zsh_history"   # resolved at first run
bash_histfile = "/Users/alice/.bash_history"
```

Location rationale: `~/.config/aliasman/` follows the XDG pattern. On macOS, `dirs::config_dir()` returns `~/Library/Application Support/` (XDG-incompatible), but the `directories` crate provides `ProjectDirs::config_dir()` which returns `~/Library/Application Support/aliasman` on macOS and `~/.config/aliasman` on Linux. Use `~/.config/aliasman/` as a hard-coded override so behaviour is consistent across macOS and Linux — this matches what atuin and zoxide do.

The alias data itself lives at `~/.config/aliasman/aliases.toml` — same directory as config, keeping all app state colocated.

---

## File Management Strategy

### Atomic Write Protocol (HIGH confidence — tempfile crate, verified via Context7)

Never write shell config files or the aliases file in place. The pattern used by all shell tooling (conda, nvm, mamba):

```
1. Read current file into memory
2. Build new content string
3. Write to NamedTempFile in the SAME directory as the target
   (same filesystem, so rename is atomic)
4. tempfile.flush()
5. tempfile.persist(target_path)   ← atomic rename, replaces target
```

The `tempfile` crate (`NamedTempFile::persist`) provides this. It is a cross-filesystem atomic rename — on failure, the temp file is cleaned up automatically.

Critical: the temp file must be on the same filesystem as the target (same directory). Writing to `/tmp` when the target is `~/.aliases` is wrong if `/tmp` is a tmpfs.

### Backup Strategy

Before any destructive write to a shell config file (`~/.zshrc`, `~/.bashrc`, `~/.bash_profile`), create a timestamped backup:

```
~/.zshrc.aliasman-backup-2026-05-10T14-30-00
```

Keep the last 3 backups per file (purge older ones on write). Do not backup `~/.aliases` — it is fully regenerated and the canonical data is `aliases.toml`.

### Marker Comments for Shell Config Injection

When adding the `source ~/.aliases` line to a shell config, wrap it in managed-block markers:

```sh
# >>> aliasman >>>
[ -f "$HOME/.aliases" ] && source "$HOME/.aliases"
# <<< aliasman <<<
```

This pattern (used by conda, mamba) makes the block idempotent to re-add, easy to remove (`aliasman uninstall`), and greppable.

On install: scan for existing marker. If present, skip. If absent, append the block.
On uninstall: scan for marker, remove exactly the lines between and including markers.

### Writing the ~/.aliases File

Regenerate completely on every alias change. The file is derived from `aliases.toml` — no partial updates. This avoids drift between the TOML store and the shell file.

Write order in the generated file: alphabetical by name, with a header comment. Shell-specific aliases are emitted in a guarded block if needed (v2 concern).

Single-quote every alias expansion to avoid shell substitution:

```sh
alias gs='git status'
```

If the command itself contains single quotes, use the `$'...'` ANSI-C quoting form:

```sh
alias greeting=$'echo \'hello world\''
```

---

## Claude Hook Architecture

### How Claude Code Hooks Work (HIGH confidence — verified from live ~/.claude/settings.json and hook source)

Claude Code hooks are commands registered in `~/.claude/settings.json` under event keys (`SessionStart`, `PreToolUse`, `PostToolUse`). On `SessionStart`, Claude Code runs each registered command and collects their stdout. The stdout must be valid JSON:

```json
{
  "hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": "## Your Aliases\nalias gs='git status'\n..."
  }
}
```

The `additionalContext` string is injected verbatim into the Claude session as context. This is the mechanism aliasman uses.

### Hook Registration

`aliasman install-hook` modifies `~/.claude/settings.json`:

1. Read and parse the JSON file.
2. Add an entry under `hooks.SessionStart`:
   ```json
   {
     "hooks": [
       {
         "type": "command",
         "command": "/absolute/path/to/aliasman hook --shell claude"
       }
     ]
   }
   ```
3. Write back atomically (tempfile + rename).

Use the absolute path to the binary (resolved via `std::env::current_exe()` or the path the user installed to). Do not rely on `$PATH` being available when Claude Code runs the hook.

### What aliasman hook --shell claude Outputs

The hook command outputs JSON to stdout. It must complete quickly (under 5 seconds — Claude Code has a hook timeout, observed as 10s in existing hooks).

```json
{
  "hookSpecificOutput": {
    "hookEventName": "SessionStart",
    "additionalContext": "## Shell Aliases (aliasman)\n\nRelevant to this project:\n- gs → git status\n- gco → git checkout\n- dk → docker\n\nUse these aliases in any shell commands you suggest.\n"
  }
}
```

Plain Markdown inside `additionalContext`. Not JSON-within-JSON for the alias list — just readable prose Claude can parse directly.

### Smart Filtering Logic

The hook reads the current working directory (from `$PWD` or `std::env::current_dir()`) and the alias store, then selects which aliases to inject.

Filtering signals (in order of weight):

1. **Tag match** — if the cwd path contains keywords matching alias tags (e.g., cwd contains `git` repo markers like `.git/`, emit aliases tagged `"git"`). Check for `.git/` → include git aliases; `docker-compose.yml` or `Dockerfile` → include docker aliases; `Cargo.toml` → include cargo/rust aliases.
2. **Recency** — aliases updated or created in the last 7 days are likely in active use.
3. **Frequency hint** — if `source = "suggested"`, the alias came from a high-frequency history command, likely relevant anywhere.
4. **Token budget** — the `max_tokens` config (default 800) caps output. Each alias line is approximately 10-15 tokens. At 800 tokens, inject ~50 aliases maximum. Count conservatively (1 token per 4 chars). Stop when budget is reached.

Algorithm:

```
1. Load all aliases from aliases.toml
2. Score each alias:
   - +3 if tags match cwd signals
   - +2 if updated_at within 7 days
   - +1 if source == "suggested"
   - +0 base score
3. Sort descending by score
4. Emit top N aliases that fit within token budget
5. Always include aliases with score >= 3 even if over budget (cap at 2x budget)
```

If no aliases match signals, emit all aliases up to the token budget (graceful fallback).

---

## Shell Detection

### Detection Priority Order (MEDIUM confidence — derived from atuin/zoxide source patterns)

```
1. $SHELL environment variable          → /bin/zsh, /bin/bash, /usr/bin/bash
2. Presence of config files:
   - ~/.zshrc          → zsh
   - ~/.bashrc         → bash
   - ~/.bash_profile   → bash (macOS default before Catalina)
3. /etc/shells presence of user's shell
4. Fallback: prompt user during `aliasman init`
```

`$SHELL` is reliable on macOS (set by Terminal.app and iTerm2). It is NOT reliable inside scripts or CI. For the `aliasman init` first-run flow, `$SHELL` is the primary signal. For the hook (which runs in Claude Code's environment, not a login shell), use the saved value from `config.toml`.

Implementation:

```rust
fn detect_shell() -> Shell {
    // 1. Check $SHELL
    if let Ok(shell_path) = std::env::var("SHELL") {
        if shell_path.contains("zsh") { return Shell::Zsh; }
        if shell_path.contains("bash") { return Shell::Bash; }
    }
    // 2. Check config file existence
    let home = dirs::home_dir().unwrap();
    if home.join(".zshrc").exists() { return Shell::Zsh; }
    if home.join(".bashrc").exists() { return Shell::Bash; }
    if home.join(".bash_profile").exists() { return Shell::Bash; }
    // 3. Ask the user
    Shell::Unknown
}
```

### Shell Config File Mapping

| Shell | Primary Config | Secondary Config | History File |
|-------|---------------|-----------------|--------------|
| zsh   | `~/.zshrc`    | `~/.zprofile`   | `$HISTFILE` or `~/.zsh_history` |
| bash  | `~/.bashrc`   | `~/.bash_profile` | `$HISTFILE` or `~/.bash_history` |

On macOS, bash uses `~/.bash_profile` for login shells (Terminal.app opens login shells). Check both; write to whichever exists. If both exist, prefer `~/.bash_profile` on macOS.

---

## History Parsing

### File Locations

- **zsh**: `$HISTFILE` env var, fallback `~/.zsh_history`, secondary `~/.zhistory`
- **bash**: `$HISTFILE` env var, fallback `~/.bash_history`

Always check `$HISTFILE` first (respects user configuration). Atuin uses this same priority.

### Zsh History Formats

**Basic format** (one command per line, no metadata):
```
git status
ls -la
docker-compose up -d
```

**Extended format** (set by `setopt EXTENDED_HISTORY` — very common):
```
: 1715300000:0;git status
: 1715300000:12;docker-compose up -d
```

Format: `: <unix_timestamp>:<elapsed_seconds>;<command>`

Multi-line commands in zsh extended history use backslash continuation:
```
: 1715300000:0;git commit -m \
  "long message"
```

**Detection**: if the first non-empty line matches the regex `^: \d+:\d+;`, it is extended format. Otherwise basic.

**Parser architecture**:

```rust
enum HistoryEntry {
    Basic { command: String },
    Extended { timestamp: u64, elapsed: u32, command: String },
}

fn parse_zsh_history(content: &str) -> Vec<HistoryEntry> {
    // Iterate lines; if extended: accumulate continuation lines
    // If basic: each line is one entry
}
```

Multi-line continuation: when a line ends with `\`, the next line is a continuation. Strip the trailing `\` and join with `\n`.

### Bash History Formats

**Basic format** (default):
```
git status
ls -la
```

**Timestamp format** (set by `HISTTIMEFORMAT`):
```
#1715300000
git status
#1715300000
ls -la
```

Lines starting with `#` followed by digits are timestamps preceding the next command.

**Detection**: if a line matches `^#\d{10}$`, the file uses timestamp format.

### Command Frequency Analysis

To suggest aliases from history, count command occurrences:

```rust
fn top_commands(entries: &[HistoryEntry], top_n: usize) -> Vec<(String, usize)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for entry in entries {
        // Normalize: trim, lowercase first word, keep first 2 tokens
        // "git status" → count as "git status"
        // "git commit -m 'foo'" → normalize to "git commit"
        let normalized = normalize_command(entry.command());
        *counts.entry(normalized).or_default() += 1;
    }
    let mut sorted: Vec<_> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.into_iter().take(top_n).collect()
}
```

Normalization: strip arguments after the second token for most commands; keep full form for commands where the subcommand matters (`git status`, `git checkout`, `docker-compose up`).

A command that appears more than 10 times in history and has no existing alias is a suggestion candidate.

---

## First-Run Import (aliasman init)

### Flow

```
1. Detect shell ($SHELL env var)
2. Load shell config file (e.g., ~/.zshrc)
3. Scan for alias lines using regex:
   ^\\s*(?:export\\s+)?alias\\s+(\\w[\\w-]*)=(['"]?)(.+)\\2\\s*(?:#.*)?$
4. For each match: create AliasRecord with source="imported"
5. Display table of found aliases, ask for confirmation
6. Write aliases.toml with imported aliases
7. Write ~/.aliases from the imported set
8. Append managed block to shell config:
   >>> aliasman >>>
   [ -f "$HOME/.aliases" ] && source "$HOME/.aliases"
   <<< aliasman <<<
9. Remove original alias lines from shell config (or leave them — user choice)
   Default: LEAVE original lines, add note that they are now managed via aliasman
   Reason: avoid corrupting zshrc; user can remove manually
```

### Step 9 Rationale

Do NOT auto-remove alias lines from zshrc on first import. The risk of corrupting zshrc is too high in v1. Instead:

- Print a table of imported aliases.
- Print: "Your existing alias lines in ~/.zshrc are unchanged. They will still be loaded. Run `aliasman clean-source` to safely remove them once you're satisfied."
- `aliasman clean-source` is a separate command that removes the original alias lines from the shell config using the marker comment approach — only run when explicitly invoked.

### Alias Line Regex

The alias regex must handle:
- `alias name='command'` — single quoted
- `alias name="command"` — double quoted
- `alias name=command` — unquoted (simple commands)
- `alias name='command'  # comment` — trailing comment
- `export alias name='command'` — export prefix (uncommon but valid)
- Multi-word commands in quotes: `alias ll='ls -la'`

The regex does NOT need to handle:
- Aliases spanning multiple lines (unusual, ignore)
- `alias` with no arguments (list all — a runtime query, not a definition)

---

## Component Responsibilities Summary

| Component | Owns | Reads | Writes |
|-----------|------|-------|--------|
| `AliasStore` | `aliases.toml` data model | `~/.config/aliasman/aliases.toml` | same (atomic) |
| `AliasFileWriter` | `~/.aliases` generation | `aliases.toml` | `~/.aliases` (atomic) |
| `ShellIntegration` | source-line injection | shell config files | shell config files (atomic + backup) |
| `ShellDetector` | shell detection logic | `$SHELL`, config files | `config.toml` (shell field) |
| `HistoryEngine` | history parsing + frequency | `~/.zsh_history` / `~/.bash_history` | nothing (read-only) |
| `HookRunner` | Claude hook output | `aliases.toml`, `$PWD`, `config.toml` | stdout (JSON) |
| `ConfigStore` | app configuration | `~/.config/aliasman/config.toml` | same (atomic) |
| `CLI` (`clap`) | command dispatch | user input | delegates to above |

---

## Patterns to Follow

### Pattern 1: Regenerate, Don't Patch

Never surgically patch `~/.aliases`. On any alias change (add/update/delete), regenerate the entire file from `aliases.toml`. The file is derived output. This eliminates drift, ordering bugs, and partial-write corruption.

### Pattern 2: Managed Block for Shell Configs

Shell config files (`~/.zshrc`, `~/.bashrc`) are NOT owned by aliasman. aliasman writes only a single managed block (the `source ~/.aliases` line) delimited by marker comments. It never touches anything outside that block.

### Pattern 3: Config Colocated with Data

`config.toml` and `aliases.toml` live in the same directory (`~/.config/aliasman/`). No splitting config across XDG data vs config dirs for a v1 CLI tool. Simplicity wins.

### Pattern 4: Hook Outputs Markdown in JSON Envelope

The Claude hook outputs a JSON envelope with `additionalContext` as Markdown. Claude reads Markdown naturally. Do not output structured JSON for the alias list — a readable bulleted list is more token-efficient and more useful to Claude than a JSON object.

---

## Anti-Patterns to Avoid

### Anti-Pattern 1: In-Place File Edit

Writing to `~/.aliases` or `~/.zshrc` directly (open for write, overwrite) creates a window where the file is empty or partial. A crash during write corrupts the shell config. Always use tempfile + atomic rename.

### Anti-Pattern 2: Storing Metadata in Shell Comments

Putting description or tags as comments in `~/.aliases` creates a second source of truth. If the user edits the shell file, comments get out of sync. Keep all metadata in `aliases.toml` only.

### Anti-Pattern 3: Injecting All Aliases into Claude

Dumping 200 aliases into the hook output wastes ~3,000-5,000 tokens per session. Use the scoring + budget approach. The token budget in `config.toml` is the safety valve.

### Anti-Pattern 4: Relying on $SHELL in the Hook

`$SHELL` may not be set when Claude Code runs the hook binary. Read the shell value from `config.toml` (written at `aliasman init` time).

### Anti-Pattern 5: Editing settings.json With String Manipulation

`~/.claude/settings.json` must be read with a JSON parser and written back as valid JSON. String manipulation risks producing invalid JSON, breaking all Claude Code hooks for the user.

---

## Scalability Considerations

| Concern | At 50 aliases | At 500 aliases | Notes |
|---------|--------------|----------------|-------|
| TOML parse time | <1ms | <10ms | Not a concern |
| ~/.aliases generation | <1ms | <5ms | Linear scan |
| Hook filter time | <1ms | <5ms | Simple scoring loop |
| Hook token budget | 800 tokens | 800 tokens | Budget is fixed; score filters |
| History parse (10k entries) | <100ms | <100ms | One-time scan, not hot path |

---

## Dependency Summary

| Crate | Version | Purpose | Confidence |
|-------|---------|---------|------------|
| `clap` | 4.x (derive) | CLI argument parsing | HIGH — Context7 verified |
| `serde` + `toml` | latest | TOML config/data serialization | HIGH — Context7 verified |
| `tempfile` | 3.x | Atomic file writes via `NamedTempFile::persist` | HIGH — Context7 verified |
| `directories` | 5.x | XDG/platform config paths | HIGH — Context7 verified |
| `anyhow` | 1.x | Ergonomic error handling | HIGH — Context7 verified |
| `serde_json` | 1.x | Hook output JSON encoding | HIGH |
| `chrono` or `std::time` | — | Timestamps for aliases | MEDIUM — `std::time::SystemTime` is sufficient |
| `regex` | 1.x | Alias line parsing, history detection | HIGH |

---

## Sources

- Context7: `/stebalien/tempfile` — `NamedTempFile::persist` atomic write pattern (HIGH)
- Context7: `/git_codeberg_org/dirs_directories-rs` — `BaseDirs` platform paths (HIGH)
- Context7: `/ajeetdsouza/zoxide` — shell init pattern, `zoxide init zsh` eval approach (HIGH)
- Context7: `/atuinsh/atuin` — history import paths, zsh extended format support (HIGH)
- Context7: `/websites/rs_clap` — derive subcommand architecture (HIGH)
- Live inspection: `~/.claude/settings.json` + `~/.claude/hooks/gsd-session-state.sh` — Claude Code hook JSON protocol (`hookSpecificOutput.additionalContext`) (HIGH)
- Live inspection: `~/.zshrc` — conda/mamba managed block marker pattern `>>> name >>>` / `<<< name <<<` (HIGH)
- Live inspection: `~/.zsh_history` — confirmed basic (non-extended) format on this machine; extended format documented from zsh man page knowledge (MEDIUM)
