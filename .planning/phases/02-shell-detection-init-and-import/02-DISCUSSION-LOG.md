# Phase 2: Shell Detection, Init, And Import - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-11
**Phase:** 2-Shell Detection, Init, And Import
**Areas discussed:** Shell detection behavior, Import policy, Config write safety, Init UX, Reload hints

---

## Shell Detection Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Auto-detect with safe fallback | Use `$SHELL` first, then existing config files; if signals conflict or no config exists, ask the user to choose. | ✓ |
| Always ask first | Present detected shell/config as a recommendation but require confirmation every time. | |
| Fully automatic | Choose based on `$SHELL`/files and proceed without prompting unless there is an error. | |

**User's choice:** Auto-detect with safe fallback.
**Notes:** Ambiguity must trigger user choice rather than silent guessing.

---

## Import Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Import safe aliases, report skipped risky ones | Import straightforward aliases, skip protected names or unsupported syntax, and show a summary of what was skipped and why. | ✓ |
| Import everything exactly as found | Preserve the user's current setup, even if it includes protected names or complex syntax. | |
| Import but mark risky aliases disabled | Store risky aliases in metadata but do not write them to `~/.aliases` until user approves. | |

**User's choice:** Import safe aliases, report skipped risky ones.
**Notes:** Risky/protected/unsupported aliases should not silently enter the managed output.

---

## Config Write Safety

| Option | Description | Selected |
|--------|-------------|----------|
| Timestamped backup before every write | Create e.g. `.zshrc.aliasman-backup-2026-05-11T16-30-00`, keep the last 3 backups, then write atomically. | ✓ |
| Single rolling backup | Maintain `.zshrc.aliasman-backup`, overwriting it each time. | |
| Backup only on first init | Create a backup during first setup, skip later backups. | |

**User's choice:** Timestamped backup before every write.
**Notes:** Keep the last 3 backups per shell config file.

---

## Init UX

| Option | Description | Selected |
|--------|-------------|----------|
| Preview then confirm | Show detected shell, target config file, aliases to import, skipped aliases, backup path, and source block; require confirmation before writing. | ✓ |
| Automatic with summary after | Proceed when detection is unambiguous, then print what changed. | |
| Dry-run by default | Never write unless user passes `--apply`. | |

**User's choice:** Preview then confirm.
**Notes:** Default init should not mutate shell files until the user confirms.

---

## Reload Hints

| Option | Description | Selected |
|--------|-------------|----------|
| Shell-specific copy-paste hints | Print exact commands like `source ~/.zshrc` or `source ~/.bash_profile`, plus note that opening a new terminal also works. | ✓ |
| Only `source ~/.aliases` | Shorter and directly reloads the managed alias file. | |
| Minimal note | "Restart your shell to use changes." | |

**User's choice:** Shell-specific copy-paste hints.
**Notes:** Include the exact config file modified and mention opening a new terminal.

---

## the agent's Discretion

- Exact confirmation flag names.
- Internal preview struct names.
- Prompt wording.

## Deferred Ideas

- PowerShell support.
- Full alias CRUD.
- Optional non-interactive init flags.
