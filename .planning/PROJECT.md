# aliasman

## What This Is

A Rust CLI that makes shell alias management a first-class developer experience. It manages aliases in a dedicated `~/.aliases` file (sourced by your shell config), provides full CRUD via named CLI flags, auto-detects existing aliases from zsh/bash config files, suggests new aliases from your shell history, integrates with Claude Code via a hook that intelligently injects relevant aliases into each session, supports local semantic search via LanceDB + MCP so Claude can find aliases by natural language, and allows creating/sharing/installing alias packs for common toolchains.

## Core Value

Developers never have to manually edit shell config files to manage aliases — and their AI coding assistant knows and uses those same aliases.

## Current State

**Shipped:** v0.1 Alias Library (2026-05-19)
**LOC:** ~5,200 Rust (v0.1 baseline)
**Tech stack:** Rust, clap, TOML, LanceDB, Ollama, MCP/JSON-RPC, reqwest

## Requirements

### Validated

- ✓ CLI alias CRUD: add, update, delete, list via named flags — v0.0.1
- ✓ Auto-detect existing aliases from zsh/bash config on init — v0.0.1
- ✓ Dedicated `~/.aliases` file with atomic writes — v0.0.1
- ✓ History-based alias suggestions with risky-command detection — v0.0.1
- ✓ Command frequency statistics from shell history — v0.0.1
- ✓ Claude Code hook with smart project-context filtering — v0.0.1 (500-token budget)
- ✓ Local LanceDB vector index for semantic search — v0.0.1 (768-dim Ollama embeddings)
- ✓ MCP stdio server for Claude alias_search tool — v0.0.1
- ✓ Console help for all commands — v0.0.1
- ✓ Shell reload hints after mutations — v0.0.1
- ✓ Alias pack creation — group aliases into named, versioned packs — v0.1
- ✓ Pack sharing — export packs as shareable files — v0.1
- ✓ Pack installation — install packs from file/URL with safety scanning — v0.1
- ✓ Built-in packs — ship curated packs for k8s (22 aliases) and docker (16 aliases) — v0.1
- ✓ Pack management — list, remove installed packs with modified_by_user preservation — v0.1
- ✓ Pack alias merge-at-render into ~/.aliases — v0.1
- ✓ Hook scoring for pack aliases alongside user/imported — v0.1
- ✓ Semantic re-index after pack install/remove — v0.1
- ✓ List output grouped by source (user, imported, suggested, pack) — v0.1

### Out of Scope

- CLAUDE.md file injection — Hook-only approach keeps it dynamic; confirmed working
- Team/org shared alias libraries — Moving to v0.2 as remote pack registry
- Homebrew formula publishing — Build the tool first; formula after v1 is solid
- Real-time alias sync — No cloud or sync service in v1
- PowerShell support — Different alias/function model; deferred until zsh/bash model is stable

## Context

Shipped v0.1 with ~5,200 LOC Rust across 15 source files and 3 integration test files.
Tech stack: clap (CLI), TOML (store), LanceDB (vector search), Ollama (local embeddings), MCP/JSON-RPC (Claude integration), reqwest (URL pack downloads).
All embeddings run locally on `localhost:11434` — no data leaves the machine.
Background index refresh uses `std::thread::spawn` (not tokio) for sync CLI compatibility.

## Constraints

- **Tech Stack**: Rust — chosen for single-binary distribution, performance, and shell tooling ecosystem
- **Platform**: macOS first; Linux compatibility is a goal but not a blocker for v1
- **Distribution**: Homebrew primary — must produce a valid Homebrew formula and binary release

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Dedicated `~/.aliases` file vs editing shell config directly | Safer, easier to version, avoids corrupting zshrc | ✓ Good — atomic writes, clean separation |
| Claude Code hook (not CLAUDE.md injection) for v1 | Dynamic and session-aware; no stale static file | ✓ Good — project-context filtering works |
| Smart alias filtering in hook (not dump all) | Token efficiency — Claude sessions have context limits | ✓ Good — 500-token budget, relevance scoring |
| Rust over Go | Single binary, better shell tooling ergonomics | ✓ Good — ~5,200 LOC, clean patterns |
| Homebrew as primary distribution | macOS developer standard; expected install UX | — Pending (deferred to v0.0.2+) |
| Local-only embeddings (Ollama) | Privacy — no alias data leaves machine | ✓ Good — no external API calls |
| LanceDB for vector index | Embedded, no server needed, Rust-native | ✓ Good — works with 768-dim embeddings |
| `std::thread::spawn` for background refresh | No active tokio runtime in sync CLI | ✓ Good — avoids runtime dependency |
| MCP stdio server for Claude integration | Standard protocol, works with Claude Code natively | ✓ Good — alias_search tool works |
| Managed block markers in shell config | Idempotent insertion, easy detection | ✓ Good — prevents duplicate source blocks |
| Flat pack names + collision detection | Simpler UX than auto-prefixing | ✓ Good — user aliases always win |
| File + URL install only (no git for v0.1) | Minimal dependencies, fast to ship | ✓ Good — git deferred to v0.2 |
| Soft safety warnings with --force | Balance security with user autonomy | ✓ Good — blocks dangerous patterns by default |
| Pack aliases merge at render time | Clean separation, no storage-time coupling | ✓ Good — .aliases always reflects current state |
| Built-in packs as files (not embedded) | No binary bloat, easy to update | ✓ Good — 22 k8s + 16 docker aliases |
| modified_by_user preservation on remove | Users who customize pack aliases expect to keep changes | ✓ Good — preserves user intent |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition:** Requirements validated/invalidated, new requirements captured, decisions logged.

**After each milestone:** Full review of all sections, core value check, out-of-scope audit.

---
*Last updated: 2026-05-19, milestone v0.1 shipped*
