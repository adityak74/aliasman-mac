# Requirements — v0.1 Alias Library

**Project:** aliasman
**Milestone:** v0.1 Alias Library — shareable alias packs for common dev tools
**Date:** 2026-05-17

## Milestone Goal

Developers can create, share, and install reusable alias packs — curated collections of aliases for common toolchains (k8s, docker, terraform) — without manually copying aliases between machines.

## Requirements

### Pack Format & Creation

- [ ] **PACK-01**: User can create a new pack directory with `aliasman pack create <name>` — generates a structured directory with a `pack.toml` manifest
- [ ] **PACK-02**: Pack manifest (`pack.toml`) includes name, version, description, author, and `format_version` fields
- [ ] **PACK-03**: User can add aliases to a pack with `aliasman pack add <pack-name> <alias-name> "<command>"`
- [ ] **PACK-04**: User can export a pack as a single shareable `.toml` file with `aliasman pack export <pack-name>`

### Pack Installation

- [ ] **INST-01**: User can install a pack from a local file with `aliasman pack install <file.toml>`
- [ ] **INST-02**: User can install a pack from a URL with `aliasman pack install --url <https-url>`
- [ ] **INST-03**: Pack install shows a dry-run preview listing all aliases before applying
- [ ] **INST-04**: Command safety scanner flags dangerous patterns (command substitution, pipe-to-shell, network access, destructive commands) during install
- [ ] **INST-05**: Safety warnings are soft — user can override with `--force` flag
- [ ] **INST-06**: Pre-install collision scan detects pack aliases that conflict with existing user aliases
- [ ] **INST-07**: User aliases always win over pack aliases in collision — pack alias is skipped unless `--force`
- [ ] **INST-08**: Pack install uses two-phase apply (validate all aliases first, write all only if validation passes)

### Pack Management

- [ ] **MGMT-01**: User can list all installed packs with `aliasman pack list`, showing name, version, source, and alias count
- [ ] **MGMT-02**: User can remove a pack with `aliasman pack remove <name>` — uninstalls pack and its aliases
- [ ] **MGMT-03**: Pack removal preserves user-modified aliases (tracked via `modified_by_user` flag)
- [ ] **MGMT-04**: Pack aliases are tracked with `AliasSource::Pack(String)` for provenance
- [ ] **MGMT-05**: Installed packs are tracked in a local registry (`registry.toml`) under `~/.config/aliasman/`

### Integration

- [ ] **INTG-01**: Pack aliases merge with user aliases at render time into `~/.aliases` (merge-at-render, not storage-time)
- [ ] **INTG-02**: Claude Code hook scores pack aliases alongside user/imported aliases
- [ ] **INTG-03**: LanceDB semantic index re-indexes automatically after pack install/remove
- [ ] **INTG-04**: `aliasman list` output groups aliases by source (user, pack, imported, suggested)

### Built-in Packs

- [ ] **CONT-01**: Ship with a curated k8s pack (15-20 kubectl aliases for common operations)
- [ ] **CONT-02**: Ship with a curated docker pack (10-15 docker/compose aliases)
- [ ] **CONT-03**: Built-in packs are shipped as files in `~/.config/aliasman/builtin_packs/` (not embedded in binary)

## Future Requirements (v0.2+)

- Pack install from git repos (`aliasman pack install --git <repo-url>`)
- Pack update — detect and upgrade installed packs to newer versions
- Pack dependency declarations (pack A requires pack B)
- Central pack registry with search/discovery
- Parameterized aliases (`{{namespace}}` placeholders)
- Interactive conflict resolution UI
- Additional built-in packs (terraform, CI/CD, git-advanced)

## Out of Scope

- **Central registry server** — No server infrastructure, no auth, no moderation. Packs are distributed as files/URLs.
- **Dependency resolution** — Packs are independent. No semver range enforcement, no transitive installs.
- **Team sharing workflows** — No built-in team/org features. Users share pack files directly.
- **Cloud sync** — Packs are installed locally. No cross-machine sync service.
- **Pack validation service** — No remote validation. Safety scanner runs locally at install time.

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| PACK-01 | Phase 7 | Pending |
| PACK-02 | Phase 7 | Pending |
| PACK-03 | Phase 7 | Pending |
| PACK-04 | Phase 7 | Pending |
| MGMT-04 | Phase 7 | Pending |
| MGMT-01 | Phase 8 | Pending |
| MGMT-05 | Phase 8 | Pending |
| INST-01 | Phase 9 | Pending |
| INST-02 | Phase 9 | Pending |
| INST-03 | Phase 9 | Pending |
| INST-04 | Phase 9 | Pending |
| INST-05 | Phase 9 | Pending |
| INST-06 | Phase 9 | Pending |
| INST-07 | Phase 9 | Pending |
| INST-08 | Phase 9 | Pending |
| MGMT-02 | Phase 10 | Pending |
| MGMT-03 | Phase 10 | Pending |
| INTG-01 | Phase 10 | Pending |
| INTG-02 | Phase 10 | Pending |
| INTG-03 | Phase 10 | Pending |
| INTG-04 | Phase 10 | Pending |
| CONT-01 | Phase 11 | Pending |
| CONT-02 | Phase 11 | Pending |
| CONT-03 | Phase 11 | Pending |

**Coverage:** 24/24 requirements mapped (100%)

---
*Requirements defined: 2026-05-17*
*Decisions: Flat names + collision detection | File + URL install only | Soft safety warnings with --force*
