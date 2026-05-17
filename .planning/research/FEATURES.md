# Feature Landscape: Alias Library (v0.1)

**Domain:** Shareable alias/command packs for aliasman
**Researched:** 2026-05-16
**Confidence:** MEDIUM (ecosystem is fragmented; no dominant standard for alias pack sharing)

## Table Stakes

Features users expect from a pack system. Missing = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Pack create | Users need to group aliases into named collections | Low | `aliasman pack create k8s` — creates a pack directory with a manifest |
| Pack add-alias | Add existing aliases to a pack | Low | `aliasman pack add-alias k8s --name ks --command "kubectl get pods"` |
| Pack export | Generate a shareable file from a pack | Low | `aliasman pack export k8s` — produces a single TOML file |
| Pack install (file) | Install a pack from a local TOML file | Low | `aliasman pack install ./k8s.toml` — merges aliases into store |
| Pack install (URL) | Install a pack from a remote URL | Medium | `aliasman pack install https://example.com/k8s.toml` — downloads then installs |
| Pack install (git) | Install a pack from a git repository | Medium | `aliasman pack install git@github.com:user/aliasman-k8s.git` — clones, reads manifest |
| Pack list | Show all installed packs and their aliases | Low | `aliasman pack list` — table of pack name, version, alias count, source |
| Pack remove | Uninstall a pack and its aliases | Medium | `aliasman pack remove k8s` — removes pack metadata and aliases marked as pack-owned |
| Pack manifest | Structured metadata file per pack | Low | TOML file with name, version, description, author, alias list |

## Differentiators

Features that set aliasman apart from other alias tools. Not expected, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Pack conflict detection | Prevents silent alias overwrites when two packs define the same name | Medium | Check name collision before install; show which pack owns the conflicting alias |
| Pack override priority | User aliases > pack aliases, with clear precedence rules | Medium | When a user manually creates an alias matching a pack alias, user version wins |
| Built-in curated packs | Ships with k8s, terraform, Docker, CI/CD packs out of the box | Medium | Pre-bundled TOML files in `~/.config/aliasman/builtin_packs/` |
| Pack source tracking | Every alias records which pack it came from | Low | Extend `AliasSource` enum with `Pack(pack_name)`. Enables targeted removal and conflict display |
| Pack update from git | `aliasman pack update k8s` pulls latest from the original git source | Medium | Requires storing git remote URL per pack; simple `git pull` on cached clone |
| Pack tag inheritance | Pack-level tags auto-applied to all aliases in pack | Low | Pack manifest has `tags = ["k8s", "kubernetes"]`; each alias inherits them |

## Anti-Features

Features to explicitly NOT build in v0.1.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Cloud pack registry | Requires server infrastructure, auth, rate limiting, moderation. Out of scope for v0.1. | Git repos + URLs are sufficient distribution. Users can host packs on GitHub. |
| Full dependency resolver | npm-style transitive dependency chains between packs add massive complexity for minimal value. Alias packs are small and self-contained. | Allow a pack manifest to list "suggested" related packs, but do not auto-install them. |
| Pack version locking with semver ranges | Overkill for alias collections. Users rarely need `^1.2` semantics for aliases. | Store a simple version string. `pack update` fetches latest. No range resolution. |
| Pack search/discovery API | Requires a central index or registry. No cloud = no search. | Document a GitHub topic (`aliasman-pack`) so users can search GitHub manually. |
| Pack validation schema | Formal schema validation adds complexity. TOML structure is self-validating. | Basic manifest field checks (name, version required). Skip formal schema. |
| Pack permission system | No auth model, no cloud, no need for permissions. | N/A — packs are local files or public git repos. |
| Automatic pack conflict resolution | Deciding which pack wins silently is dangerous. Users should make the choice. | Surface conflicts explicitly. Require `--force` to overwrite. |

## Feature Dependencies

```
Pack create → Pack manifest (manifest format must be defined first)
Pack add-alias → Pack create (need a pack to add aliases into)
Pack export → Pack create + Pack add-alias (need aliases in pack to export)
Pack install (file) → Pack manifest (must parse manifest format)
Pack install (URL) → Pack install (file) (download then delegate to file install)
Pack install (git) → Pack install (file) (clone then delegate to file install)
Pack list → Pack install (need installed pack metadata to list)
Pack remove → Pack source tracking (need to know which aliases belong to which pack)
Pack update → Pack install (git) + Pack source tracking
Built-in packs → Pack install (file) (pre-bundled TOML files installed like user packs)
Pack conflict detection → Pack source tracking + Pack install
Pack override priority → Pack source tracking + Alias CRUD (existing update/delete)
Pack tag inheritance → Pack manifest + Pack add-alias
```

## Detailed Feature Analysis

### Pack Manifest Format

The pack manifest is the backbone of the entire system. Every pack operation reads or writes this.

**Recommended format (TOML):**

```toml
[pack]
name = "k8s"
version = "1.0.0"
description = "Kubernetes alias pack for common kubectl operations"
author = "Aditya Karnam"
tags = ["k8s", "kubernetes", "devops"]

[git]
url = "https://github.com/user/aliasman-k8s.git"

[[aliases]]
name = "ks"
command = "kubectl get pods"
description = "List all pods"
tags = ["k8s", "pods"]

[[aliases]]
name = "ksn"
command = "kubectl get pods -n {{namespace}}"
description = "List pods in namespace"
tags = ["k8s", "pods", "namespace"]
```

**Key design decisions:**
- TOML matches existing aliasman store format (same serialization library, zero new dependencies)
- `[[aliases]]` array mirrors `AliasRecord` fields (reuse model)
- `{{namespace}}` style placeholders allow parameterized aliases (differentiator)
- `[git]` section is optional — packs from files have no git source
- `tags` at pack level enable tag inheritance on install

### Pack Install Flow

```
User runs: aliasman pack install <source>

Source = file  → Read TOML → Parse manifest → Validate → Merge aliases into store
Source = URL   → Download to temp → Read TOML → Parse manifest → Validate → Merge
Source = git   → Clone to ~/.config/aliasman/packs/<name>/ → Read pack.toml → Validate → Merge

Merge step:
  For each alias in pack:
    If alias name not in store → Add with source=Pack(pack_name)
    If alias name already in store:
      If existing source=User → Skip, log conflict
      If existing source=Pack(old_pack) → Log conflict, require --force to overwrite
      If existing source=Imported/Suggested → Skip, log conflict

After merge:
  Regenerate ~/.aliases
  Refresh semantic search index
  Print install summary (X aliases added, Y conflicts skipped)
```

### Pack Remove Flow

```
User runs: aliasman pack remove <pack_name>

1. Look up pack metadata from ~/.config/aliasman/packs/<name>/
2. Find all aliases in store where source=Pack(pack_name)
3. Delete those aliases from store
4. Remove pack directory from packs/
5. Regenerate ~/.aliases
6. Refresh semantic search index
7. Print removal summary (X aliases removed)
```

### Conflict Resolution Model

**Precedence order (highest to lowest):**
1. User-created aliases (`source=User`) — always wins
2. Pack-installed aliases (`source=Pack`) — can be overwritten by newer pack with `--force`
3. Imported aliases (`source=Imported`) — lower priority than pack aliases
4. Suggested aliases (`source=Suggested`) — lowest priority

**On install conflict:**
- Default: skip conflicting alias, log warning, continue with rest
- With `--force`: overwrite conflicting alias, attribute to new pack
- With `--skip-existing`: skip all conflicts silently (batch install mode)

### Built-in Packs

Pre-bundled packs shipped with aliasman installation. Stored in `~/.config/aliasman/builtin_packs/` after `aliasman init`.

**Recommended initial set:**
| Pack | Aliases | Target User |
|------|---------|-------------|
| k8s | 15-20 common kubectl shortcuts | Kubernetes developers |
| terraform | 10-15 tf plan/apply/destroy shortcuts | Infrastructure engineers |
| docker | 10-15 docker/compose shortcuts | Container developers |
| git-advanced | 10-15 git beyond basics (bisect, stash, rebase) | Power git users |
| cicd | 5-10 GitHub Actions, CI shortcuts | CI/CD operators |

**Installation:** During `aliasman init --apply`, offer to install built-in packs. User selects which ones. They are installed like any other pack but marked as `builtin=true` in metadata.

### Source Tracking Extension

The existing `AliasSource` enum needs a new variant:

```rust
pub enum AliasSource {
    User,
    Imported,
    Suggested,
    Pack(String),  // pack name, e.g., "k8s"
}
```

This enables:
- `pack list` to count aliases per pack
- `pack remove` to find and delete pack-owned aliases
- `list` to show pack origin in the source column
- Conflict detection during install

## Expected User Behavior

### Primary Workflow

```bash
# 1. Create a pack from existing aliases
aliasman pack create my-tools
aliasman pack add-alias my-tools --name gs --command "git status"
aliasman pack add-alias my-tools --name ll --command "ls -lah"

# 2. Export for sharing
aliasman pack export my-tools  # → writes ~/.config/aliasman/packs/my-tools/pack.toml

# 3. Share (user copies file, pushes to GitHub, emails, etc.)

# 4. Install on another machine
aliasman pack install https://github.com/user/my-tools/raw/main/pack.toml

# 5. Manage
aliasman pack list          # see all installed packs
aliasman pack update k8s    # pull latest from git source
aliasman pack remove docker # uninstall and remove aliases
```

### Secondary Workflow (Built-in Packs)

```bash
# During init
aliasman init --apply
# Prompt: "Install built-in packs? [k8s, terraform, docker, git-advanced, cicd]"
# User selects: k8s, docker

# Later
aliasman pack install builtin:terraform  # install additional built-in pack
```

## MVP Recommendation

Prioritize for v0.1:

1. **Pack manifest format** — Define TOML schema, implement parser
2. **Pack create + add-alias** — Core creation workflow
3. **Pack export** — Generate shareable TOML from pack
4. **Pack install (file)** — Install from local TOML
5. **Pack install (URL)** — Download + install from URL
6. **Pack list** — Show installed packs
7. **Pack remove** — Uninstall packs and their aliases
8. **Source tracking extension** — `AliasSource::Pack(String)` variant
9. **Built-in k8s + docker packs** — Two curated packs to prove the model

Defer to v0.2:
- **Pack install (git)** — More complex (clone, track, update). Can be added after file/URL model is stable.
- **Pack update** — Requires git source tracking.
- **Pack conflict detection with UI** — Basic conflict logging is enough for v0.1. Interactive resolution is v0.2.
- **Parameterized aliases** (`{{namespace}}` placeholders) — Nice-to-have, not blocking.
- **Remaining built-in packs** (terraform, git-advanced, cicd) — Can be community-contributed.

## Sources

- **turboalias** (mcdominik/turboalias, 10 stars): JSON config, Git sync, category-based organization. No pack model. Reference for cross-machine sync patterns. [GitHub](https://github.com/mcdominik/turboalias)
- **oblivion** (sealmove/oblivion, 9 stars): INI-based config, interface/command groups. No sharing model. Reference for command grouping patterns. [GitHub](https://github.com/sealmove/oblivion)
- **cheat** (cheat/cheat): Community cheatsheets from separate repo, plain-text files, conf.yml paths, read-only upstream with transparent copy-to-writable on edit. Reference for community content distribution. [GitHub](https://github.com/cheat/cheat)
- **Homebrew taps**: Git repo clones into `$(brew --repository)/Library/Taps/`. Naming convention: `homebrew-something`. Reference for git-based collection distribution. [Docs](https://docs.brew.sh/Taps)
- **npm packages**: package.json with semver dependencies, multiple dependency types. Reference for dependency model (decided NOT to follow for aliasman). [Docs](https://docs.npmjs.com)
- **Oh-my-zsh plugins**: `custom/plugins/` directory, `XYZ.plugin.zsh` naming. Reference for extensible plugin architecture. [GitHub](https://github.com/ohmyzsh/ohmyzsh)
- **rcm + dotfiles**: Git repo cloned locally, symlinks via `rcup`, `.local` suffix for overrides. Reference for layering shared base with personal overrides. [GitHub](https://github.com/thoughtbot/dotfiles)
