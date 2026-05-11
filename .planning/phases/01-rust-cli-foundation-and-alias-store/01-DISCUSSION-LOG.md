# Phase 1: Rust CLI Foundation And Alias Store - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-11
**Phase:** 1-Rust CLI Foundation And Alias Store
**Areas discussed:** Store shape, Generated alias file, CLI command surface, Validation policy, Testing baseline

---

## Store Shape

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal record | Store `name` and `command` only. Fastest foundation, but later phases must migrate when adding tags/source/history. | |
| Full v0.0.1 record | Store `name`, `command`, optional `description`, `tags`, `shell`, `source`, `created_at`, `updated_at`. Slightly more upfront work, avoids early schema churn. | ✓ |
| Agent-focused record | Full record plus context fields like `projects`, `tools`, or `relevance_hints` for Claude filtering later. More future-ready, but risks over-designing Phase 1. | |

**User's choice:** Full v0.0.1 record.
**Notes:** Keep enough metadata for history suggestions and Claude filtering, but avoid agent-specific relevance fields in Phase 1.

---

## Generated Alias File

| Option | Description | Selected |
|--------|-------------|----------|
| Fixed `~/.aliases` | Simple, predictable, matches the project brief; config can come later for edge cases. | ✓ |
| Configurable path now | Store `aliases_file.path` in config from Phase 1, defaulting to `~/.aliases`; more flexible but adds config surface early. | |
| XDG-only generated file | Keep generated shell output under `~/.config/aliasman/aliases.sh`; cleaner app ownership but less familiar for shell users. | |

**User's choice:** Fixed `~/.aliases`.
**Notes:** Configuration for generated alias output is deferred.

---

## CLI Command Surface

| Option | Description | Selected |
|--------|-------------|----------|
| Foundation-only | `aliasman --help`, `aliasman --version`, and internal library/store behavior. Later phases add real user commands. | ✓ |
| Stub full command tree | Define `init`, `add`, `update`, `delete`, `list`, `suggest`, `stats`, `hook` now, but most return "not implemented yet." | |
| Foundation plus store debug commands | Add temporary developer commands like `store validate` or `store render` to test the store manually. | |

**User's choice:** Foundation-only.
**Notes:** Prefer tests over temporary debug commands.

---

## Validation Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Strict now | Validate names with `[A-Za-z_][A-Za-z0-9_-]*`, reject invalid names, and require an explicit force path for protected names like `rm`, `sudo`, `git`, `curl`. | ✓ |
| Name validation only | Validate syntax now, but defer protected-name policy until CRUD commands exist. | |
| Permissive now | Accept most names and let later phases tighten behavior. | |

**User's choice:** Strict now.
**Notes:** Validation should be reusable library behavior for later CRUD and history phases.

---

## Testing Baseline

| Option | Description | Selected |
|--------|-------------|----------|
| Unit + tempdir integration tests | Unit tests for validation/serialization plus tempdir tests proving atomic managed alias output writes without touching real home files. | ✓ |
| Unit tests only | Enough for the library foundation, with filesystem integration deferred to shell init. | |
| CLI smoke tests too | Unit + tempdir integration + `aliasman --help`/`--version` command execution tests. | |

**User's choice:** Unit + tempdir integration tests.
**Notes:** CLI process smoke tests are optional if cheap, but not required for Phase 1 completion.

---

## the agent's Discretion

- Exact Rust module names.
- Crate layout details.
- Timestamp representation.
- TOML crate choice.
- Atomic-write helper signatures.

## Deferred Ideas

- Configurable generated alias output paths.
- Full future command tree stubs.
- CLI process smoke tests unless cheap to include.
- PowerShell support.
