# Requirements: aliasman

**Defined:** 2026-05-11
**Core Value:** Developers never have to manually edit shell config files to manage aliases, and their AI coding assistant knows and uses those same aliases.
**Milestone:** v0.0.1 CLI Alias Manager MVP

## v0.0.1 Requirements

### Foundation

- [x] **FND-01**: User can install and run an `aliasman` Rust CLI binary with global help and per-command help.
- [x] **FND-02**: User alias metadata is stored in a canonical aliasman data file separate from generated shell output.
- [x] **FND-03**: aliasman writes managed alias output atomically and can regenerate it from canonical metadata.
- [x] **FND-04**: aliasman rejects invalid alias names and warns before protected command names can be shadowed.

### Shell Integration

- [x] **SHL-01**: User can initialize aliasman for zsh or bash with automatic shell detection.
- [x] **SHL-02**: aliasman can import existing aliases from zsh/bash config files without duplicating aliases on repeated runs.
- [x] **SHL-03**: aliasman adds an idempotent managed source block to the detected shell config.
- [x] **SHL-04**: aliasman creates backups before modifying user shell config files.
- [x] **SHL-05**: aliasman prints actionable shell reload hints after any command that changes aliases or shell config.

### Alias Management

- [x] **ALS-01**: User can add an alias with named flags.
- [x] **ALS-02**: User can update an existing alias with named flags.
- [x] **ALS-03**: User can delete an alias by name.
- [x] **ALS-04**: User can list aliases in a readable CLI table.
- [x] **ALS-05**: User can see useful errors for duplicate aliases, missing aliases, invalid flags, and write failures.

### History Intelligence

- [x] **HST-01**: User can view command frequency statistics from zsh or bash history.
- [x] **HST-02**: aliasman correctly parses zsh extended history format.
- [x] **HST-03**: aliasman suggests short aliases for frequent long commands.
- [x] **HST-04**: User must explicitly approve a history-derived suggestion before it becomes an alias.
- [x] **HST-05**: aliasman flags risky history-derived commands that contain command substitution or other shell metacharacters.

### Claude Integration

- [x] **CLD-01**: User can install an aliasman Claude Code `SessionStart` hook without overwriting existing Claude settings.
- [x] **CLD-02**: The Claude hook command emits valid hook JSON containing alias context when relevant aliases exist.
- [x] **CLD-03**: The Claude hook filters aliases by current project context instead of injecting the entire alias list.
- [x] **CLD-04**: The Claude hook respects a configurable token budget for injected context.
- [x] **CLD-05**: The Claude hook exits cleanly with no noisy output when no alias context should be injected.

### Semantic Search

- [x] **SEM-01**: User aliases are embedded locally and indexed in a LanceDB-backed vector store.
- [x] **SEM-02**: aliasman can refresh the semantic alias index when aliases are added, updated, deleted, imported, or suggested.
- [x] **SEM-03**: User can search aliases semantically from the CLI using natural-language queries.
- [x] **SEM-04**: Claude can search aliases semantically through a local MCP tool/server instead of relying only on hook-injected context.
- [x] **SEM-05**: Semantic search runs locally by default and does not send alias commands or metadata to remote embedding services unless the user explicitly configures that later.

## Future Requirements

### Shells

- **SHL-F01**: User can manage PowerShell aliases and functions.
- **SHL-F02**: User can manage shell-specific alias syntax beyond POSIX-compatible zsh/bash aliases.

### Agent Integrations

- **AGT-F01**: User can inject alias context into Codex sessions.
- **AGT-F02**: User can configure additional coding-agent integrations.

### Distribution

- **DST-F01**: User can install aliasman through Homebrew.
- **DST-F02**: User can install aliasman through `cargo install`.

## Out of Scope

| Feature | Reason |
|---------|--------|
| PowerShell support | Different alias/function model; deferred until zsh/bash model is stable. |
| Team or organization shared alias libraries | v0.0.1 is focused on a solo developer workflow. |
| Cloud sync or real-time sync | Not required for local alias management MVP. |
| CLAUDE.md alias injection | Hook-based injection is dynamic and avoids stale files. |
| Homebrew publishing | Distribution packaging follows after the CLI behavior is implemented and verified. |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| FND-01 | Phase 1 | Complete |
| FND-02 | Phase 1 | Complete |
| FND-03 | Phase 1 | Complete |
| FND-04 | Phase 1 | Complete |
| SHL-01 | Phase 2 | Complete |
| SHL-02 | Phase 2 | Complete |
| SHL-03 | Phase 2 | Complete |
| SHL-04 | Phase 2 | Complete |
| SHL-05 | Phase 2 | Complete |
| ALS-01 | Phase 3 | Complete |
| ALS-02 | Phase 3 | Complete |
| ALS-03 | Phase 3 | Complete |
| ALS-04 | Phase 3 | Complete |
| ALS-05 | Phase 3 | Complete |
| HST-01 | Phase 4 | Complete |
| HST-02 | Phase 4 | Complete |
| HST-03 | Phase 4 | Complete |
| HST-04 | Phase 4 | Complete |
| HST-05 | Phase 4 | Complete |
| CLD-01 | Phase 5 | Complete |
| CLD-02 | Phase 5 | Complete |
| CLD-03 | Phase 5 | Complete |
| CLD-04 | Phase 5 | Complete |
| CLD-05 | Phase 5 | Complete |
| SEM-01 | Phase 6 | Complete |
| SEM-02 | Phase 6 | Complete |
| SEM-03 | Phase 6 | Complete |
| SEM-04 | Phase 6 | Complete |
| SEM-05 | Phase 6 | Complete |

**Coverage:**
- v0.0.1 requirements: 29 total
- Mapped to phases: 29
- Unmapped: 0

---
*Requirements defined: 2026-05-11*
*Last updated: 2026-05-15 — all 29 requirements complete, milestone v0.0.1 verified*
