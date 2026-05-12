# Phase 5: Claude Hook Integration - Context

**Gathered:** 2026-05-11
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase installs and runs the Claude Code integration for aliasman. It should preview and confirm hook installation, preserve existing Claude settings with backups and atomic writes, inject relevant aliases into Claude `SessionStart` context, respect a strict token budget, and stay silent by default on no-op or recoverable error paths. It does not add Codex integration, CLAUDE.md generation, PreToolUse command rewriting, PowerShell support, or package distribution.

</domain>

<decisions>
## Implementation Decisions

### Hook Install UX
- **D-01:** Installing the Claude hook should preview changes and require confirmation before writing.
- **D-02:** The preview must show the settings file, hook entry to add, that existing hooks/settings will be preserved, and the write target.
- **D-03:** Automatic install without confirmation is not the default behavior.

### Settings Merge Policy
- **D-04:** Parse existing `~/.claude/settings.json` as JSON and preserve unrelated keys and hooks.
- **D-05:** Add only the aliasman `SessionStart` hook entry.
- **D-06:** Create a timestamped backup before writing Claude settings.
- **D-07:** Write Claude settings atomically.
- **D-08:** Hook install must be idempotent and not duplicate the aliasman hook.
- **D-09:** Do not rewrite the whole hooks section in a way that clobbers user hooks.

### Alias Relevance Filtering
- **D-10:** Use both project files and alias metadata to choose injected aliases.
- **D-11:** Project signals include `.git`, `Cargo.toml`, `package.json`, and `Dockerfile`.
- **D-12:** Alias metadata signals include `tags`, `source`, and recency.
- **D-13:** Filtering should favor project-relevant aliases over dumping the full alias list.

### Token Budget Behavior
- **D-14:** Use a configurable token cap with a strict default around 500 tokens.
- **D-15:** Stop adding aliases when the budget is reached.
- **D-16:** Do not allow high-scoring aliases to exceed the hard cap by default.

### No-op And Error Behavior
- **D-17:** By default, the hook should exit 0 with no stdout when there are no aliases to inject, no relevant matches, or a recoverable error.
- **D-18:** Diagnostics may be available only in debug mode.
- **D-19:** Debug diagnostics should go to a log file or stderr where safe; they must not pollute Claude session context by default.

### the agent's Discretion
The planner may choose exact debug flag/config shape, backup timestamp formatting, scoring weights, and hook command nesting, provided the locked decisions above are implemented.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Planning Artifacts
- `.planning/PROJECT.md` — Project vision and core value.
- `.planning/REQUIREMENTS.md` — Phase 5 requirement IDs `CLD-01` through `CLD-05`.
- `.planning/ROADMAP.md` — Phase 5 goal and success criteria.
- `.planning/STATE.md` — Current milestone state.

### Prior Phases
- `.planning/phases/01-rust-cli-foundation-and-alias-store/01-CONTEXT.md` — Alias metadata and validation decisions.
- `.planning/phases/03-alias-crud-and-listing/03-CONTEXT.md` — CRUD and list behavior decisions.
- `.planning/phases/03-alias-crud-and-listing/03-01-PLAN.md` — CRUD implementation contract.

### Research
- `.planning/research/STACK.md` — Claude Code hook format and serde_json guidance.
- `.planning/research/ARCHITECTURE.md` — Hook registration, smart filtering, and shell/alias architecture.
- `.planning/research/PITFALLS.md` — Claude hook token overload and malformed hook output pitfalls.
- `.planning/phases/05-claude-hook-integration/05-RESEARCH.md` — Phase-specific Claude hook research.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Reuse canonical alias metadata from Phase 1 and alias tags/source from later CRUD/history flows.
- Reuse atomic write and backup patterns from prior phases.

### Established Patterns
- File-mutating install flows should preview then confirm.
- Tests should use tempdir fixtures rather than real user settings files.

### Integration Points
- `src/hook.rs` owns settings merge, hook install, hook runner, relevance scoring, token budgeting, and debug/no-op behavior.
- `src/main.rs` gains hook install and hook runner commands.
- Tests should use fixture Claude settings JSON and fixture project directories with marker files.

</code_context>

<specifics>
## Specific Ideas

- Default token budget: about 500 tokens.
- Silent no-op is the default hook behavior to avoid polluting Claude sessions.
- Hook install should visibly promise that existing Claude hooks are preserved.

</specifics>

<deferred>
## Deferred Ideas

- Codex integration.
- CLAUDE.md generation.
- PreToolUse command rewriting or blocking.
- Non-confirmed automatic hook installation by default.

</deferred>

---

*Phase: 5-Claude Hook Integration*
*Context gathered: 2026-05-11*
