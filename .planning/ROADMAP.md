# Roadmap: aliasman

## Milestones

- ✅ **v0.0.1 CLI Alias Manager MVP** — Phases 1-6 (shipped 2026-05-16)
- ✅ **v0.1 Alias Library** — Phases 7-11 (shipped 2026-05-19)

## Phases

<details>
<summary>✅ v0.0.1 CLI Alias Manager MVP (Phases 1-6) — SHIPPED 2026-05-16</summary>

- [x] Phase 1: Rust CLI Foundation And Alias Store (1/1 plans) — completed 2026-05-12
- [x] Phase 2: Shell Detection, Init, And Import (1/1 plans) — completed 2026-05-14
- [x] Phase 3: Alias CRUD And Listing (1/1 plans) — completed 2026-05-14
- [x] Phase 4: History Stats And Suggestions (1/1 plans) — completed 2026-05-14
- [x] Phase 5: Claude Hook Integration (1/1 plans) — completed 2026-05-14
- [x] Phase 6: Local Semantic Alias Search With LanceDB And MCP (1/1 plans) — completed 2026-05-15

</details>

<details>
<summary>✅ v0.1 Alias Library (Phases 7-11) — SHIPPED 2026-05-19</summary>

- [x] Phase 7: Pack Foundation (1/1 plans) — completed 2026-05-18
- [x] Phase 8: Pack Registry & Lifecycle (1/1 plans) — completed 2026-05-18
- [x] Phase 9: Pack Install with Safety (2/2 plans) — completed 2026-05-19
- [x] Phase 10: Pack Remove & Integration (1/1 plans) — completed 2026-05-19
- [x] Phase 11: Built-in Packs (1/1 plans) — completed 2026-05-19

</details>

### Phase 7: Pack Foundation
**Goal**: Users can create, populate, and export alias packs as shareable TOML files
**Depends on**: Phase 6
**Requirements**: PACK-01, PACK-02, PACK-03, PACK-04, MGMT-04
**Success Criteria** (what must be TRUE):
    1. User runs `aliasman pack create mypack` and gets a structured directory with a valid `pack.toml` manifest
    2. User can add aliases to the pack with `aliasman pack add mypack kget "kubectl get pods"`
    3. User runs `aliasman pack export mypack` and receives a single shareable `.toml` file
    4. Every alias in the pack is tracked with `AliasSource::Pack("mypack")` for provenance
**Status**: ✅ COMPLETE (2026-05-18)

### Phase 8: Pack Registry & Lifecycle
**Goal**: Installed packs are tracked in a local registry, and the user can list them with metadata
**Depends on**: Phase 7
**Requirements**: MGMT-01, MGMT-05
**Success Criteria** (what must be TRUE):
    1. User runs `aliasman pack list` and sees a table of installed packs
    2. Installed packs are recorded in `~/.config/aliasman/registry.toml`
    3. Registry state survives tool restarts (persists across invocations)
**Status**: ✅ COMPLETE (2026-05-18)

### Phase 9: Pack Install with Safety
**Goal**: Users can safely install packs from local files or URLs with full safety guarantees
**Depends on**: Phase 8
**Requirements**: INST-01, INST-02, INST-03, INST-04, INST-05, INST-06, INST-07, INST-08
**Success Criteria** (what must be TRUE):
    1. User runs `aliasman pack install mypack.toml` and sees a dry-run preview
    2. User installs a pack from URL with `aliasman pack install --url https://...`
    3. Dangerous patterns flagged by safety scanner during install
    4. User can override safety warnings with `--force` flag
    5. Pre-install collision scan detects conflicts, user aliases always win
    6. Install uses two-phase apply — validate first, write atomically
    7. Post-install: aliases regenerated, index refreshed
    8. Pack registered in registry with metadata
**Status**: ✅ COMPLETE (2026-05-19)

### Phase 10: Pack Remove & Integration
**Goal**: Users can uninstall packs cleanly, pack aliases integrate with hook scoring and semantic search
**Depends on**: Phase 9
**Requirements**: MGMT-02, MGMT-03, INTG-01, INTG-02, INTG-03, INTG-04
**Success Criteria** (what must be TRUE):
    1. User runs `aliasman pack remove mypack` and pack + aliases are uninstalled
    2. Aliases marked `modified_by_user` are preserved when pack is removed
    3. Pack aliases merge with user aliases at render time into `~/.aliases`
    4. Claude Code hook scores pack aliases alongside user and imported aliases
    5. LanceDB semantic index automatically re-indexes after pack install or remove
    6. `aliasman list` output groups aliases by source (user, pack, imported, suggested)
**Status**: ✅ COMPLETE (2026-05-19)

### Phase 11: Built-in Packs
**Goal**: Ship curated k8s and docker packs so users have ready-to-use aliases from day one
**Depends on**: Phase 10
**Requirements**: CONT-01, CONT-02, CONT-03
**Success Criteria** (what must be TRUE):
    1. Curated k8s pack with 22 kubectl aliases for common operations
    2. Curated docker pack with 16 docker/compose aliases
    3. Built-in packs shipped as files in `builtin_packs/` directory
**Status**: ✅ COMPLETE (2026-05-19)

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6 -> 7 -> 8 -> 9 -> 10 -> 11

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Rust CLI Foundation | v0.0.1 | 1/1 | Complete | 2026-05-12 |
| 2. Shell Detection & Import | v0.0.1 | 1/1 | Complete | 2026-05-14 |
| 3. Alias CRUD & Listing | v0.0.1 | 1/1 | Complete | 2026-05-14 |
| 4. History Stats & Suggestions | v0.0.1 | 1/1 | Complete | 2026-05-14 |
| 5. Claude Hook Integration | v0.0.1 | 1/1 | Complete | 2026-05-14 |
| 6. Local Semantic Search | v0.0.1 | 1/1 | Complete | 2026-05-15 |
| 7. Pack Foundation | v0.1 | 1/1 | Complete | 2026-05-18 |
| 8. Pack Registry & Lifecycle | v0.1 | 1/1 | Complete | 2026-05-18 |
| 9. Pack Install with Safety | v0.1 | 2/2 | Complete | 2026-05-19 |
| 10. Pack Remove & Integration | v0.1 | 1/1 | Complete | 2026-05-19 |
| 11. Built-in Packs | v0.1 | 1/1 | Complete | 2026-05-19 |

---
*Roadmap last updated: 2026-05-19, v0.1 Alias Library shipped*
