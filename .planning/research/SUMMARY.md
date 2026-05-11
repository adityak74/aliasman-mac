# Research Summary: aliasman v0.0.1

**Project:** aliasman
**Milestone:** v0.0.1 CLI Alias Manager MVP
**Synthesized:** 2026-05-11

---

## Stack Additions

- Use Rust with `clap` derive for the command surface.
- Use `serde`, `serde_json`, and TOML support for config, alias metadata, and Claude settings merge.
- Use `tempfile` for atomic writes to managed alias files and shell config files.
- Use `regex` narrowly for alias parsing, history parsing, and managed block detection.
- Use buffered file reads for large shell history files.

## Feature Table Stakes

- Alias CRUD via named CLI flags.
- Shell detection for zsh and bash, including macOS bash profile behavior.
- Safe first-run import of existing aliases without duplicates.
- Dedicated managed alias output file sourced by shell config.
- History command stats and suggestion workflow with explicit user approval.
- Console help and actionable shell reload hints.
- Claude Code hook install and `SessionStart` context injection with relevance filtering.

## Architecture Guidance

- Treat `aliases.toml` as canonical metadata and `~/.aliases` as derived shell output.
- Never mutate shell config files in place.
- Regenerate the managed aliases file from canonical data on every alias write.
- Split implementation into clear components:
  - CLI parsing
  - Alias store
  - Shell detector and shell config integration
  - History engine
  - Claude hook installer and hook runner
- Keep PowerShell out of v0.0.1 because its alias/function model is different from POSIX shell aliases.

## Watch Out For

- Non-atomic writes can corrupt user shell config files.
- Re-running init must not duplicate imported aliases or source blocks.
- Alias names that shadow dangerous system commands need a protected-name guard and explicit `--force`.
- Users need a reload instruction after every mutation because the current parent shell cannot be changed by the CLI process.
- zsh extended history parsing must be handled before history suggestions are trusted.
- Claude hook output must be filtered and token-capped to avoid wasting context.
- Generated aliases should use bash/zsh-compatible syntax only.

## Recommended Roadmap Shape

1. Project scaffold and safe storage foundation.
2. Shell detection, init, import, and safe config writes.
3. Alias CRUD/list UX.
4. History stats and suggestions.
5. Claude hook install and smart injection.

