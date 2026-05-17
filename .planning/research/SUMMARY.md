# Project Research Summary

**Project:** aliasman — Rust CLI alias manager with shareable alias packs
**Domain:** Shell alias management + pack-based distribution (macOS/Linux)
**Researched:** 2026-05-16
**Confidence:** HIGH (all crate versions verified via crates.io; architecture validated against existing codebase)

---

## Executive Summary

aliasman is a Rust CLI tool that manages shell aliases with intelligent Claude Code hook integration. The v0.1 Alias Library feature adds a pack-based system for creating, sharing, and installing curated alias collections. Experts build this kind of product by keeping the pack format simple (a single TOML file), isolating pack data from user data, and merging at render time rather than at storage time. The recommended approach is a two-phase install (validate all, then apply all), user-first conflict resolution (user aliases always win), and file-based distribution via local files, HTTP URLs, and optionally git repos.

The key risk is security: third-party packs execute arbitrary shell commands with the user's full privileges. This requires a mandatory dry-run preview, a command safety scanner that flags dangerous patterns (command substitution, network access, pipe-to-shell), and explicit user confirmation before any pack alias is activated. The second key risk is scope creep into building a full package manager -- the research is unequivocal that v0.1 must have no dependency resolution, no central registry, and no semver range enforcement.

## Key Findings

### Recommended Stack

Six new crates are required for the Alias Library feature. All versions were verified against crates.io. The existing `reqwest` and `tokio` dependencies are reused for HTTP downloads and async runtime, avoiding unnecessary additions.

**New dependencies:**
- `semver` 1.0 — Pack version parsing and comparison (dtolnay, authoritative)
- `git2` 0.20 (features: `https` only) — Git repo cloning for pack install; SSH is optional behind `--features ssh` to avoid OpenSSL dependency
- `tar` 0.4 + `flate2` 1.1 — `.tar.gz` archive extraction for distributed packs
- `sha2` 0.10 — SHA-256 checksum verification for pack integrity
- `url` 2.5 — URL parsing to route pack sources (file vs HTTP vs git)

**System dependencies to document:**
- `git2` requires `libgit2` + `pkg-config` on the build system (`brew install libgit2` on macOS)
- `flate2` requires `libz` (pre-installed on macOS, `zlib1g-dev` on Debian/Ubuntu)

**Crates NOT needed (reusing existing):** `reqwest`, `tokio`, `toml`, `serde_json`, `tempfile`, `dirs` — all already in Cargo.toml.

### Expected Features

**Must have (table stakes):**
- Pack create — generate a pack directory with manifest
- Pack add-alias — add aliases to an existing pack
- Pack export — produce a shareable TOML file from a pack
- Pack install (file) — install from local TOML
- Pack install (URL) — download from HTTP URL then install
- Pack list — show installed packs with metadata
- Pack remove — uninstall pack and its aliases
- Pack manifest — structured TOML with name, version, description, author, aliases
- Source tracking — `AliasSource::Pack(String)` variant for provenance

**Should have (differentiators):**
- Pack conflict detection — pre-install collision scan with user prompt
- Pack override priority — user aliases always win over pack aliases
- Built-in curated packs — k8s and docker pre-bundled to prove the model
- Pack source tracking — every alias records pack origin for targeted removal

**Defer (v0.2+):**
- Pack install (git) — clone workflow is complex; defer until file/URL model is stable
- Pack update — requires git source tracking and diff logic
- Interactive conflict resolution — basic logging + `--force` is enough for v0.1
- Parameterized aliases (`{{namespace}}` placeholders) — nice-to-have, not blocking
- Remaining built-in packs (terraform, git-advanced, cicd) — community-contributed
- Central pack registry — requires server infrastructure, auth, moderation

### Architecture Approach

The core design principle is **isolation with merge-at-render**. Packs do NOT write to `aliases.toml`. Instead, each pack lives in an isolated directory under `~/.config/aliasman/packs/{name}/`. The `render_aliases_file()` function in `store.rs` is extended to read from both the user store and all installed packs, merging them into a single `~/.aliases` output with user-first conflict resolution. This keeps pack aliases fully reversible (uninstall = delete pack directory) and prevents pack data from polluting the user's canonical store.

**New modules:**
1. `pack_manifest.rs` — Parse and validate `pack.toml` files; the foundational component every other pack feature depends on
2. `pack_registry.rs` — Manage `registry.toml` tracking installed packs (name, version, source, install time, conflicts)
3. `pack_manager.rs` — Orchestrate pack lifecycle (create, export, install, remove) with an `AliasMerger` submodule for conflict-aware merging

**Modified existing modules:**
- `model.rs` — Add `AliasSource::Pack(String)` variant
- `store.rs` — Change `render_aliases_file()` to accept merged alias set instead of just user store
- `hook.rs` — Add `AliasSource::Pack` scoring arm in `score_alias()`
- `main.rs` — Add `Pack` subcommand with `create`, `export`, `install`, `list`, `remove` sub-subcommands
- `lib.rs` — Declare 3 new modules

**Build order (dependency graph):**
pack_manifest.rs -> model.rs (Pack variant) -> pack_registry.rs -> pack_manager.rs -> store.rs/hook.rs/main.rs modifications

### Critical Pitfalls

1. **Untrusted command execution (P0, CRITICAL)** — Third-party packs run arbitrary shell commands with full user privileges. Prevention: mandatory dry-run preview showing every alias command before install, command safety scanner flagging dangerous patterns (`$(...)`, `curl`, `| bash`, `rm -rf`), explicit `[y/N]` confirmation. This is not retrofittable — must be in the first pack install implementation.

2. **Silent name collision overwrites (P0, CRITICAL)** — Pack install silently replacing a user's personal alias erodes trust. Prevention: pre-install collision scan, present all conflicts before proceeding, default to "keep personal" (user always wins), require `--force` to overwrite.

3. **Package manager scope creep (P0, CRITICAL)** — Implementing dependency resolution, semver ranges, and a central registry multiplies effort 10x for unrequested features. Prevention: v0.1 is strictly file + URL install only. No dependencies, no registry, no version range resolution. Version is advisory, not enforced.

4. **Partial install on failure (P1)** — Installing 12 of 20 aliases before the 13th fails validation leaves inconsistent state. Prevention: two-phase install (Phase 1 validates all aliases, Phase 2 applies all only if Phase 1 passes). Build complete new store in memory first; only write to disk if everything passes.

5. **Pack update silently changes commands (P1)** — Updating a pack changes alias commands without the user knowing, breaking muscle memory. Prevention: pre-update diff showing what changed, explicit confirmation, rollback support. (Note: pack update itself is deferred to v0.2, but the pattern should be designed now.)

## Implications for Roadmap

Based on research, the recommended phase structure follows the architectural dependency graph and groups features by risk level.

### Phase 1: Pack Foundation
**Rationale:** `pack_manifest.rs` is the dependency root — every pack component reads or validates the manifest format. The `AliasSource::Pack` variant must exist before any pack code compiles.
**Delivers:** Pack manifest schema (TOML), `pack_manifest.rs` module with parse/validate, `model.rs` extension with `Pack` variant
**Addresses:** Pack manifest (FEATURES.md table stakes)
**Avoids:** Pitfall #12 (scope creep) by defining a minimal manifest with no `dependencies` field, no registry URL, no version ranges
**Pitfalls to address:** Pitfall #14 (format migration) — include `format_version` in the initial schema from day one

### Phase 2: Pack Registry & Lifecycle
**Rationale:** Registry and manager depend on the manifest from Phase 1. This phase establishes the data tracking and CRUD operations before any install logic touches the user's aliases.
**Delivers:** `pack_registry.rs` (load/save/query), `pack_manager.rs` shell (create, export, list), pack directory structure under `~/.config/aliasman/packs/`
**Addresses:** Pack create, add-alias, export, list (FEATURES.md table stakes)
**Avoids:** Pitfall #20 (dual source of truth) — registry tracks metadata only; `AliasSource::Pack` on each alias is the authoritative source for ownership
**Pitfalls to address:** Pitfall #17 (built-in pack rigidity) — ship built-in packs as files in config dir, not embedded via `include_str!`

### Phase 3: Pack Install with Safety
**Rationale:** This is the highest-risk phase. It must implement all P0 security measures from the start. There is no safe way to retrofit security on pack install.
**Delivers:** `install_pack_from_file()`, `install_pack_from_url()`, dry-run preview, command safety scanner, pre-install collision scan, two-phase validate-then-apply
**Uses:** `reqwest` (HTTP download), `tar` + `flate2` (extraction), `sha2` (checksum), `url` (source routing)
**Implements:** `AliasMerger::merge_aliases()` with user-first conflict resolution
**Addresses:** Pack install (file), Pack install (URL), conflict detection (FEATURES.md)
**Avoids:** Pitfall #9 (untrusted execution), Pitfall #10 (silent overwrite), Pitfall #16 (partial install)
**Pitfalls to address:** All three P0 pitfalls (#9, #10, #12). This phase must not proceed without the safety scanner and dry-run preview.

### Phase 4: Pack Remove & Integration
**Rationale:** Remove depends on install working correctly. Integration changes (hook scoring, regenerate_aliases) are low-risk modifications to existing code.
**Delivers:** `remove_pack()` with modified-by-user tracking, `hook.rs` Pack scoring, `main.rs` regenerate_aliases using merger, full Pack CLI wiring
**Addresses:** Pack remove (FEATURES.md table stakes), source tracking (differentiator)
**Avoids:** Pitfall #15 (uninstall removes user-modified aliases) — track `modified_by_user` flag
**Pitfalls to address:** Pitfall #15 (modified alias preservation on uninstall)

### Phase 5: Built-in Packs
**Rationale:** This is content, not code. It depends on all pack infrastructure being functional. Two curated packs (k8s, docker) prove the model without over-investing.
**Delivers:** k8s pack (15-20 kubectl aliases), docker pack (10-15 docker/compose aliases), `aliasman init --apply` prompt for built-in pack selection
**Addresses:** Built-in curated packs (FEATURES.md differentiator)
**Pitfalls to address:** Pitfall #17 — ship as files, not embedded

### Phase Ordering Rationale

- **Dependency-driven:** `pack_manifest.rs` must exist before anything else compiles. Registry depends on manifest. Manager depends on both. Install depends on manager. This is a strict linear chain.
- **Risk-front-loaded:** Security (P0 pitfalls #9, #10) is implemented in Phase 3 alongside the first install logic, not retrofitted later.
- **Scope-controlled:** git-based install (Phase 3 scope creep risk) is deferred to v0.2. The file + URL path covers 95% of use cases.
- **Content last:** Built-in packs are data files that exercise the full install pipeline. They come after all code is stable.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 3 (Pack Install with Safety):** Command safety scanner needs a comprehensive pattern list. The research identifies categories (command substitution, network access, pipe-to-shell, sensitive file writes) but the exact regex patterns and edge cases need validation during planning. Consider `/gsd:plan-phase --research-phase 3`.

Phases with standard patterns (skip research-phase):
- **Phase 1 (Pack Foundation):** Well-documented TOML parsing with existing `toml` 0.9 crate. Standard serde deserialize pattern.
- **Phase 2 (Registry & Lifecycle):** Standard file CRUD with atomic writes via `tempfile`. Established pattern from v0.0.1.
- **Phase 4 (Remove & Integration):** Standard CRUD + existing hook pattern extended with one enum arm.
- **Phase 5 (Built-in Packs):** Content creation, no technical complexity.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All 6 new crate versions verified via crates.io API. System dependencies documented. Existing crate reuse confirmed against Cargo.toml. |
| Features | MEDIUM | Ecosystem is fragmented — no dominant standard for alias pack sharing. Competitive analysis covers turboalias, oblivion, cheat, ohmyzsh, but the pack concept is novel to aliasman. Anti-features are well-reasoned. |
| Architecture | HIGH | Direct codebase analysis of all existing modules. Component boundaries follow established patterns from v0.0.1. Isolation + merge-at-render is the correct pattern (validated against Homebrew taps, conda managed blocks). |
| Pitfalls | HIGH | zsh internals verified against official docs. Security pitfalls validated against ohmyzsh plugin override patterns and npm dependency conflict patterns. |

**Overall confidence:** HIGH

### Gaps to Address

- **Command safety scanner pattern completeness:** The research identifies dangerous pattern categories but not specific regex implementations. A planning-phase research pass on shell metacharacter patterns would strengthen Phase 3.
- **git2 system dependency friction:** Users without `libgit2` installed will hit build failures. The research recommends making git support an optional compile feature, but the UX of "your build failed because you need to install libgit2" needs validation. Consider whether git install should be deferred entirely to v0.2.
- **Pack namespace vs flat naming:** The research identifies pack-to-pack collisions (Pitfall #11) but presents two solutions (auto-prefix vs author-declared namespace) without a recommendation. This needs a user decision before Phase 1 manifest schema is finalized.
- **`sha2` version ambiguity:** Research notes both `sha2` 0.10.x and 0.11.0. The `crypto-common` ecosystem uses 0.10.x as its stable line. Verify which version is compatible with other crypto-common crates in the dependency graph.

## Sources

### Primary (HIGH confidence)
- crates.io API — All 6 new crate versions verified (`tar`, `flate2`, `sha2`, `semver`, `git2`, `url`)
- Direct codebase inspection — `src/main.rs`, `src/store.rs`, `src/model.rs`, `src/hook.rs`, `src/search.rs`, `src/validation.rs`, `Cargo.toml`
- zsh official documentation (alias section 6.8, history options) — Shell behavior verified
- tempfile crate docs — `NamedTempFile::persist()` atomic write pattern

### Secondary (MEDIUM confidence)
- Oh My Zsh plugin override and loading order — Collision pattern reference
- npm/cli arborist test fixtures — Dependency conflict pattern reference (decided not to follow)
- Homebrew taps documentation — Git-based distribution pattern reference
- cheat/cheat community cheatsheet model — Content distribution reference

### Tertiary (LOW confidence)
- Shell security best practices for untrusted command execution — Accumulated engineering knowledge, needs regex-level validation
- Chezmoi/stow dotfile management patterns — General dotfile tooling reference

---
*Research completed: 2026-05-16*
*Ready for roadmap: yes*
