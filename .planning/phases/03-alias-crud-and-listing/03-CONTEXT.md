# Phase 3: Alias CRUD And Listing - Context

**Gathered:** 2026-05-11
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase delivers normal user-facing alias CRUD: add, update, delete, and list. It builds on the existing alias store, validation, managed alias rendering, shell init, and reload-hint helpers. It does not add history suggestions, Claude hook integration, PowerShell support, sharing/sync, or package distribution.

</domain>

<decisions>
## Implementation Decisions

### Command Names And Flags
- **D-01:** Use explicit verbs and named flags: `aliasman add --name gs --command "git status"`, `aliasman update --name gs --command "..."`, `aliasman delete --name gs`, and `aliasman list`.
- **D-02:** Do not add short shell-like aliases such as `rm`, `ls`, or command synonyms in Phase 3.
- **D-03:** Named flags are preferred over positional ambiguity for user clarity.

### Duplicate And Update Behavior
- **D-04:** `aliasman add` must fail when the alias name already exists.
- **D-05:** Duplicate-add errors should point users to `aliasman update --name <name> --command "..."`.
- **D-06:** `add` should not prompt to overwrite and should not overwrite via `--force`; `update` is the explicit replacement path.

### List Output
- **D-07:** `aliasman list` default output should be a compact table with columns `Name`, `Command`, and `Source`.
- **D-08:** Full metadata output is not required in Phase 3.
- **D-09:** If easy, verbose output can be added later, but the Phase 3 required behavior is compact default output.

### Error Style
- **D-10:** CRUD errors should be actionable: include the problem and the next command or flag that fixes it.
- **D-11:** Duplicate errors should include the existing alias name and an `aliasman update --name ... --command ...` hint.
- **D-12:** Protected-name errors should include the `--force` remedy.
- **D-13:** Missing alias errors should name the missing alias and suggest `aliasman add` when appropriate.

### Reload Hints After CRUD
- **D-14:** Every alias mutation (`add`, `update`, `delete`) should print shell-specific reload guidance.
- **D-15:** Reload guidance should include the exact shell config file to source, such as `source ~/.zshrc` or `source ~/.bash_profile`, and mention opening a new terminal.
- **D-16:** Do not limit reload hints to init only.

### the agent's Discretion
The planner may decide table formatting implementation, exact wording, and whether to include an optional `--verbose` list mode, provided compact default output and actionable errors are implemented.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Planning Artifacts
- `.planning/PROJECT.md` — Project vision and core value.
- `.planning/REQUIREMENTS.md` — Phase 3 requirement IDs `ALS-01` through `ALS-05`.
- `.planning/ROADMAP.md` — Phase 3 goal and success criteria.
- `.planning/STATE.md` — Current milestone state.

### Prior Phases
- `.planning/phases/01-rust-cli-foundation-and-alias-store/01-CONTEXT.md` — Alias store, validation, and foundation decisions.
- `.planning/phases/02-shell-detection-init-and-import/02-CONTEXT.md` — Shell detection, reload hint, and init decisions.
- `.planning/phases/02-shell-detection-init-and-import/02-01-PLAN.md` — Shell init implementation contract.

### Research
- `.planning/research/FEATURES.md` — Alias CRUD feature expectations.
- `.planning/research/PITFALLS.md` — Protected-name and reload UX pitfalls.
- `.planning/phases/03-alias-crud-and-listing/03-RESEARCH.md` — Phase-specific CRUD research.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Reuse Phase 1 validation and store helpers.
- Reuse Phase 2 reload-hint and shell config knowledge so CRUD mutation output stays shell-specific.

### Established Patterns
- Named flags are the project standard for CLI commands.
- Protected-name force handling is a reusable validation policy, not per-command ad hoc logic.

### Integration Points
- `src/main.rs` gains `add`, `update`, `delete`, and `list`.
- `src/store.rs` owns canonical mutations and managed alias regeneration.
- Tests should use tempdir-backed stores and avoid real home files.

</code_context>

<specifics>
## Specific Ideas

- Default list output: `Name`, `Command`, `Source`.
- Duplicate-add guidance should explicitly direct users to `update`.

</specifics>

<deferred>
## Deferred Ideas

- Short command aliases like `rm`/`ls`.
- Full metadata list output by default.
- History-derived alias suggestions.
- Claude hook integration.

</deferred>

---

*Phase: 3-Alias CRUD And Listing*
*Context gathered: 2026-05-11*
