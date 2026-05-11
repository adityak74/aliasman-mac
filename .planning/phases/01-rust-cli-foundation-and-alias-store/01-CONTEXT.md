# Phase 1: Rust CLI Foundation And Alias Store - Context

**Gathered:** 2026-05-11
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase creates the initial Rust CLI scaffold and the safe canonical alias storage foundation for aliasman. It does not implement shell init, user-facing CRUD flows, history analysis, or Claude hook behavior; those are later phases. The work here should prove that the CLI exists, the alias metadata schema is stable enough for v0.0.1, managed alias output can be regenerated atomically, and validation rules are available for later commands.

</domain>

<decisions>
## Implementation Decisions

### Store Shape
- **D-01:** Use the full v0.0.1 alias record from the start: `name`, `command`, optional `description`, `tags`, `shell`, `source`, `created_at`, and `updated_at`.
- **D-02:** Avoid a minimal `name`/`command`-only schema because it would force early migration churn when history suggestions and Claude filtering arrive.
- **D-03:** Do not add agent-specific fields such as `projects`, `tools`, or `relevance_hints` in Phase 1. Claude filtering should use tags/source and project signals later unless a later phase proves more metadata is needed.

### Generated Alias File
- **D-04:** Phase 1 should generate a fixed `~/.aliases` file path, matching the project brief and keeping the foundation predictable.
- **D-05:** Do not add configurable generated alias output paths in Phase 1. Configuration can be added later if real use cases require it.

### CLI Command Surface
- **D-06:** Phase 1 should expose only the foundation CLI surface: `aliasman --help`, `aliasman --version`, and the internal library/store behavior needed by later phases.
- **D-07:** Do not add the full future command tree as not-implemented stubs in Phase 1.
- **D-08:** Do not add temporary developer/debug commands such as `store validate` or `store render` unless the planner identifies a strong testing need. Prefer tests over temporary CLI surface.

### Validation Policy
- **D-09:** Enforce strict alias-name validation in Phase 1 using the pattern `[A-Za-z_][A-Za-z0-9_-]*`.
- **D-10:** Include protected-name handling in Phase 1. Protected names such as `rm`, `sudo`, `git`, and `curl` must require an explicit force path before later CRUD code can write them.
- **D-11:** Validation should be implemented as reusable library behavior so later CRUD and history-suggestion phases use the same rules.

### Testing Baseline
- **D-12:** Phase 1 completion requires unit tests for validation and serialization.
- **D-13:** Phase 1 completion requires tempdir-based integration tests proving managed alias output writes atomically without touching real home files.
- **D-14:** CLI process smoke tests are not required for Phase 1 unless they are cheap to include; the required baseline is unit tests plus tempdir integration tests.

### the agent's Discretion
The planner may decide exact Rust module names, crate layout, timestamp representation, TOML crate choice, and atomic-write helper signatures, provided the locked decisions above and Phase 1 requirements are satisfied.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Planning Artifacts
- `.planning/PROJECT.md` — Project vision, core value, v0.0.1 milestone scope, constraints, and key decisions.
- `.planning/REQUIREMENTS.md` — Phase 1 requirement IDs `FND-01` through `FND-04` and future scope boundaries.
- `.planning/ROADMAP.md` — Phase 1 goal, success criteria, and dependencies.
- `.planning/STATE.md` — Current milestone and planning state.

### Research
- `.planning/research/STACK.md` — Recommended Rust CLI stack and crate choices.
- `.planning/research/ARCHITECTURE.md` — Alias store, config, file-management, and component boundary guidance.
- `.planning/research/PITFALLS.md` — Data-loss, duplicate import, protected-name, reload UX, and syntax compatibility pitfalls.
- `.planning/research/FEATURES.md` — v0.0.1 feature categories and table-stakes behavior.
- `.planning/research/SUMMARY.md` — Synthesized stack, architecture, and roadmap guidance.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- No source scaffold exists yet. There are no reusable source files, modules, tests, or build scripts to preserve.

### Established Patterns
- No application code patterns exist yet. The planner should establish a conservative Rust crate layout suitable for a CLI plus reusable library modules.

### Integration Points
- New code connects to the repository root by creating the Rust project scaffold and tests.
- `IDEA.md` exists as untracked raw project input, but downstream planning should rely on `.planning/PROJECT.md`, `.planning/REQUIREMENTS.md`, `.planning/ROADMAP.md`, and this `CONTEXT.md` as canonical planning sources.

</code_context>

<specifics>
## Specific Ideas

- The initial user-facing CLI surface should be intentionally small in Phase 1.
- Store/schema decisions should anticipate history suggestions and Claude filtering without over-designing agent-specific metadata.
- Safety proof matters early: no tests should touch real home-directory shell files.

</specifics>

<deferred>
## Deferred Ideas

- Configurable generated alias output paths are deferred beyond Phase 1.
- Full command tree stubs are deferred until the phases that implement those capabilities.
- CLI smoke tests can be added if cheap, but are not part of the required Phase 1 baseline.
- PowerShell support remains outside v0.0.1.

</deferred>

---

*Phase: 1-Rust CLI Foundation And Alias Store*
*Context gathered: 2026-05-11*
