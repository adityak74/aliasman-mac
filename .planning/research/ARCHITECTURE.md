# Architecture Patterns — aliasman

**Domain:** Rust CLI tool — shell alias manager with Claude Code integration + Alias Library
**Researched:** 2026-05-10 (v0.0.1), 2026-05-16 (v0.1 Alias Library extension)
**Confidence:** HIGH (verified via Context7 for v0.0.1 crates, direct codebase analysis for v0.1)

---

## v0.0.1 Foundation Architecture

### Component Boundary Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         aliasman binary                          │
│                                                                  │
│   ┌───────────┐    ┌──────────────┐    ┌────────────────────────┐ │
│   │  CLI       │    │  AliasStore   │    │  ShellIntegration       │ │
│   │   (clap)   │──▶│   (TOML r/w)  │    │   (read/write configs)  │ │
│   │            │    │   ~/.aliases   │    │   ~/.zshrc / ~/.bashrc │ │
│   └─────┬─────┘    └──────────────┘    └───────────┬────────────┘ │
│         │                                          │              │
│         │          ┌──────────────┐    ┌────────────▼────────────┐ │
│         ├────────▶│ HistoryEngine  │    │  ShellDetector          │ │
│         │          │   (parse zsh/ │    │   ($SHELL, config files) │ │
│         │          │   bash hist)  │    └────────────────────────┘ │
│         │          └──────────────┘                               │
│         │                                                        │
│         │          ┌──────────────────────────────────────────┐   │
│         └────────▶│  HookRunner   (aliasman hook --shell claude│   │
│                   │  reads AliasStore + cwd context,           │   │
│                   │  outputs JSON {additionalContext: "..."}   │   │
│                   └──────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘

External surfaces:
   ~/.aliases               ← AliasStore owns this file (atomic writes)
   ~/.zshrc / ~/.bashrc     ← ShellIntegration appends ONE source line
   ~/.config/aliasman/      ← App config (TOML), managed by ConfigStore
   ~/.claude/settings.json ← HookRunner registers itself here on install
```

### Data Model

**The Alias Record:**

```toml
[[aliases]]
name         = "gs"
command      = "git status"
description = "Quick git status"           # optional, free text
tags         = ["git", "vcs"]               # optional, for filtering
shell        = "all"                        # "all" | "zsh" | "bash"
created_at   = 1715300000                   # Unix timestamp (u64)
updated_at   = 1715300000
source       = "user"                       # "user" | "imported" | "suggested"
```

**The AliasStore File:**
Location: `~/.config/aliasman/aliases.toml` (canonical data).
Derived output: `~/.aliases` (regenerated on every write).

### File Management Strategy

**Atomic Write Protocol:**
Never write shell config files or the aliases file in place. Write to `NamedTempFile` in the same directory, flush, then `persist()` (atomic rename).

**Backup Strategy:**
Before any destructive write to shell config files, create timestamped backups. Keep last 3 per file. Do not backup `~/.aliases` — it is fully regenerated.

**Managed Block Markers:**
```sh
# >>> aliasman >>>
[ -f "$HOME/.aliases" ] && source "$HOME/.aliases"
# <<< aliasman <<<
```

### Claude Hook Architecture

`SessionStart` hook registered in `~/.claude/settings.json`. Reads alias store + cwd context, scores aliases by tag/project relevance, injects top N within a ~500 token budget as Markdown in `additionalContext`.

### Patterns to Follow

1. **Regenerate, Don't Patch** — `~/.aliases` is derived output, regenerated fully on every change.
2. **Managed Block for Shell Configs** — aliasman writes only a delimited source block, never touches anything else in zshrc/bashrc.
3. **Config Colocated with Data** — `~/.config/aliasman/` holds everything.
4. **Hook Outputs Markdown in JSON Envelope** — readable, token-efficient.

### Anti-Patterns to Avoid

1. **In-Place File Edit** — crash during write corrupts config. Use tempfile + atomic rename.
2. **Storing Metadata in Shell Comments** — second source of truth. Keep metadata in TOML only.
3. **Injecting All Aliases into Claude** — waste tokens. Use scoring + budget.
4. **Relying on $SHELL in the Hook** — may not be set. Read from config.
5. **Editing settings.json With String Manipulation** — use serde_json.

---

## v0.1 Alias Library Architecture

### Executive Summary

The Alias Library feature adds a pack-based system for creating, sharing, and installing curated alias collections. The design centers on a **separation of concerns** between user-owned aliases (in `~/.config/aliasman/aliases.toml`) and pack-owned aliases (in `~/.config/aliasman/packs/`). The existing `~/.aliases` shell file becomes a derived output of the merged set of all active aliases from both sources.

**Key design decision: Packs do NOT write to `aliases.toml`.** Instead, they live in isolated directories under `~/.config/aliasman/packs/`. The `render_aliases_file()` function in `store.rs` is extended to read from both the user store and all installed packs, merging them into a single `~/.aliases` output. This keeps pack aliases reversible (uninstall = delete pack directory) and prevents pack aliases from polluting the user's canonical data file.

### Extended Component Boundary Diagram

```
   ┌──────────────────────────────────────────────────────────────┐
   │                         aliasman binary                        │
   │                                                               │
   │    ┌───────────┐                                             │
   │    │  CLI        │                                             │
   │    │   (clap)    │                                             │
   │    └─────┬─────┘                                             │
   │          │                                                    │
   │          ├───┐                                               │
   │          │   ┌▼────────────────┐                             │
   │          │   │  AliasStore      │  (v0.0.1, unchanged)       │
   │          │   │   ~/.config/      │                             │
   │          │   │   aliasman/       │                             │
   │          │   │   aliases.toml    │                             │
   │          │   └────────┬─────────┘                             │
   │          │            │                                        │
   │          │            │                                        │
   │          │   ┌▼────────────────┐                             │
   │          │   │  PackRegistry     │  NEW                       │
   │          │   │   ~/.config/      │                             │
   │          │   │   aliasman/       │                             │
   │          │   │   registry.toml   │                             │
   │          │   └────────┬─────────┘                             │
   │          │            │                                        │
   │          │   ┌▼────────────────┐                             │
   │          │   │  Pack Store       │  NEW                       │
   │          │   │   ~/.config/       │                             │
   │          │   │   aliasman/        │                             │
   │          │   │   packs/           │                             │
   │          │   │   k8s/             │                             │
   │          │   │   terraform/       │                             │
   │          │   │   cicd/            │                             │
   │          │   └────────┬─────────┘                             │
   │          │            │                                        │
   │          │   ┌▼────────────────┐                             │
   │          │   │  AliasMerger      │  NEW (in pack_manager.rs)  │
   │          │   │   (merge user +   │                             │
   │          │   │   pack aliases,   │                             │
   │          │   │   resolve conf.) │                             │
   │          │   └────────┬─────────┘                             │
   │          │            │                                        │
   │          └────┬───────┴──────────────────────────────────────┘
   │               │
   │   ┌▼──────────▼──────────────────────────────────────────┐
   │   │  render_aliases_file() (EXTENDED in store.rs)          │
   │   │  Reads merged alias set, writes ~/.aliases atomically │
   │   └───────────────────────────────────────────────────────┘
   │               │
   │   ┌▼──────────▼──────────────────────────────────────────┐
   │   │  refresh_index() (unchanged, triggered after merge)    │
   │   │  Rebuilds LanceDB index from merged alias set          │
   │   └───────────────────────────────────────────────────────┘
   └──────────────────────────────────────────────────────────────┘

External surfaces (NEW):
   ~/.config/aliasman/registry.toml   ← PackRegistry owns this
   ~/.config/aliasman/packs/*/        ← PackManager owns these directories
```

### Data Model — Alias Packs

**Pack Manifest (`pack.toml`):**

Every alias pack is a directory containing a `pack.toml` manifest.

```toml
[pack]
name         = "k8s"
version      = "0.1.0"
description  = "Kubernetes alias pack for common kubectl operations"
author       = "aliasman"
tags         = ["kubernetes", "k8s", "devops", "kubectl"]

[compatibility]
min_aliasman_version = "0.1.0"

[aliases]
gs = "kubectl get services"
gp = "kubectl get pods"
gd = "kubectl get deployments"
gn = "kubectl get nodes"
lo = "kubectl logs --tail=50"
ex = "kubectl exec -it"
ap = "kubectl apply -f"
de = "kubectl delete"
ru = "kubectl rollout status deployment/"
st = "kubectl describe pod"
```

**Pack Registry (`registry.toml`):**

Tracks installed packs, versions, and metadata.

```toml
[[installed]]
name = "k8s"
version = "0.1.0"
installed_at = 1715300000
source = "file"
alias_count = 10
conflicts = ["gs", "st"]

[[installed]]
name = "terraform"
version = "0.1.0"
installed_at = 1715301000
source = "url"
source_url = "https://example.com/terraform-pack.tar.gz"
alias_count = 8
conflicts = []
```

**Pack Directory Structure:**

```
~/.config/aliasman/packs/
  k8s/
    pack.toml
  terraform/
    pack.toml
  cicd/
    pack.toml
```

### New Components

#### 1. PackManifest (`src/pack_manifest.rs`)

Parses and validates `pack.toml` files.

| Function | Purpose |
|----------|---------|
| `parse_manifest(path: &Path) -> Result<PackManifest>` | Read and deserialize |
| `validate_manifest(&PackManifest) -> Result<(), PackError>` | Validate fields, version, alias names |

Uses existing `validation::validate_alias_name()` and `validation::is_protected_name()` for alias name checks.

#### 2. PackRegistry (`src/pack_registry.rs`)

Manages `registry.toml` — tracks installed packs.

| Function | Purpose |
|----------|---------|
| `load_registry() -> PackRegistry` | Read registry, return empty if missing |
| `save_registry(&PackRegistry)` | Atomic write |
| `is_installed(&PackRegistry, name: &str) -> bool` | Lookup |
| `add_entry(&mut PackRegistry, entry: PackEntry)` | Record installation |
| `remove_entry(&mut PackRegistry, name: &str)` | Unregister |

#### 3. PackManager (`src/pack_manager.rs`)

Orchestrates pack lifecycle. Includes the AliasMerger submodule.

| Function | Purpose |
|----------|---------|
| `create_pack(...)` | Generate pack directory with pack.toml |
| `export_pack(store, filter, output_dir)` | Export user aliases into a pack |
| `install_pack_from_file(tar_path)` | Extract, validate, install |
| `install_pack_from_url(url)` | Download, extract, validate, install |
| `install_pack_from_git(repo_url)` | Clone, validate, install |
| `install_pack_from_dir(dir_path)` | Install from local directory |
| `remove_pack(name)` | Uninstall: remove dir, update registry, regenerate |
| `merge_aliases(user_store, registry, packs_dir)` | Merge all sources, resolve conflicts |

**Install flow:**
```
1. Extract/copy pack to temp dir
2. Parse pack.toml via PackManifest
3. Validate manifest + each alias name
4. Detect conflicts with user aliases (user wins) and other packs (newest wins)
5. Copy pack to ~/.config/aliasman/packs/{name}/
6. Update registry.toml
7. Call merge_aliases() → render_aliases_file() → refresh_index()
8. Report InstallResult (added, skipped, conflicts)
```

### Modified Existing Components

| Component | File | Change |
|-----------|------|--------|
| `AliasSource` enum | `src/model.rs` | Add `Pack` variant |
| `render_aliases_file()` | `src/store.rs` | Param: `&AliasStore` → `&[AliasRecord]` |
| `score_alias()` | `src/hook.rs` | Add `AliasSource::Pack => score += 0.5` arm |
| `Cli` enum | `src/main.rs` | Add `Pack` subcommand with `PackCommands` |
| `regenerate_aliases()` | `src/main.rs` | Call `AliasMerger::merge_aliases()` before render |
| `lib.rs` | `src/lib.rs` | Add 3 new module declarations |

**Backward compatibility:** Existing `aliases.toml` files will not have `source = "pack"`. Deserialization is safe — `Pack` is only written by the merger, never read from user files. The `render_aliases_file()` signature change requires updating all callers but the function body is unchanged.

### Conflict Resolution Strategy

**Priority (highest to lowest):**
1. User aliases from `aliases.toml` — always win
2. Pack aliases by install time — newest pack wins
3. Alphabetical pack name — deterministic tie-break

**Conflict reporting:**
```
Installed pack 'k8s' v0.1.0
   8 aliases added
   2 aliases skipped (name conflict with your aliases):
     - gs (your: 'git status', pack: 'kubectl get services')
     - st (your: 'ssh -t', pack: 'kubectl describe pod')
```

### Data Flow: Pack Install

```
aliasman pack install --file k8s-pack.tar.gz
  ↓
PackManager::install_pack_from_file()
  ↓ Extract to temp dir
  ↓ Parse pack.toml
  ↓ Validate (names, protected, semver)
  ↓ Conflict detection vs user aliases + other packs
  ↓ Copy to ~/.config/aliasman/packs/k8s/
  ↓ Update registry.toml
  ↓ AliasMerger::merge_aliases()
  ↓ render_aliases_file(&merged) → ~/.aliases
  ↓ refresh_index(merged) → LanceDB
  ↓ Print InstallResult
```

### Data Flow: Pack Remove

```
aliasman pack remove --name k8s
  ↓
PackManager::remove_pack("k8s")
  ↓ Lookup in registry
  ↓ Remove ~/.config/aliasman/packs/k8s/
  ↓ Remove from registry.toml
  ↓ AliasMerger::merge_aliases() (without k8s)
  ↓ render_aliases_file(&merged) → ~/.aliases
  ↓ refresh_index(merged) → LanceDB
  ↓ Print result
```

### CLI Commands

```rust
aliasman pack create   --name k8s --version 0.1.0 --description "..." --author "me" --tag kubectl --output ./k8s-pack/
aliasman pack export   --name mypack --tag git --output ./mypack/
aliasman pack install  --file pack.tar.gz   | --url https://...  | --git git@...  | --dir ./pack/
aliasman pack list
aliasman pack remove   --name k8s
```

### File System Layout

```
~/.config/aliasman/
  aliases.toml            # User's personal aliases (UNCHANGED by packs)
  registry.toml           # NEW: Installed pack tracking
  index/                  # LanceDB (unchanged, re-indexed on pack changes)
  packs/                  # NEW: Installed pack directories
    k8s/
      pack.toml
    terraform/
      pack.toml
    cicd/
      pack.toml
```

### Build Order (Dependency Graph)

```
Phase 1: Foundation
  ├── pack_manifest.rs       (PackManifest, parse/validate)
  └── model.rs (modified)    (AliasSource::Pack variant)

Phase 2: Registry
  ├── pack_registry.rs       (PackRegistry, load/save/queries)
  └── pack_manager.rs        (PackManager shell, install/remove/list)

Phase 3: Merge
  ├── pack_manager.rs        (AliasMerger::merge_aliases)
  ├── store.rs (modified)    (render_aliases_file param change)
  └── main.rs (modified)     (Pack subcommand wiring)

Phase 4: Integration
  ├── hook.rs (modified)     (score_alias handles Pack source)
  └── main.rs (modified)     (regenerate_aliases uses merger)

Phase 5: Built-in Packs
  ├── packs/k8s/             (curated pack directory)
  ├── packs/terraform/       (curated pack directory)
  └── packs/cicd/            (curated pack directory)
```

**Critical path:** `pack_manifest.rs` first (every pack component depends on it). `model.rs` changes second (needed for compilation). `pack_registry.rs` third. `pack_manager.rs` fourth (depends on all three). Store/hook/main modifications last.

### Anti-Patterns to Avoid

1. **Writing Pack Aliases into aliases.toml** — makes uninstall ambiguous, entangles user/pack data. Keep isolated, merge at render time.
2. **Pack Aliases Override User Aliases Silently** — user loses custom aliases. User always wins, conflicts reported explicitly.
3. **Monolithic Pack Format** — single file with no directory. Use directory + manifest for extensibility (README, license, git-clonable).
4. **Lazy Re-index on Pack Install** — stale search results. Trigger `refresh_index()` immediately, same as CRUD.

### Scalability

| Concern | 5 packs, 50 aliases | 20 packs, 200 aliases |
|---------|---------------------|----------------------|
| Merge time | <1ms | <5ms |
| Registry parse | <1ms | <1ms |
| Conflict detection | <1ms | <3ms |
| Re-index (background) | ~500ms | ~2s |

No architectural changes needed at expected scale.

---

## Sources

- Direct codebase analysis: `src/main.rs`, `src/store.rs`, `src/model.rs`, `src/hook.rs`, `src/search.rs`, `src/validation.rs`, `src/import.rs`, `Cargo.toml`
- Existing architecture research: `.planning/research/ARCHITECTURE.md` (v0.0.1), `.planning/research/STACK.md`
- npm package.json pattern for manifest design
- Homebrew formula structure for pack directory layout
- conda/mamba managed block pattern for isolation principles
