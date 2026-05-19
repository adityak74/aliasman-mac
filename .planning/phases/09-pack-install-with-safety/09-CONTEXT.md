# Phase 9: Pack Install with Safety - Context

**Gathered:** 2026-05-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can safely install alias packs from local files or URLs. The install process includes:
- Dry-run preview of all aliases before applying
- Safety scanning for dangerous command patterns
- Collision detection where user aliases always win
- `--force` flag to override warnings and collisions
- Two-phase atomic apply (validate all, then write all)
- URL download support via `--url` flag

</domain>

<decisions>
## Implementation Decisions

### Install Flow
- Dry-run preview always runs first, showing alias count, safety warnings, and collisions
- Safety warnings block install unless `--force` is passed
- Collisions are skipped by default (user alias preserved), `--force` overrides
- Two-phase apply: validate all aliases first, then write atomically
- URL install downloads to temp file, parses, previews, installs

### CLI Interface
- `aliasman pack install <file>` — install from local TOML file
- `aliasman pack install --url <URL>` — install from URL
- `--force` flag overrides safety warnings and collision skipping
- Output shows preview, then install result with counts

### Code Integration
- `pack_installer.rs` already has all core logic (scan, collisions, preview, install)
- Need to add `Install` variant to `PackSubcommand` enum in `main.rs`
- Need to add handler in `run_pack()` for `PackSubcommand::Install`
- `download_pack()` is async — needs `tokio::runtime::Runtime::new().block_on()` wrapper in sync CLI
- After install: reindex semantic search, regenerate aliases file, show reload hint

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `pack_installer.rs` — complete with `scan_pack_safety()`, `detect_collisions()`, `InstallPreview`, `parse_pack_file()`, `download_pack()`, `install_pack()`, `InstallResult`
- `pack_manager.rs` — `create_pack()`, `save_pack_aliases()`, `export_pack()`
- `pack_registry.rs` — `PackRegistry::load()`, `register_pack()`, `save()`
- `pack_manifest.rs` — `PackManifest` with validation

### Established Patterns
- CLI handlers use `run_<command>` functions returning `Result<(), Box<dyn std::error::Error>>`
- Async calls from sync CLI wrapped in `tokio::runtime::Runtime::new().block_on()`
- Shell reload hint printed after mutations via `print_reload_hint()`
- Semantic index refreshed in background via `refresh_index()`

### Integration Points
- `PackSubcommand` enum in `main.rs:221-268` — add `Install` variant
- `run_pack()` in `main.rs:843-918` — add `Install` handler arm
- `main.rs` imports — add `pack_installer` module imports
- After install: call `regenerate_aliases()`, `refresh_index()`, `print_reload_hint()`

</code_context>

<specifics>
## Specific Ideas

No specific requirements — follow existing patterns from pack create/add/export handlers.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>
