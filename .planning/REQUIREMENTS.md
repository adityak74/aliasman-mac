# Requirements: aliasman

**Defined:** 2026-05-11
**Core Value:** Developers never have to manually edit shell config files to manage aliases, and their AI coding assistant knows and uses those same aliases.
**Milestone:** v0.0.1 CLI Alias Manager MVP

## v0.0.1 Requirements

### Foundation

- [ ] **FND-01**: User can install and run an `aliasman` Rust CLI binary with global help and per-command help.
- [ ] **FND-02**: User alias metadata is stored in a canonical aliasman data file separate from generated shell output.
- [ ] **FND-03**: aliasman writes managed alias output atomically and can regenerate it from canonical metadata.
- [ ] **FND-04**: aliasman rejects invalid alias names and warns before protected command names can be shadowed.

### Shell Integration

- [ ] **SHL-01**: User can initialize aliasman for zsh or bash with automatic shell detection.
- [ ] **SHL-02**: aliasman can import existing aliases from zsh/bash config files without duplicating aliases on repeated runs.
- [ ] **SHL-03**: aliasman adds an idempotent managed source block to the detected shell config.
- [ ] **SHL-04**: aliasman creates backups before modifying user shell config files.
- [ ] **SHL-05**: aliasman prints actionable shell reload hints after any command that changes aliases or shell config.

### Alias Management

- [ ] **ALS-01**: User can add an alias with named flags.
- [ ] **ALS-02**: User can update an existing alias with named flags.
- [ ] **ALS-03**: User can delete an alias by name.
- [ ] **ALS-04**: User can list aliases in a readable CLI table.
- [ ] **ALS-05**: User can see useful errors for duplicate aliases, missing aliases, invalid flags, and write failures.

### History Intelligence

- [ ] **HST-01**: User can view command frequency statistics from zsh or bash history.
- [ ] **HST-02**: aliasman correctly parses zsh extended history format.
- [ ] **HST-03**: aliasman suggests short aliases for frequent long commands.
- [ ] **HST-04**: User must explicitly approve a history-derived suggestion before it becomes an alias.
- [ ] **HST-05**: aliasman flags risky history-derived commands that contain command substitution or other shell metacharacters.

### Claude Integration

- [ ] **CLD-01**: User can install an aliasman Claude Code `SessionStart` hook without overwriting existing Claude settings.
- [ ] **CLD-02**: The Claude hook command emits valid hook JSON containing alias context when relevant aliases exist.
- [ ] **CLD-03**: The Claude hook filters aliases by current project context instead of injecting the entire alias list.
- [ ] **CLD-04**: The Claude hook respects a configurable token budget for injected context.
- [ ] **CLD-05**: The Claude hook exits cleanly with no noisy output when no alias context should be injected.

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
| FND-01 | TBD | Pending |
| FND-02 | TBD | Pending |
| FND-03 | TBD | Pending |
| FND-04 | TBD | Pending |
| SHL-01 | TBD | Pending |
| SHL-02 | TBD | Pending |
| SHL-03 | TBD | Pending |
| SHL-04 | TBD | Pending |
| SHL-05 | TBD | Pending |
| ALS-01 | TBD | Pending |
| ALS-02 | TBD | Pending |
| ALS-03 | TBD | Pending |
| ALS-04 | TBD | Pending |
| ALS-05 | TBD | Pending |
| HST-01 | TBD | Pending |
| HST-02 | TBD | Pending |
| HST-03 | TBD | Pending |
| HST-04 | TBD | Pending |
| HST-05 | TBD | Pending |
| CLD-01 | TBD | Pending |
| CLD-02 | TBD | Pending |
| CLD-03 | TBD | Pending |
| CLD-04 | TBD | Pending |
| CLD-05 | TBD | Pending |

**Coverage:**
- v0.0.1 requirements: 24 total
- Mapped to phases: 0
- Unmapped: 24

---
*Requirements defined: 2026-05-11*
*Last updated: 2026-05-11 after initial definition*
