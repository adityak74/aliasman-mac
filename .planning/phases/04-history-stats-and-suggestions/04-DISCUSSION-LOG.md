# Phase 4: History Stats And Suggestions - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-11
**Phase:** 4-History Stats And Suggestions
**Areas discussed:** History source selection, Stats output, Suggestion style, Risky command handling, Apply flow

---

## History Source Selection

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-detect with override | Use `$HISTFILE`, then default shell history paths; also support `--history-file <path>` for tests and power users. | ✓ |
| Require `--history-file` | Simpler and safer, but less convenient. | |
| Auto-detect only | Convenient, but harder to test and less transparent. | |

**User's choice:** Auto-detect with override.
**Notes:** Support `--history-file <path>`.

---

## Stats Output

| Option | Description | Selected |
|--------|-------------|----------|
| Top commands table | Columns: `Count`, `Command`, sorted by frequency. Simple and useful. | ✓ |
| Counts plus percentages | `Count`, `%`, `Command`; more informative but slightly noisier. | partial |
| Grouped analytics | Group by executable/tool (`git`, `cargo`, `npm`) plus top commands; richer but more scope. | partial |

**User's choice:** Top commands table by default, with a more-info mode for percentages and grouped analytics.
**Notes:** Capture as default `stats` plus `--verbose` or similar.

---

## Suggestion Style

| Option | Description | Selected |
|--------|-------------|----------|
| Generate alias names automatically | Show suggested alias name + command + reason, e.g. `gst -> git status`. | ✓ |
| Show command candidates only | User chooses names manually later. | |
| Ask interactively for names | Suggest commands, then prompt user to name/apply them. | |

**User's choice:** Generate alias names automatically.
**Notes:** Include reason in output.

---

## Risky Command Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Show with warnings | Include them only if otherwise high-value, clearly marked `Review carefully`; never auto-apply. | ✓ |
| Exclude from suggestions | Safest and simplest, but may hide useful real workflows. | |
| Separate risky section | Normal suggestions first, risky candidates under a separate heading. | |

**User's choice:** Show with warnings.
**Notes:** Risky suggestions must never be auto-applied.

---

## Apply Flow

| Option | Description | Selected |
|--------|-------------|----------|
| Display-only by default, explicit apply flag | `aliasman suggest` only shows suggestions; `aliasman suggest --apply <alias>` or similar explicitly creates one through CRUD validation. | ✓ |
| Display-only only | No apply path in v0.0.1; user manually runs `aliasman add`. | |
| Interactive apply | `suggest` prompts the user to accept suggestions in the same flow. | |

**User's choice:** Display-only by default, explicit apply flag.
**Notes:** Apply path must go through CRUD validation.

---

## the agent's Discretion

- Exact verbose flag name.
- Exact apply flag shape.
- Alias-name generation heuristics.

## Deferred Ideas

- PowerShell history parsing.
- Fully interactive suggestion flow.
