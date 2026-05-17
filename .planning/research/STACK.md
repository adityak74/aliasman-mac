# Technology Stack Additions: Alias Library (v0.1)

**Project:** aliasman — Rust CLI alias manager for macOS/Linux
**Feature:** Shareable alias packs (v0.1 Alias Library)
**Researched:** 2026-05-16
**Confidence:** HIGH (all crate versions verified via crates.io API)

---

## Existing Stack (v0.0.1 baseline — no changes needed)

| Crate | Version | Role |
|-------|---------|------|
| `clap` | 4 (derive) | CLI framework |
| `toml` | 0.9 | Alias store serialization |
| `serde` / `serde_json` | 1 | Serialization |
| `reqwest` | 0.12 (json) | HTTP client (already present) |
| `tokio` | 1 (macros, rt-multi-thread) | Async runtime (already present) |
| `tempfile` | 3 | Atomic writes (already present) |
| `dirs` | 5 | Home directory resolution (already present) |
| `lancedb` | 0.29 | Vector search (unchanged) |
| `rmcp` | 1 (server, macros, schemars) | MCP server (unchanged) |

**Key observation:** `reqwest` is already a dependency with `json` features. `tokio` is already present. HTTP-based pack downloads can reuse existing infrastructure without adding new crates.

---

## New Crates for Alias Library

### 1. Pack Format: TOML (no new crate)

**Decision:** Pack files use TOML. The existing `toml` 0.9 crate handles serialization/deserialization. No new dependency.

**Rationale:** The alias store already uses TOML. A pack is a superset of the store format — it adds a `[metadata]` section with pack-level fields (name, version, description, author) alongside the existing `[[aliases]]` array. Users can hand-edit pack files, tools in the ecosystem understand TOML, and the project already has TOML deserialization infrastructure.

**Pack file structure (`pack.toml`):**

```toml
[metadata]
name = "kubernetes"
version = "0.1.0"
description = "Common kubectl aliases for Kubernetes development"
author = "aliasman"

[options]
# Optional: conflict resolution strategy when pack alias name collides with user alias
on_conflict = "skip"       # "skip" | "overwrite" | "prompt" (default: "skip")

[[aliases]]
name = "kget"
command = "kubectl get"
description = "Quick kubectl get"
tags = ["kubectl", "kubernetes"]
shell = "all"

[[aliases]]
name = "klogs"
command = "kubectl logs -f"
description = "Follow pod logs"
tags = ["kubectl", "kubernetes", "logs"]
shell = "all"
```

**Alt considered and rejected:**
- **JSON pack format** — Less human-editable. TOML is the project standard. No benefit.
- **YAML pack format** — Requires `serde_yaml` crate (adds a dependency). YAML's ambiguity (indentation, anchors) makes it worse for programmatic consumption.
- **Custom binary format** — Overkill. Packs are small text files. Human readability is a feature.

### 2. Semantic Versioning for Packs: `semver` 1.0.28

**Why:** Pack versions follow semver (`0.1.0`, `1.0.0-beta.1`). The `semver` crate by dtolnay provides parsing, comparison, and requirement matching (`^0.1`, `>=0.2.0, <1.0.0`). This enables version constraint checking during install and update operations.

```toml
[dependencies]
semver = "1.0"
```

```rust
use semver::{Version, VersionReq};

// Parse pack version
let v = Version::parse("0.1.0")?;

// Check if installed version meets requirement
let req = VersionReq::parse(">=0.1.0")?;
if req.matches(&v) { /* compatible */ }
```

**Version source:** crates.io API confirms `semver` 1.0.28 is the latest release.

**Alt considered and rejected:**
- **Rolling our own version parsing** — Semver has edge cases (pre-release ordering, build metadata, comparator operators). The dtolnay crate is authoritative and well-tested.
- **`cargo-semver-checks`** — Designed for API compatibility linting between crate versions, not for parsing/comparing version strings at runtime. Wrong tool.

### 3. Pack Distribution — Git Clone: `git2` 0.20.4

**Why:** Users should be able to install packs from a git repo URL (`aliasman pack install git@github.com:user/k8s-pack.git` or `https://github.com/user/k8s-pack.git`). `git2` provides Rust bindings to libgit2 for cloning, fetching, and checking out repositories.

```toml
[dependencies]
git2 = { version = "0.20", features = ["https"] }
```

**Feature flag strategy:** Enable only `https` by default. Add `ssh` as an optional feature behind a compile flag (`--features ssh`) to avoid pulling in OpenSSL/SSH dependencies for users who only need HTTPS-based pack installs.

```rust
use git2::Repository;

fn clone_pack_repo(url: &str, dest: &std::path::Path) -> anyhow::Result<()> {
    let repo = Repository::clone(url, dest)?;
    Ok(())
}
```

**Version source:** crates.io API confirms `git2` 0.20.4 is the latest release.

**Alt considered and rejected:**
- **Spawning `git` CLI via `std::process::Command`** — Simpler but fragile: requires git to be installed on the user's system, platform-specific path issues, harder to parse output reliably. `git2` is self-contained and cross-platform.
- **`git2` with both `https` and `ssh` features** — SSH adds OpenSSL/libssh2 system dependencies. Keep it optional to minimize install friction. Users who need SSH can enable it.

**System dependency warning:** `git2` links against `libgit2`. On macOS this is typically available via Homebrew (`libgit2`). The build may require `pkg-config`. This is a documented build requirement, not a blocker.

### 4. Pack Distribution — HTTP Download: (no new crate)

**Why:** Users should install packs from a URL (`aliasman pack install https://example.com/packs/k8s-0.1.0.tar.gz`). The existing `reqwest` 0.12 crate handles HTTP GET requests. No new dependency needed.

**Expected pack download flow:**
1. `reqwest::get(url)` to fetch the tarball
2. Write bytes to a temp file via `tempfile` (already a dependency)
3. Verify SHA-256 checksum (see below)
4. Extract with `tar` (see below)

### 5. Pack Archive Extraction: `tar` 0.4.45 + `flate2` 1.1.9

**Why:** Distributed packs arrive as `.tar.gz` archives. The `tar` crate handles tar archive reading/extraction. `flate2` provides gzip decompression (required as a `Read` wrapper for gzipped tarballs).

```toml
[dependencies]
tar = "0.4"
flate2 = "1.1"
```

```rust
use flate2::read::GzDecoder;
use std::fs::File;

fn extract_pack(tarball_path: &str, dest: &std::path::Path) -> anyhow::Result<()> {
    let file = File::open(tarball_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    archive.unpack(dest)?;
    Ok(())
}
```

**Version source:** crates.io API confirms `tar` 0.4.45 and `flate2` 1.1.9 are the latest releases.

**Alt considered and rejected:**
- **`zip` crate** — Zip is common on Windows but tar.gz is the standard in the Rust/Unix ecosystem (crates.io uses `.crate` which is a `.tar.gz`). Matches user expectations for CLI tool distributions.
- **`zstd` compression** — Newer and faster, but tar.gz has universal tooling support (`tar -xzf`). Users can inspect pack contents with standard shell tools.

### 6. Checksum Verification: `sha2` 0.11.0

**Why:** Pack integrity must be verified after download. SHA-256 checksums prevent corrupted or tampered packs from being installed. The `sha2` crate is the standard Rust implementation of SHA-2.

```toml
[dependencies]
sha2 = "0.10"
```

**Note on versioning:** The `sha2` crate follows a different version scheme — 0.10.x is current (it is part of the `crypto-common` ecosystem which uses 0.10.x as its stable line). Version 0.11.0 is the latest per crates.io.

```rust
use sha2::{Digest, Sha256};

fn verify_checksum(data: &[u8], expected: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let actual = format!("{:x}", result);
    actual == expected
}
```

**Version source:** crates.io API confirms `sha2` 0.11.0 is the latest release.

**Alt considered and rejected:**
- **`hmac`** — Designed for keyed hash authentication, not simple integrity checksums. SHA-256 is sufficient.
- **Spawning `shasum` CLI** — Platform-dependent (`shasum -a 256` on macOS, `sha256sum` on Linux). `sha2` is cross-platform and pure Rust.

### 7. URL Parsing: `url` 2.5.8

**Why:** Pack install sources can be paths, HTTP URLs, or git URLs. The `url` crate provides robust URL parsing, scheme detection, and component extraction (host, path, query) to route install requests to the correct handler (file vs HTTP vs git).

```toml
[dependencies]
url = "2.5"
```

```rust
use url::Url;

fn parse_pack_source(source: &str) -> PackSource {
    let parsed = Url::parse(source);
    match parsed {
        Ok(u) => match u.scheme() {
            "file" => PackSource::Local(u.to_file_path().unwrap()),
            "https" | "http" => {
                if u.path().ends_with(".git") {
                    PackSource::Git(source.to_string())
                } else {
                    PackSource::Http(source.to_string())
                }
            }
            _ => PackSource::Git(source.to_string()), // git@... format
        },
        Err(_) => PackSource::Local(std::path::PathBuf::from(source)),
    }
}
```

**Version source:** crates.io API confirms `url` 2.5.8 is the latest release.

---

## New Dependency Summary

```toml
[dependencies]
# Pack versioning
semver = "1.0"

# Pack distribution — git clone
git2 = { version = "0.20", features = ["https"] }

# Pack archive extraction
tar = "0.4"
flate2 = "1.1"

# Pack integrity verification
sha2 = "0.10"

# URL parsing for pack source routing
url = "2.5"
```

**Crates NOT needed (reusing existing):**

| Capability | Existing Crate | Why No Addition |
|-----------|---------------|-----------------|
| HTTP download | `reqwest` 0.12 | Already in dependencies |
| Async runtime | `tokio` 1 | Already in dependencies |
| TOML serialization | `toml` 0.9 | Already in dependencies |
| JSON serialization | `serde_json` 1 | Already in dependencies |
| Atomic file writes | `tempfile` 3 | Already in dependencies |
| Home dir resolution | `dirs` 5 | Already in dependencies |

---

## Pack Registry Design (no new crate)

**Local registry:** A TOML file at `~/.config/aliasman/packs.toml` tracking installed packs. Uses existing `toml` crate.

```toml
[[installed]]
name = "kubernetes"
version = "0.1.0"
source = "https://github.com/aliasman/packs/raw/main/kubernetes-0.1.0.tar.gz"
checksum = "a1b2c3d4..."
installed_at = 1715300000
pack_file = "/Users/user/.config/aliasman/packs/kubernetes/pack.toml"
```

**Remote registry (deferred to v0.2):** A JSON index file hosted on GitHub (e.g., `https://aliasman.packs/index.json`). Fetched via existing `reqwest`. Not needed for v0.1 MVP — v0.1 supports direct install from URL/git/file only. A registry is a convenience layer on top of that.

---

## Build Dependency Notes

| New Crate | System Dependency | Resolution |
|-----------|-------------------|------------|
| `git2` | `libgit2` + `pkg-config` | `brew install libgit2` on macOS. Document in install guide. |
| `flate2` | `libz` (zlib) | Pre-installed on macOS. May need `zlib1g-dev` on Debian/Ubuntu. |
| `tar`, `sha2`, `semver`, `url` | None | Pure Rust. |

**Risk:** `git2`'s libgit2 dependency is the only new system-level requirement. If a user does not have `libgit2` installed, the build fails. Mitigation: document the prerequisite, and consider making git support an optional feature (`--features git`) so users who only need file/HTTP installs can build without it.

---

## Integration with Existing Stack

**Data model extension:** `AliasRecord` in `model.rs` gains a new `source` variant: `AliasSource::Pack`. The existing `AliasSource` enum (User, Imported, Suggested) expands to five values.

**Store layer extension:** The `AliasStore` struct in `store.rs` gains a `packs` field tracking which packs contributed which aliases. This enables conflict detection (user alias vs pack alias with same name) and pack uninstall (remove only pack-contributed aliases).

**CLI extension:** New top-level subcommand `Pack` with sub-subcommands: `create`, `export`, `install`, `list`, `remove`, `update`. Built on existing `clap` derive infrastructure.

**Search integration:** Pack-installed aliases are indexed into the existing LanceDB vector store alongside user aliases. No change to the search layer — pack aliases are just aliases with `source = "pack"`.

---

## Sources

- `tar` 0.4.45: verified via crates.io API (`https://crates.io/api/v1/crates/tar`)
- `flate2` 1.1.9: verified via crates.io API (`https://crates.io/api/v1/crates/flate2`)
- `sha2` 0.11.0: verified via crates.io API (`https://crates.io/api/v1/crates/sha2`)
- `semver` 1.0.28: verified via crates.io API (`https://crates.io/api/v1/crates/semver`) + Context7 docs (`/dtolnay/semver`)
- `git2` 0.20.4: verified via crates.io API (`https://crates.io/api/v1/crates/git2`) + Context7 docs (`/rust-lang/git2-rs`)
- `url` 2.5.8: verified via crates.io API (`https://crates.io/api/v1/crates/url`)
- `reqwest` 0.12: already in project Cargo.toml
- Pack format rationale: TOML chosen to match existing `toml` 0.9 dependency and project convention
