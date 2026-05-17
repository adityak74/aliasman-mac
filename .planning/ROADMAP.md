# Roadmap: aliasman

## Milestones

- ✅ **v0.0.1 CLI Alias Manager MVP** — Phases 1-6 (shipped 2026-05-16)
- 🚧 **v0.1 Alias Library** — Phases 7-11 (in progress)

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

### 🚧 v0.1 Alias Library (In Progress)

**Milestone Goal:** Developers can create, share, and install reusable alias packs — curated collections of aliases for common toolchains (k8s, docker, terraform) — without manually copying aliases between machines.

- [ ] **Phase 7: Pack Foundation** — Manifest schema, AliasSource::Pack, pack create/add/export
- [ ] **Phase 8: Pack Registry & Lifecycle** — registry.toml, pack list, pack tracking
- [ ] **Phase 9: Pack Install with Safety** — File/URL install, safety scanner, collision detection, two-phase apply
- [ ] **Phase 10: Pack Remove & Integration** — Uninstall, hook scoring, semantic re-index, CLI wiring
- [ ] **Phase 11: Built-in Packs** — k8s + docker curated content

## Phase Details

<details>
<summary>✅ v0.0.1 CLI Alias Manager MVP (Phases 1-6) — SHIPPED 2026-05-16</summary>

### Phase 1: Rust CLI Foundation And Alias Store
**Goal**: Establish Rust project scaffold with clap CLI, AliasRecord model, TOML round-trip, and alias store
**Depends on**: Nothing (first phase)
**Plans**: 1 plan

Plans:
- [x] 01-01: Rust CLI foundation and alias store

### Phase 2: Shell Detection, Init, And Import
**Goal**: Users can initialize aliasman with automatic shell detection and alias import
**Depends on**: Phase 1
**Plans**: 1 plan

Plans:
- [x] 02-01: Shell detection, init, and alias import

### Phase 3: Alias CRUD And Listing
**Goal**: Users can fully manage aliases via add, update, delete, and list commands
**Depends on**: Phase 2
**Plans**: 1 plan

Plans:
- [x] 03-01: Alias CRUD operations and listing

### Phase 4: History Stats And Suggestions
**Goal**: Users can analyze shell history and receive data-driven alias suggestions
**Depends on**: Phase 3
**Plans**: 1 plan

Plans:
- [x] 04-01: History statistics and alias suggestions

### Phase 5: Claude Hook Integration
**Goal**: Claude Code sessions receive contextually relevant aliases via the hook
**Depends on**: Phase 4
**Plans**: 1 plan

Plans:
- [x] 05-01: Claude Code hook with smart alias filtering

### Phase 6: Local Semantic Alias Search With LanceDB And MCP
**Goal**: Users and Claude can semantically search aliases using natural language
**Depends on**: Phase 5
**Plans**: 1 plan

Plans:
- [x] 06-01: LanceDB vector index and MCP alias_search tool

</details>

### Phase 7: Pack Foundation
**Goal**: Users can create, populate, and export alias packs as shareable TOML files
**Depends on**: Phase 6
**Requirements**: PACK-01, PACK-02, PACK-03, PACK-04, MGMT-04
**Success Criteria** (what must be TRUE):
   1. User runs `aliasman pack create mypack` and gets a structured directory with a valid `pack.toml` manifest containing name, version, description, author, and format_version fields
   2. User can add aliases to the pack with `aliasman pack add mypack kget "kubectl get pods"` and the alias is persisted inside the pack directory
   3. User runs `aliasman pack export mypack` and receives a single shareable `.toml` file
   4. Every alias in the pack is tracked with `AliasSource::Pack("mypack")` for provenance
**Plans**: TBD

### Phase 8: Pack Registry & Lifecycle
**Goal**: Installed packs are tracked in a local registry, and the user can list them with metadata
**Depends on**: Phase 7
**Requirements**: MGMT-01, MGMT-05
**Success Criteria** (what must be TRUE):
   1. User runs `aliasman pack list` and sees a table of installed packs showing name, version, source, and alias count
   2. Installed packs are recorded in `~/.config/aliasman/registry.toml` with install metadata
   3. Registry state survives tool restarts (persists across invocations)
**Plans**: TBD

### Phase 9: Pack Install with Safety
**Goal**: Users can safely install packs from local files or URLs with full safety guarantees
**Depends on**: Phase 8
**Requirements**: INST-01, INST-02, INST-03, INST-04, INST-05, INST-06, INST-07, INST-08
**Success Criteria** (what must be TRUE):
   1. User runs `aliasman pack install mypack.toml` and sees a dry-run preview listing all aliases before any are applied
   2. User installs a pack from a URL with `aliasman pack install --url https://example.com/pack.toml` and it downloads and installs correctly
   3. Dangerous patterns (command substitution, pipe-to-shell, destructive commands) are flagged by the safety scanner during install
   4. User can override safety warnings with `--force` flag
   5. Pre-install collision scan detects pack aliases that conflict with existing user aliases, and user aliases always win (pack alias skipped unless `--force`)
   6. Install uses two-phase apply — all aliases are validated first, and only if validation passes are all written atomically
**Plans**: TBD

### Phase 10: Pack Remove & Integration
**Goal**: Users can uninstall packs cleanly, and pack aliases integrate with hook scoring, semantic search, and the list command
**Depends on**: Phase 9
**Requirements**: MGMT-02, MGMT-03, INTG-01, INTG-02, INTG-03, INTG-04
**Success Criteria** (what must be TRUE):
   1. User runs `aliasman pack remove mypack` and the pack and its aliases are uninstalled
   2. Aliases marked as `modified_by_user` are preserved when their pack is removed
   3. Pack aliases merge with user aliases at render time into `~/.aliases` (not at storage time)
   4. Claude Code hook scores pack aliases alongside user and imported aliases
   5. LanceDB semantic index automatically re-indexes after pack install or remove
   6. `aliasman list` output groups aliases by source (user, pack, imported, suggested)
**Plans**: TBD

### Phase 11: Built-in Packs
**Goal**: Ship curated k8s and docker packs so users have ready-to-use aliases from day one
**Depends on**: Phase 10
**Requirements**: CONT-01, CONT-02, CONT-03
**Success Criteria** (what must be TRUE):
   1. A curated k8s pack with 15-20 kubectl aliases for common operations is available
   2. A curated docker pack with 10-15 docker/compose aliases is available
   3. Built-in packs are shipped as files in `~/.config/aliasman/builtin_packs/` (not embedded in the binary)
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 7 -> 8 -> 9 -> 10 -> 11

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Rust CLI Foundation | v0.0.1 | 1/1 | Complete | 2026-05-12 |
| 2. Shell Detection & Import | v0.0.1 | 1/1 | Complete | 2026-05-14 |
| 3. Alias CRUD & Listing | v0.0.1 | 1/1 | Complete | 2026-05-14 |
| 4. History Stats & Suggestions | v0.0.1 | 1/1 | Complete | 2026-05-14 |
| 5. Claude Hook Integration | v0.0.1 | 1/1 | Complete | 2026-05-14 |
| 6. Local Semantic Search | v0.0.1 | 1/1 | Complete | 2026-05-15 |
| 7. Pack Foundation | v0.1 | 0/0 | Not started | - |
| 8. Pack Registry & Lifecycle | v0.1 | 0/0 | Not started | - |
| 9. Pack Install with Safety | v0.1 | 0/0 | Not started | - |
| 10. Pack Remove & Integration | v0.1 | 0/0 | Not started | - |
| 11. Built-in Packs | v0.1 | 0/0 | Not started | - |

---
*Roadmap last updated: 2026-05-17, v0.1 Alias Library roadmap created*
