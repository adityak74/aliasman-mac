# Phase 5: Claude Hook Integration - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-11
**Phase:** 5-Claude Hook Integration
**Areas discussed:** Hook install UX, Settings merge policy, Alias relevance filtering, Token budget behavior, No-op/error behavior

---

## Hook Install UX

| Option | Description | Selected |
|--------|-------------|----------|
| Preview then confirm | Show the settings file, hook entry to add, existing hooks preserved, and require confirmation before writing. | ✓ |
| Apply automatically with summary after | Install immediately and print what changed. | |
| Dry-run by default | Show changes only; require `--apply` to write. | |

**User's choice:** Preview then confirm.
**Notes:** Default install should not mutate settings without confirmation.

---

## Settings Merge Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Strict preserve and backup | Parse existing JSON, preserve unrelated keys/hooks exactly, add only the aliasman `SessionStart` entry, create a timestamped backup, write atomically, and be idempotent. | ✓ |
| Preserve but no backup | Merge carefully and write atomically, but no backup. | |
| Rewrite hooks section only | Simpler, but risks clobbering user hooks. | |

**User's choice:** Strict preserve and backup.
**Notes:** Existing user hooks must not be clobbered.

---

## Alias Relevance Filtering

| Option | Description | Selected |
|--------|-------------|----------|
| Project files + alias metadata | Use `.git`, `Cargo.toml`, `package.json`, `Dockerfile`, alias `tags`, `source`, and recency. | ✓ |
| Alias metadata only | Use tags/source/recency, ignore project files. | |
| Project files only | Infer from cwd files and alias command text, ignore metadata. | |

**User's choice:** Project files + alias metadata.
**Notes:** Filtering should avoid dumping the whole alias list.

---

## Token Budget Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Configurable cap with strict default | Default around 500 tokens, configurable later, hard-stop output when budget is reached. | ✓ |
| Soft cap | Try to stay under budget, but allow high-scoring aliases to exceed it. | |
| No cap initially | Simpler, but risks context bloat. | |

**User's choice:** Configurable cap with strict default.
**Notes:** Hard cap by default, around 500 tokens.

---

## No-op/Error Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Silent no-op for Claude, diagnostics in debug mode | Exit 0 with no stdout by default; if debug is enabled, log diagnostics to a file or stderr where safe. | ✓ |
| Always emit empty JSON | Valid but may add noise. | |
| Print warning context into Claude | Visible, but pollutes sessions. | |

**User's choice:** Silent no-op for Claude, diagnostics in debug mode.
**Notes:** No-op/error paths should not pollute Claude context.

---

## the agent's Discretion

- Exact debug setting shape.
- Scoring weights.
- Backup timestamp formatting.

## Deferred Ideas

- Codex integration.
- CLAUDE.md generation.
- PreToolUse command rewriting.
