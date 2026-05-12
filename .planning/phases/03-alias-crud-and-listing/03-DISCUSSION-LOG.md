# Phase 3: Alias CRUD And Listing - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-11
**Phase:** 3-Alias CRUD And Listing
**Areas discussed:** Command names and flags, Duplicate/update behavior, List output, Error style, Reload hints after CRUD

---

## Command Names And Flags

| Option | Description | Selected |
|--------|-------------|----------|
| Explicit verbs + named flags | `aliasman add --name gs --command "git status"`, `update`, `delete`, `list`. Clear and matches the project brief. | ✓ |
| Short shell-like verbs | `aliasman rm gs`, `aliasman ls`, shorter but less self-documenting. | |
| Both explicit and short aliases | Support `delete` and `rm`, `list` and `ls`; convenient but expands surface area. | |

**User's choice:** Explicit verbs + named flags.
**Notes:** Do not add short aliases in Phase 3.

---

## Duplicate/Update Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Fail and point to update | `add` never overwrites; error says to use `aliasman update --name gs --command ...`. | ✓ |
| Prompt to overwrite | Interactive confirmation when duplicate exists. | |
| Overwrite with `--force` | Duplicate add fails unless `--force` is passed. | |

**User's choice:** Fail and point to update.
**Notes:** `update` is the explicit replacement path.

---

## List Output

| Option | Description | Selected |
|--------|-------------|----------|
| Compact table | Columns: `Name`, `Command`, `Source`. Keeps repeated use readable. | ✓ |
| Full metadata table | `Name`, `Command`, `Shell`, `Source`, `Tags`, `Description`, `Updated`. More complete but wider. | |
| Compact by default, full with `--verbose` | Default table plus optional full metadata. | |

**User's choice:** Compact table.
**Notes:** Full metadata output is not required by default.

---

## Error Style

| Option | Description | Selected |
|--------|-------------|----------|
| Actionable guidance | Errors include the problem and next command, e.g. duplicate `gs`; use `aliasman update --name gs --command "..."`. | ✓ |
| Terse CLI errors | Short messages only, e.g. `alias exists`. | |
| Verbose explanatory errors | Include extra context about where data is stored and what changed. | |

**User's choice:** Actionable guidance.
**Notes:** Include exact command hints where possible.

---

## Reload Hints After CRUD

| Option | Description | Selected |
|--------|-------------|----------|
| Always shell-specific | Every mutation prints exact reload guidance, e.g. `source ~/.zshrc` or open a new terminal. | ✓ |
| Only `source ~/.aliases` | Every mutation prints a shorter direct reload command. | |
| Only after init | CRUD commands do not print reload hints. | |

**User's choice:** Always shell-specific.
**Notes:** CRUD should reuse shell-specific reload guidance from Phase 2.

---

## the agent's Discretion

- Exact table formatting.
- Exact error wording.
- Optional `--verbose` list mode if cheap.

## Deferred Ideas

- Short command aliases.
- History suggestions.
- Claude hook integration.
