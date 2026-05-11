# aliasman

## What This Is

A Rust CLI that makes shell alias management a first-class developer experience. It manages aliases in a dedicated `~/.aliases` file (sourced by your shell config), provides full CRUD via named CLI flags, auto-detects existing aliases across zsh/bash/PowerShell, suggests new aliases from your shell history, and integrates with Claude Code via a hook that intelligently injects your relevant aliases into each session so Claude uses your shortcuts instead of retyping long commands.

## Core Value

Developers never have to manually edit shell config files to manage aliases — and their AI coding assistant knows and uses those same aliases.

## Current Milestone: v0.0.1 CLI Alias Manager MVP

**Goal:** Build the first usable aliasman release: a Rust CLI for managing shell aliases safely, with zsh/bash support, history-based suggestions and stats, and Claude Code hook injection.

**Target features:**
- CLI alias CRUD: add, update, delete, and list aliases through named CLI flags
- Shell integration: detect zsh/bash/bash_profile aliases and manage a dedicated alias file safely
- History intelligence: suggest short aliases for frequent commands and show command frequency analytics
- CLI ergonomics: console help, options, arguments, and shell workflow hints
- Claude integration: install a Claude Code hook that injects relevant aliases into context without dumping everything

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] User can add, update, delete, and list aliases via CLI with named flags
- [ ] aliasman auto-detects existing aliases from shell config files on first run
- [ ] aliasman manages a dedicated `~/.aliases` file and adds `source ~/.aliases` to the detected shell config
- [ ] aliasman suggests short aliases for frequent commands based on shell history
- [ ] aliasman shows shell history statistics and command frequency analytics
- [ ] Claude Code hook fires on session start and injects contextually relevant aliases
- [ ] Hook uses smart filtering — only injects aliases relevant to the current project/directory
- [ ] Console help for all commands, options, and arguments
- [ ] Shell workflow hints surfaced where useful

### Out of Scope

- CLAUDE.md file injection — Hook-only approach keeps it dynamic; file injection is a separate concern for v2
- Team/org shared alias libraries — Solo developer focus for v1; sharing/sync is a v2 feature
- Homebrew formula publishing — Build the tool first; formula after v1 is solid
- Real-time alias sync — No cloud or sync service in v1

## Context

- Target platform: macOS first, with cross-platform (Linux) compatibility goal
- Shell support: zsh (primary), bash, bash_profile — PowerShell deferred to v2
- Language: Rust — single binary distribution, ideal for shell tooling
- Distribution: Homebrew as primary channel; `cargo install` as secondary
- The Claude Code hook should leverage the existing Claude Code hooks system (settings.json hooks) and should be smart about token budget — inject only what's relevant, not the entire alias file
- Alias storage strategy: `~/.aliases` file managed by aliasman, with a single `source ~/.aliases` line added to the user's shell config on install
- Parametrized aliases: Named CLI flags approach (`aliasman add --name gs --command "git status"`)

## Constraints

- **Tech Stack**: Rust — chosen for single-binary distribution, performance, and shell tooling ecosystem
- **Platform**: macOS first; Linux compatibility is a goal but not a blocker for v1
- **Distribution**: Homebrew primary — must produce a valid Homebrew formula and binary release

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Dedicated `~/.aliases` file vs editing shell config directly | Safer, easier to version, avoids corrupting zshrc | — Pending |
| Claude Code hook (not CLAUDE.md injection) for v1 | Dynamic and session-aware; no stale static file to maintain | — Pending |
| Smart alias filtering in hook (not dump all) | Token efficiency — Claude Code sessions have context limits | — Pending |
| Rust over Go | Single binary, better shell tooling ergonomics, no runtime | — Pending |
| Homebrew as primary distribution | macOS developer standard; expected install UX | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-05-11 after starting milestone v0.0.1*
