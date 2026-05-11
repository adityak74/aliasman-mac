# Feature Research: aliasman v0.0.1

**Project:** aliasman — Rust CLI alias manager for macOS/Linux
**Milestone:** v0.0.1 CLI Alias Manager MVP
**Researched:** 2026-05-11
**Confidence:** HIGH for core CLI/shell behavior, MEDIUM for Claude hook polish until implemented and tested locally

---

## Feature Categories

### Alias CRUD

**Table stakes**
- User can add an alias with explicit named flags: `aliasman add --name gs --command "git status"`.
- User can update an existing alias without manually editing shell files.
- User can delete an alias by name.
- User can list aliases in a readable table.
- User gets clear errors for duplicate names, missing names, invalid names, and protected command names.

**Expected behavior**
- Alias names must be validated before write.
- Mutating commands regenerate the managed aliases file from canonical stored data.
- Every mutating command prints a shell reload hint because child processes cannot update the parent shell.
- CRUD should be idempotent where possible: repeated init/import should not duplicate aliases or source blocks.

**Differentiators**
- Provenance labels: user-created, imported, suggested.
- Optional description/tags for later hook filtering.
- Machine-readable output later (`--json`) if automation becomes important.

### Shell Integration

**Table stakes**
- User can run an init/install command that detects zsh or bash.
- aliasman can scan existing shell config files for `alias name='command'` style aliases.
- aliasman creates or updates a dedicated managed aliases file.
- aliasman adds one managed source block to the detected shell config.
- aliasman can safely avoid adding duplicate source blocks.

**Expected behavior**
- zsh first on macOS, bash support via `~/.bashrc` and `~/.bash_profile`.
- Managed block markers should make install/uninstall reliable:
  ```sh
  # >>> aliasman >>>
  [ -f "$HOME/.aliases" ] && source "$HOME/.aliases"
  # <<< aliasman <<<
  ```
- Atomic writes and backups are mandatory for shell config files.

**Deferred**
- PowerShell support. It uses a different alias/function model and should not shape the v0.0.1 storage format.
- Team/shared alias libraries.
- Cloud sync or real-time sync.

### History Suggestions And Stats

**Table stakes**
- User can inspect frequent commands from zsh/bash history.
- User can ask for alias suggestions from frequent long commands.
- Suggestions are displayed for explicit approval; aliasman never auto-creates aliases from history.
- User can see basic command frequency analytics.

**Expected behavior**
- zsh extended history must be parsed correctly.
- Large history files should be streamed or bounded.
- Suggested alias names must avoid protected command names.
- Commands with shell substitution or other dangerous patterns should be flagged for careful review.

**Differentiators**
- Score suggestions by frequency, command length, recency, and whether a shorter alias already exists.
- Explain why each alias was suggested.

### CLI Ergonomics

**Table stakes**
- `--help` output exists for every command and option.
- Error messages say what went wrong and how to fix it.
- Mutation success messages include next shell action.
- Commands use named flags rather than positional ambiguity.

**Expected commands**
- `aliasman init`
- `aliasman add --name <name> --command <command>`
- `aliasman update --name <name> --command <command>`
- `aliasman delete --name <name>`
- `aliasman list`
- `aliasman suggest`
- `aliasman stats`
- `aliasman hook --install claude`
- `aliasman hook --shell claude`

### Claude Code Hook

**Table stakes**
- User can install a Claude Code `SessionStart` hook.
- The hook command reads aliasman data and emits valid Claude hook JSON.
- The hook injects only relevant aliases, not the whole alias file.
- If no aliases are relevant or an error occurs, the hook exits cleanly without noisy output.

**Expected behavior**
- Hook registration must merge with existing `~/.claude/settings.json` hooks rather than overwriting them.
- The hook command path should be absolute.
- Relevance filtering should use current working directory signals such as `.git`, `Cargo.toml`, `package.json`, `Dockerfile`, and alias tags.
- Output should be capped by an approximate token budget.

**Deferred**
- Codex or other coding-agent integrations.
- PreToolUse command rewriting or blocking.
- CLAUDE.md generation.

---

## v0.0.1 Scope Recommendation

Build the milestone as four user-visible slices:

1. Safe alias store and shell init.
2. Alias CRUD and listing.
3. History suggestions and stats.
4. Claude hook install and relevant context injection.

This order keeps data safety and shell compatibility ahead of convenience features, then layers AI integration after aliases exist and can be filtered.

