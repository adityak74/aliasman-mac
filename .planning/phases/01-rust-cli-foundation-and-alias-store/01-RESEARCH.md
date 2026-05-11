# Phase 1 Research: Rust CLI Foundation And Alias Store

**Phase:** 01 - Rust CLI Foundation And Alias Store
**Researched:** 2026-05-11
**Status:** Ready for planning

## Phase Goal

Create the Rust CLI scaffold and safe canonical alias storage foundation for aliasman. This phase should establish the crate, the minimal public CLI surface, the canonical alias metadata model, reusable validation, atomic managed alias output writes, and the test baseline required before shell init or CRUD features are added.

## Requirements Covered

- **FND-01:** User can install and run an `aliasman` Rust CLI binary with global help and per-command help.
- **FND-02:** User alias metadata is stored in a canonical aliasman data file separate from generated shell output.
- **FND-03:** aliasman writes managed alias output atomically and can regenerate it from canonical metadata.
- **FND-04:** aliasman rejects invalid alias names and warns before protected command names can be shadowed.

## Recommended Crate Layout

Use a standard Rust package at the repository root:

```text
Cargo.toml
src/
  main.rs
  lib.rs
  model.rs
  store.rs
  validation.rs
tests/
  store_atomic.rs
```

Rationale:
- `main.rs` owns only CLI parsing and exit wiring.
- `lib.rs` exports reusable modules for later phases.
- `model.rs` defines alias metadata and enum types.
- `validation.rs` centralizes alias-name and protected-name policy.
- `store.rs` owns canonical metadata serialization and managed alias file rendering/writing.
- `tests/store_atomic.rs` proves filesystem behavior in a tempdir without touching the real home directory.

## Dependencies

Use these dependencies in Phase 1:

```toml
[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
tempfile = "3"
thiserror = "2"
toml = "0.9"

[dev-dependencies]
assert_fs = "1"
predicates = "3"
```

Notes:
- `serde` + `toml` support canonical alias metadata.
- `tempfile` provides same-directory temp files and atomic persist behavior.
- `thiserror` keeps validation/store errors reusable by later CLI commands.
- `assert_fs` is useful for tempdir integration tests; direct `tempfile::TempDir` is also acceptable if the executor prefers fewer dev dependencies.

## Canonical Data Model

Implement the full v0.0.1 alias record now:

```rust
pub struct AliasRecord {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub shell: AliasShell,
    pub source: AliasSource,
    pub created_at: u64,
    pub updated_at: u64,
}
```

Recommended enums:

```rust
pub enum AliasShell {
    All,
    Zsh,
    Bash,
}

pub enum AliasSource {
    User,
    Imported,
    Suggested,
}
```

Canonical metadata file shape:

```toml
[[aliases]]
name = "gs"
command = "git status"
description = "Quick git status"
tags = ["git"]
shell = "all"
source = "user"
created_at = 1715300000
updated_at = 1715300000
```

Phase 1 does not need to decide the final user config path. It can expose load/save helpers that accept explicit `Path` arguments so tests and later phases can route paths safely.

## Managed Alias Output

Phase 1 should render generated shell aliases to a fixed `~/.aliases` target conceptually, but implementation should accept a path argument for testability. Later shell integration can resolve the actual home path.

Generated output should be deterministic:

```sh
# aliasman managed - do not edit manually
# Run `aliasman list` to view aliases once CRUD commands are available.

alias gs='git status'
```

Sort aliases by `name` before rendering. Use bash/zsh-compatible alias syntax only.

Quoting rule for Phase 1:
- Single-quote alias commands by default: `alias gs='git status'`.
- Escape embedded single quotes with the standard shell sequence: close quote, escaped quote, reopen quote. For example, `echo 'hi'` becomes `alias x='echo '\''hi'\'''`.

## Atomic Write Pattern

Implement a reusable helper in `store.rs`:

```rust
pub fn write_atomic(path: &Path, contents: &str) -> Result<(), StoreError>
```

Required behavior:
- Create the parent directory if needed for aliasman-owned metadata paths.
- Create the temporary file in the same directory as the target path.
- Write all bytes.
- Flush the file.
- Persist/rename over the target path atomically.
- Return structured errors instead of panicking.

For shell config files, backups are Phase 2. For Phase 1, prove atomic managed alias output writes in a tempdir.

## Validation Policy

Alias-name syntax must be reusable library behavior:

```text
[A-Za-z_][A-Za-z0-9_-]*
```

Protected names must be recognized in Phase 1 so later writes cannot accidentally shadow dangerous commands. Initial protected set:

```text
rm, mv, cp, ln, chmod, chown, kill, sudo, su, cd, source, exec,
eval, export, unset, exit, logout, git, ssh, curl, wget, brew
```

Recommended API:

```rust
pub fn validate_alias_name(name: &str) -> Result<(), ValidationError>
pub fn is_protected_name(name: &str) -> bool
pub fn validate_alias_name_for_write(name: &str, force: bool) -> Result<(), ValidationError>
```

`validate_alias_name_for_write("rm", false)` should fail. `validate_alias_name_for_write("rm", true)` should pass if the syntax is valid.

## CLI Surface

Phase 1 should keep the user-facing command surface intentionally small:

- `aliasman --help`
- `aliasman --version`

Do not add future command stubs that print "not implemented". Later phases should add real commands when their behavior exists.

Use `clap` derive so global help/version are generated consistently.

## Testing Strategy

Required tests:
- Validation accepts valid names such as `gs`, `_local`, `git-status`, and `g1`.
- Validation rejects invalid names such as `1bad`, `bad name`, `bad$name`, and empty string.
- Protected-name checks flag `rm`, `sudo`, `git`, and `curl`.
- Protected names require `force = true` in the write validator.
- Alias metadata serializes/deserializes through TOML without dropping fields.
- Rendered alias output is deterministic and sorted by name.
- Rendered alias output escapes embedded single quotes correctly.
- Atomic write helper writes expected contents to a tempdir target.
- Atomic write helper can overwrite an existing tempdir target.

Optional if cheap:
- CLI help/version smoke tests.

## Planning Guidance

One plan is sufficient for this phase if it clearly sequences scaffold, model/validation, store rendering/writes, and tests. Do not split into multiple plans unless the planner needs separate waves for clarity.

The plan must not include shell init, alias CRUD commands, history parsing, Claude hook registration, Homebrew packaging, or PowerShell.

## Threat Model

Primary risks in this phase:
- Writing outside test tempdirs during tests.
- Corrupting a target file with a partial write.
- Accepting alias names that later shadow dangerous commands without explicit force.
- Emitting shell alias output that is malformed when commands contain quotes.

Mitigations should be present in the plan tasks and verification steps.

