# Phase 2: Shell Detection, Init, And Import - Context

**Gathered:** 2026-05-11
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase adds safe `aliasman init` behavior for zsh and bash. It should detect the user's shell/config file, preview planned changes, import straightforward existing aliases, skip and report risky aliases, add the managed source block idempotently, create backups before modifying shell config files, and print shell-specific reload instructions. It does not implement normal alias CRUD, history suggestions, Claude hooks, PowerShell support, or Homebrew distribution.

</domain>

<decisions>
## Implementation Decisions

### Shell Detection Behavior
- **D-01:** `aliasman init` should auto-detect with safe fallback: use `$SHELL` first, then existing config files.
- **D-02:** If detection signals conflict or no config file exists, `aliasman init` must ask the user to choose instead of silently guessing.
- **D-03:** Fully automatic no-prompt behavior is not allowed when shell/config detection is ambiguous.

### Import Policy
- **D-04:** Import straightforward safe aliases from `.zshrc`, `.bashrc`, and `.bash_profile`.
- **D-05:** Skip protected names and unsupported/complex shell syntax during import.
- **D-06:** Show a summary of skipped aliases with the reason each was skipped.
- **D-07:** Do not import everything exactly as found if doing so would pull risky/protected names or unsupported syntax into the managed file.

### Config Write Safety
- **D-08:** Create a timestamped backup before every write to `.zshrc`, `.bashrc`, or `.bash_profile`.
- **D-09:** Backup names should follow the pattern `.zshrc.aliasman-backup-YYYY-MM-DDTHH-MM-SS` or equivalent per target config file.
- **D-10:** Keep the last 3 backups per shell config file and prune older aliasman backups.
- **D-11:** Shell config writes must still be atomic after the backup is created.

### Init UX
- **D-12:** `aliasman init` should preview changes and require confirmation before writing.
- **D-13:** The preview must show the detected shell, target config file, aliases to import, skipped aliases, backup path, and managed source block.
- **D-14:** The command may support explicit non-interactive/apply behavior later, but the default user-facing init path is preview-then-confirm.

### Reload Hints
- **D-15:** After init or alias changes, print shell-specific copy-paste reload instructions such as `source ~/.zshrc` or `source ~/.bash_profile`.
- **D-16:** Also tell the user that opening a new terminal works.
- **D-17:** Do not rely only on `source ~/.aliases`; after init, the shell config source block also needs to be available.

### the agent's Discretion
The planner may choose exact prompt wording, confirmation flag names, shell config detection helper names, and internal preview structs, provided the locked decisions above are implemented.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Planning Artifacts
- `.planning/PROJECT.md` — Project vision and v0.0.1 scope.
- `.planning/REQUIREMENTS.md` — Phase 2 requirement IDs `SHL-01` through `SHL-05`.
- `.planning/ROADMAP.md` — Phase 2 goal and success criteria.
- `.planning/STATE.md` — Current milestone state.

### Prior Phase
- `.planning/phases/01-rust-cli-foundation-and-alias-store/01-CONTEXT.md` — Store, validation, fixed `~/.aliases`, and testing decisions from Phase 1.
- `.planning/phases/01-rust-cli-foundation-and-alias-store/01-RESEARCH.md` — Foundation research.
- `.planning/phases/01-rust-cli-foundation-and-alias-store/01-01-PLAN.md` — Foundation implementation contract.

### Research
- `.planning/research/ARCHITECTURE.md` — Shell detection, managed block, backup, and atomic write guidance.
- `.planning/research/PITFALLS.md` — Data-loss, duplicate import, shell reload UX, and cross-shell syntax pitfalls.
- `.planning/research/SUMMARY.md` — Milestone-level stack and architecture synthesis.
- `.planning/phases/02-shell-detection-init-and-import/02-RESEARCH.md` — Phase-specific shell init/import research.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Phase 2 should build on Phase 1 modules: CLI parsing, alias model, store serialization, managed alias rendering, atomic writes, and validation/protected-name helpers.

### Established Patterns
- Phase 1 established the policy that real home files must not be touched in tests. Phase 2 should preserve that by using tempdir home fixtures and injectable paths.

### Integration Points
- `src/main.rs` gains the `init` command.
- `src/shell.rs` should own shell detection and config selection.
- `src/import.rs` should own shell alias parsing, safe import decisions, managed source block helpers, and skipped-alias reporting.
- `src/store.rs` should provide backup/atomic write helpers where appropriate.

</code_context>

<specifics>
## Specific Ideas

- Default init UX should be preview-then-confirm, not silent file mutation.
- Skipped imports should be visible to the user and actionable.
- Reload hints should be specific to the shell config file actually modified.

</specifics>

<deferred>
## Deferred Ideas

- PowerShell support remains deferred.
- CRUD commands beyond init belong to Phase 3.
- Non-interactive init flags may be added if useful, but should not replace the default preview-then-confirm UX.

</deferred>

---

*Phase: 2-Shell Detection, Init, And Import*
*Context gathered: 2026-05-11*
