# Phase 4 Research: History Stats And Suggestions

**Phase:** 04 - History Stats And Suggestions
**Researched:** 2026-05-11
**Status:** Ready for planning

## Phase Goal

Use zsh/bash shell history to show command frequency analytics and suggest useful aliases safely. Suggestions must remain review-only until the user explicitly accepts them.

## Requirements Covered

- **HST-01:** User can view command frequency statistics from zsh or bash history.
- **HST-02:** aliasman correctly parses zsh extended history format.
- **HST-03:** aliasman suggests short aliases for frequent long commands.
- **HST-04:** User must explicitly approve a history-derived suggestion before it becomes an alias.
- **HST-05:** aliasman flags risky history-derived commands that contain command substitution or other shell metacharacters.

## Implementation Guidance

- Add `stats` and `suggest` subcommands.
- Add `history.rs` with parsers for bash plain history, bash timestamped history, zsh plain history, and zsh extended history.
- Use streaming or bounded reads; avoid naive full-file assumptions.
- Aggregate command frequency with a map.
- Suggestions should score by frequency and command length.
- Do not auto-create aliases from suggestions. Provide an explicit accept/apply path only if it routes through existing CRUD validation and force handling.
- Flag risky commands containing `$(`, backticks, `<(`, `|`, `;`, `&&`, or `||`.

## Tests To Require

- zsh extended history parser strips timestamp/duration prefix.
- bash timestamped history parser associates commands without treating timestamps as commands.
- stats rank commands by frequency.
- suggestions prefer frequent long commands.
- risky command detection flags command substitution and shell chaining.
- suggestions do not write aliases unless explicit accept/apply path is invoked.

