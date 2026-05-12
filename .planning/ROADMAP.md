# Roadmap: aliasman v0.0.1

**Milestone:** v0.0.1 CLI Alias Manager MVP
**Created:** 2026-05-11
**Requirements:** 29 total, 29 mapped

## Phase 1: Rust CLI Foundation And Alias Store

**Goal:** Create the Rust CLI scaffold and safe canonical alias storage foundation.

**Requirements:** FND-01, FND-02, FND-03, FND-04

**Success criteria:**
1. `aliasman --help` and command help render successfully.
2. Alias metadata can be loaded from and saved to the canonical aliasman data file.
3. Managed alias output can be regenerated from canonical metadata using atomic writes.
4. Invalid alias names are rejected and protected names require an explicit force path.

**Dependencies:** None.

## Phase 2: Shell Detection, Init, And Import

**Goal:** Initialize aliasman safely for zsh/bash and import existing aliases without corrupting shell configuration.

**Requirements:** SHL-01, SHL-02, SHL-03, SHL-04, SHL-05

**Success criteria:**
1. `aliasman init` detects zsh or bash and chooses the correct config file.
2. Existing shell aliases are imported into canonical metadata without duplication.
3. Re-running init does not duplicate imported aliases or managed source blocks.
4. Shell config edits create backups before writing.
5. Init and mutation flows print clear reload instructions.

**Dependencies:** Phase 1.

## Phase 3: Alias CRUD And Listing

**Goal:** Deliver the core user workflow for creating, changing, deleting, and viewing aliases from the CLI.

**Requirements:** ALS-01, ALS-02, ALS-03, ALS-04, ALS-05

**Success criteria:**
1. User can add an alias with named flags and see it in the generated aliases file.
2. User can update an alias and the generated aliases file reflects the new command.
3. User can delete an alias and the generated aliases file removes it.
4. User can list aliases in a readable table.
5. Duplicate aliases, missing aliases, invalid flags, and write failures produce actionable errors.

**Dependencies:** Phase 1, Phase 2.

## Phase 4: History Stats And Suggestions

**Goal:** Use zsh/bash shell history to show command analytics and suggest useful aliases safely.

**Requirements:** HST-01, HST-02, HST-03, HST-04, HST-05

**Success criteria:**
1. User can view top command frequency statistics from shell history.
2. zsh extended history lines are parsed as commands, not timestamp noise.
3. User can view suggested aliases for frequent long commands.
4. History-derived suggestions are never written without explicit approval.
5. Risky suggested commands are flagged before the user accepts them.

**Dependencies:** Phase 1, Phase 3.

## Phase 5: Claude Hook Integration

**Goal:** Install a Claude Code hook that injects relevant alias context without overloading session tokens.

**Requirements:** CLD-01, CLD-02, CLD-03, CLD-04, CLD-05

**Success criteria:**
1. User can install a Claude `SessionStart` hook without overwriting existing settings.
2. The hook command emits valid Claude hook JSON when relevant aliases exist.
3. Hook output changes based on current project context signals.
4. Hook output respects the configured token budget.
5. Hook exits cleanly with no noisy output when no alias context should be injected.

**Dependencies:** Phase 1, Phase 3.

## Phase 6: Local Semantic Alias Search With LanceDB And MCP

**Goal:** Add local semantic alias search using local embeddings, a LanceDB vector index, CLI natural-language search, and an MCP tool/server that lets Claude search aliases on demand.

**Requirements:** SEM-01, SEM-02, SEM-03, SEM-04, SEM-05

**Success criteria:**
1. Aliases are embedded locally and stored in a LanceDB-backed vector index.
2. Alias index refresh handles add, update, delete, import, and suggestion-apply flows.
3. User can run a CLI semantic search query and receive relevant aliases with scores or ranked ordering.
4. Claude can call a local MCP tool/server to search aliases semantically instead of depending only on hook-injected aliases.
5. No alias command or metadata leaves the machine for embeddings by default.

**Dependencies:** Phase 5.

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| FND-01 | Phase 1 | Pending |
| FND-02 | Phase 1 | Pending |
| FND-03 | Phase 1 | Pending |
| FND-04 | Phase 1 | Pending |
| SHL-01 | Phase 2 | Pending |
| SHL-02 | Phase 2 | Pending |
| SHL-03 | Phase 2 | Pending |
| SHL-04 | Phase 2 | Pending |
| SHL-05 | Phase 2 | Pending |
| ALS-01 | Phase 3 | Pending |
| ALS-02 | Phase 3 | Pending |
| ALS-03 | Phase 3 | Pending |
| ALS-04 | Phase 3 | Pending |
| ALS-05 | Phase 3 | Pending |
| HST-01 | Phase 4 | Pending |
| HST-02 | Phase 4 | Pending |
| HST-03 | Phase 4 | Pending |
| HST-04 | Phase 4 | Pending |
| HST-05 | Phase 4 | Pending |
| CLD-01 | Phase 5 | Pending |
| CLD-02 | Phase 5 | Pending |
| CLD-03 | Phase 5 | Pending |
| CLD-04 | Phase 5 | Pending |
| CLD-05 | Phase 5 | Pending |
| SEM-01 | Phase 6 | Pending |
| SEM-02 | Phase 6 | Pending |
| SEM-03 | Phase 6 | Pending |
| SEM-04 | Phase 6 | Pending |
| SEM-05 | Phase 6 | Pending |

**Coverage:**
- v0.0.1 requirements: 29 total
- Mapped to phases: 29
- Unmapped: 0

---
*Roadmap created: 2026-05-11*
*Last updated: 2026-05-11 after adding Phase 6 semantic search*
