# Phase 3 Research: Alias CRUD And Listing

**Phase:** 03 - Alias CRUD And Listing
**Researched:** 2026-05-11
**Status:** Ready for planning

## Phase Goal

Deliver the core user workflow for creating, updating, deleting, and listing aliases through named CLI flags. This phase builds on the store, validation, rendering, and shell init foundation from Phases 1-2.

## Requirements Covered

- **ALS-01:** User can add an alias with named flags.
- **ALS-02:** User can update an existing alias with named flags.
- **ALS-03:** User can delete an alias by name.
- **ALS-04:** User can list aliases in a readable CLI table.
- **ALS-05:** User can see useful errors for duplicate aliases, missing aliases, invalid flags, and write failures.

## Implementation Guidance

- Add real subcommands: `add`, `update`, `delete`, `list`.
- Use named flags only: `--name`, `--command`, optional `--description`, optional repeated `--tag`, and `--force` for protected names.
- All mutating commands should load canonical store, validate, mutate, save metadata, regenerate managed aliases, and print reload hints.
- `add` fails on duplicate names.
- `update` and `delete` fail on missing names.
- `list` should render a compact table with at least name and command; include description/source/tags if cheap.
- Use the same validation helpers from Phase 1.
- Keep PowerShell and shell-specific syntax out of this phase.

## Tests To Require

- Add writes canonical store and regenerated aliases.
- Duplicate add fails with actionable error.
- Update changes command and regenerated aliases.
- Delete removes command and regenerated aliases.
- Missing update/delete reports useful error.
- Protected name add requires `--force`.
- List renders a readable table containing alias name and command.

