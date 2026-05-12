# Phase 4: History Stats And Suggestions - Context

**Gathered:** 2026-05-11
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase adds shell history analytics and alias suggestions. It should detect or accept history file paths, show command frequency stats, generate suggested alias names, flag risky commands, and provide an explicit apply path that routes through existing CRUD validation. It does not add Claude hook behavior, PowerShell history parsing, cloud sync, or package distribution.

</domain>

<decisions>
## Implementation Decisions

### History Source Selection
- **D-01:** `aliasman stats` and `aliasman suggest` should auto-detect history files with override.
- **D-02:** Detection should use `$HISTFILE` first, then default shell history paths.
- **D-03:** Both commands must support `--history-file <path>` for tests and power users.

### Stats Output
- **D-04:** Default `aliasman stats` output should be a simple top commands table with columns `Count` and `Command`, sorted by frequency.
- **D-05:** Add a richer more-info mode, likely `--verbose`, that includes percentages and grouping by executable/tool such as `git`, `cargo`, or `npm`.
- **D-06:** Grouped analytics should not replace the default compact stats view.

### Suggestion Style
- **D-07:** `aliasman suggest` should generate alias names automatically.
- **D-08:** Suggestions should show suggested alias name, command, and reason, such as frequency or command length.
- **D-09:** The user should not be required to name suggestions manually before seeing useful output.

### Risky Command Handling
- **D-10:** Risky command suggestions can be shown if otherwise high-value, but must be clearly marked `Review carefully`.
- **D-11:** Risky suggestions must never be auto-applied.
- **D-12:** Risky patterns include command substitution, backticks, pipes, semicolons, `&&`, and `||`.

### Apply Flow
- **D-13:** `aliasman suggest` should be display-only by default.
- **D-14:** Provide an explicit apply path, such as `aliasman suggest --apply <alias>`, that creates a suggestion through existing CRUD validation.
- **D-15:** Applying a suggestion must respect duplicate-name, protected-name, and risky-command validation/confirmation rules.

### the agent's Discretion
The planner may choose exact `--verbose` and `--apply` flag shapes, alias-name generation heuristics, and output formatting, provided the decisions above are implemented.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Planning Artifacts
- `.planning/PROJECT.md` — Project vision and core value.
- `.planning/REQUIREMENTS.md` — Phase 4 requirement IDs `HST-01` through `HST-05`.
- `.planning/ROADMAP.md` — Phase 4 goal and success criteria.
- `.planning/STATE.md` — Current milestone state.

### Prior Phases
- `.planning/phases/01-rust-cli-foundation-and-alias-store/01-CONTEXT.md` — Store and validation decisions.
- `.planning/phases/03-alias-crud-and-listing/03-CONTEXT.md` — CRUD command, error, reload, and apply-through-CRUD decisions.
- `.planning/phases/03-alias-crud-and-listing/03-01-PLAN.md` — CRUD implementation contract.

### Research
- `.planning/research/PITFALLS.md` — zsh extended history, large history, and command-injection pitfalls.
- `.planning/research/FEATURES.md` — History suggestions and stats feature expectations.
- `.planning/phases/04-history-stats-and-suggestions/04-RESEARCH.md` — Phase-specific history research.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Reuse Phase 3 CRUD/store mutation path for explicit suggestion apply behavior.
- Reuse Phase 1 validation and protected-name policy.

### Established Patterns
- CLI commands use explicit names and named flags.
- Mutating flows must provide actionable errors and reload hints.

### Integration Points
- `src/history.rs` owns parsing, aggregation, risk detection, and suggestion generation.
- `src/main.rs` gains `stats` and `suggest`.
- Tests should use fixture/tempdir history files and not depend on the user's real shell history.

</code_context>

<specifics>
## Specific Ideas

- Default stats output: `Count`, `Command`.
- More-info stats mode: percentages and grouping by tool/executable.
- Suggestion output: alias name, command, reason, risk warning when applicable.

</specifics>

<deferred>
## Deferred Ideas

- PowerShell history parsing.
- Fully interactive suggestion acceptance flow.
- Advanced semantic alias naming.

</deferred>

---

*Phase: 4-History Stats And Suggestions*
*Context gathered: 2026-05-11*
