# Phase 6: Local Semantic Alias Search With LanceDB And MCP - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-11
**Phase:** 6-Local Semantic Alias Search With LanceDB And MCP
**Areas discussed:** Embedding provider, Index refresh behavior, Search command UX, MCP surface, Privacy and failure mode

---

## Embedding Provider

| Option | Description | Selected |
|--------|-------------|----------|
| Ollama embeddings | Use local Ollama if available; easy for developer machines and keeps data local, but requires Ollama/model setup. | ✓ |
| fastembed/candle in-process | No external service once dependencies are installed, but Rust integration and model packaging may be heavier. | |
| Provider abstraction with Ollama first | Implement an embedding trait and ship Ollama as the first provider; leaves room for fastembed later. | |

**User's choice:** Ollama embeddings.
**Notes:** Keep embeddings local by default. Provider abstraction is optional, but only Ollama is required in v0.0.1.

---

## Index Refresh Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Automatic refresh on every mutation | Add/update/delete/import/suggestion-apply update the index immediately; also provide `aliasman search reindex` for repair. | ✓ |
| Explicit reindex only | Simpler and safer, but users/Claude may see stale results. | |
| Lazy refresh on search | Detect stale index during search and refresh then. | |

**User's choice:** Automatic refresh on every mutation.
**Notes:** Also provide explicit reindex command for repair.

---

## Search Command UX

| Option | Description | Selected |
|--------|-------------|----------|
| `aliasman search "query"` with ranked table | Columns: `Alias`, `Command`, `Score`, `Reason`; default top 5, optional `--limit`. | ✓ |
| `aliasman semantic-search "query"` | More explicit, but longer. | |
| Subcommands under search | `aliasman search query "..."`, `aliasman search reindex`; clearer grouping but more verbose. | |

**User's choice:** `aliasman search "query"` with ranked table.
**Notes:** Include `aliasman search reindex` as a repair command.

---

## MCP Surface

| Option | Description | Selected |
|--------|-------------|----------|
| One `alias_search` tool | Claude calls `alias_search(query, limit?)` and gets ranked aliases with commands/reasons. Simple and focused. | ✓ |
| Two tools: `alias_search` + `alias_lookup` | Semantic search plus exact lookup by alias name. | |
| Tools plus resources | Tool calls for search, plus MCP resources exposing all aliases or index metadata. | |

**User's choice:** One `alias_search` tool.
**Notes:** Do not add resources or lookup tool unless essentially free and non-disruptive.

---

## Privacy And Failure Mode

| Option | Description | Selected |
|--------|-------------|----------|
| Warn user, fallback to lexical search where possible | CLI shows a clear warning and does substring/tag matching; Claude MCP returns a structured warning plus fallback results. | ✓ |
| Silent degradation | Quietly fallback if possible, otherwise return no results. | |
| Hard fail | Surface the error and do not fallback. | |

**User's choice:** Warn user, fallback to lexical search where possible.
**Notes:** CLI warning should be visible. MCP should return structured warning plus fallback results.

---

## the agent's Discretion

- Ollama embedding model default.
- LanceDB table schema.
- Score formatting.
- Reason wording.
- MCP server transport details.
- Lexical fallback ranking.

## Deferred Ideas

- fastembed/candle in-process embeddings.
- Remote embedding providers.
- Additional MCP tools/resources.
- Codex semantic-search integration.
