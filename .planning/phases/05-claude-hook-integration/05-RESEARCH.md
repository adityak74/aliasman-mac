# Phase 5 Research: Claude Hook Integration

**Phase:** 05 - Claude Hook Integration
**Researched:** 2026-05-11
**Status:** Ready for planning

## Phase Goal

Install a Claude Code `SessionStart` hook and implement a hook runner that emits relevant alias context without overwriting existing Claude settings or overloading tokens.

## Requirements Covered

- **CLD-01:** User can install an aliasman Claude Code `SessionStart` hook without overwriting existing Claude settings.
- **CLD-02:** The Claude hook command emits valid hook JSON containing alias context when relevant aliases exist.
- **CLD-03:** The Claude hook filters aliases by current project context instead of injecting the entire alias list.
- **CLD-04:** The Claude hook respects a configurable token budget for injected context.
- **CLD-05:** The Claude hook exits cleanly with no noisy output when no alias context should be injected.

## Implementation Guidance

- Add `hook install claude` and `hook --shell claude` or equivalent subcommands.
- Hook install merges into `~/.claude/settings.json`; never clobber unrelated hooks/settings.
- Use `serde_json` for settings merge and atomic write helpers for settings writes.
- Hook runner reads stdin JSON and emits Claude hook JSON only when useful.
- SessionStart output format:
  ```json
  {
    "hookSpecificOutput": {
      "hookEventName": "SessionStart",
      "additionalContext": "..."
    }
  }
  ```
- Relevance signals: `.git`, `Cargo.toml`, `package.json`, `Dockerfile`, alias tags/source, recent aliases if available.
- Token budget can be approximate: one token per four characters is acceptable for capping.
- If no aliases exist or no context should be injected, exit 0 with no stdout.

## Tests To Require

- Installing hook preserves unrelated Claude settings.
- Installing hook twice is idempotent.
- Hook runner emits valid JSON for relevant aliases.
- Hook runner filters aliases by project context.
- Hook runner respects budget.
- Hook runner emits no output when no aliases exist.

