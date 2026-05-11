# Phase 2 Research: Shell Detection, Init, And Import

**Phase:** 02 - Shell Detection, Init, And Import
**Researched:** 2026-05-11
**Status:** Ready for planning

## Phase Goal

Initialize aliasman safely for zsh and bash. This phase adds shell detection, first-run import from shell config files, idempotent managed source block insertion, backups before shell config edits, and reload hints after shell/config mutations.

## Requirements Covered

- **SHL-01:** User can initialize aliasman for zsh or bash with automatic shell detection.
- **SHL-02:** aliasman can import existing aliases from zsh/bash config files without duplicating aliases on repeated runs.
- **SHL-03:** aliasman adds an idempotent managed source block to the detected shell config.
- **SHL-04:** aliasman creates backups before modifying user shell config files.
- **SHL-05:** aliasman prints actionable shell reload hints after any command that changes aliases or shell config.

## Implementation Guidance

- Add a real `init` subcommand to the existing `clap` CLI.
- Keep all filesystem paths injectable for tests; do not write real `~/.zshrc`, `~/.bashrc`, or `~/.bash_profile` in tests.
- Detect shells in this order: `$SHELL`, then existing config files, then explicit unknown/error.
- On macOS bash, prefer `~/.bash_profile` when present; otherwise `~/.bashrc`.
- Use managed block markers:
  ```sh
  # >>> aliasman >>>
  [ -f "$HOME/.aliases" ] && source "$HOME/.aliases"
  # <<< aliasman <<<
  ```
- Insert the managed block only if it is absent.
- Create timestamped backups before modifying shell config files.
- Import simple alias definitions from shell config files into canonical store metadata with `source = imported`.
- Deduplicate imported aliases by alias name before writing canonical metadata.
- Regenerate managed `~/.aliases` from canonical metadata after import.

## Tests To Require

- Shell detection from `$SHELL`-like path strings.
- zsh/bash config selection behavior using tempdir home fixtures.
- Managed block insertion is idempotent.
- Existing managed block is not duplicated.
- Backups are created before shell config writes.
- Importing the same aliases twice produces one canonical record per name.
- `init` output contains `source ~/.aliases` or an equivalent reload hint.

