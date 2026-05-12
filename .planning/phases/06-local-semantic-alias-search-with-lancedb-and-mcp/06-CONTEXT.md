# Phase 6: Local Semantic Alias Search With LanceDB And MCP - Context

**Gathered:** 2026-05-11
**Status:** Ready for planning

<domain>
## Phase Boundary

This phase adds local semantic alias search. It should embed aliases locally using Ollama embeddings, store vectors in a LanceDB-backed index, keep that index current as aliases change, expose CLI natural-language alias search, and expose a focused MCP `alias_search` tool so Claude can search aliases on demand. It does not add remote embedding services, Codex integration, CLAUDE.md generation, PowerShell support, or cloud sync.

</domain>

<decisions>
## Implementation Decisions

### Embedding Provider
- **D-01:** Use Ollama embeddings as the v0.0.1 local embedding backend.
- **D-02:** Embedding must stay local by default; alias commands and metadata must not be sent to remote embedding services.
- **D-03:** Do not implement fastembed/candle in-process embeddings in this phase.
- **D-04:** A provider abstraction can exist if useful, but Ollama is the only required provider for v0.0.1.

### Index Refresh Behavior
- **D-05:** Refresh the LanceDB semantic index automatically on every alias mutation: add, update, delete, import, and suggestion-apply.
- **D-06:** Also provide an explicit repair command such as `aliasman search reindex`.
- **D-07:** Search results should not depend on users remembering to manually reindex after normal alias changes.

### Search Command UX
- **D-08:** Use `aliasman search "query"` as the primary CLI semantic search command.
- **D-09:** Default search output should be a ranked table with columns `Alias`, `Command`, `Score`, and `Reason`.
- **D-10:** Default result count should be top 5.
- **D-11:** Support an optional `--limit <n>` flag.
- **D-12:** Do not use `aliasman semantic-search` as the primary command name.

### MCP Surface
- **D-13:** Expose one focused MCP tool named `alias_search`.
- **D-14:** `alias_search` should accept at least `query` and optional `limit`.
- **D-15:** The MCP response should return ranked aliases with commands and reasons.
- **D-16:** Do not add MCP resources or a separate `alias_lookup` tool in v0.0.1 unless the planner finds it essentially free and non-disruptive.

### Privacy And Failure Mode
- **D-17:** If Ollama, LanceDB, or MCP search fails, warn the user and fall back to lexical search where possible.
- **D-18:** CLI fallback should show a clear warning and substring/tag matching results.
- **D-19:** Claude MCP fallback should return a structured warning plus fallback results.
- **D-20:** Do not silently degrade in user-facing CLI search.
- **D-21:** Do not hard-fail without fallback when lexical fallback is possible.

### the agent's Discretion
The planner may choose the Ollama embedding model default, LanceDB table schema, exact score formatting, reason wording, MCP server transport details, and lexical fallback ranking, provided the locked decisions above and requirements are satisfied.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Planning Artifacts
- `.planning/PROJECT.md` — Project vision, active semantic search scope, and constraints.
- `.planning/REQUIREMENTS.md` — Phase 6 requirement IDs `SEM-01` through `SEM-05`.
- `.planning/ROADMAP.md` — Phase 6 goal, success criteria, and dependency on Phase 5.
- `.planning/STATE.md` — Current milestone state.

### Prior Phases
- `.planning/phases/03-alias-crud-and-listing/03-CONTEXT.md` — CRUD mutation behavior and reload/error decisions; index refresh must hook into these mutation flows.
- `.planning/phases/04-history-stats-and-suggestions/04-CONTEXT.md` — Suggestion-apply behavior; index refresh must handle accepted suggestions.
- `.planning/phases/05-claude-hook-integration/05-CONTEXT.md` — Claude integration decisions; MCP search is the on-demand complement to token-capped hook injection.

### Research
- `.planning/research/STACK.md` — Rust CLI stack baseline.
- `.planning/research/ARCHITECTURE.md` — Alias store and Claude integration architecture.
- `.planning/research/PITFALLS.md` — Token overload, local file safety, and risky command context.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Reuse canonical alias metadata and tags/source fields from the alias store.
- Reuse CRUD mutation hooks from Phase 3 for automatic semantic index refresh.
- Reuse suggestion-apply path from Phase 4 for index refresh after accepted suggestions.
- Reuse Claude integration patterns from Phase 5 for local tool behavior and quiet/default-safe Claude interactions.

### Established Patterns
- User-facing commands use explicit names and named flags.
- Mutating flows must avoid touching real home files in tests.
- Claude-facing no-op/error paths should avoid polluting context, but Phase 6 MCP failures should return structured warnings because Claude explicitly asked for search.

### Integration Points
- A likely `src/search.rs` owns embedding, LanceDB indexing/search, lexical fallback, and CLI search behavior.
- A likely `src/mcp.rs` or extension of `src/hook.rs` owns the MCP `alias_search` tool.
- CRUD/import/suggestion-apply code paths must call semantic index refresh helpers after successful alias mutations.

</code_context>

<specifics>
## Specific Ideas

- Primary CLI shape: `aliasman search "how do I check git status"`.
- Default search output columns: `Alias`, `Command`, `Score`, `Reason`.
- Default result count: top 5.
- Repair command: `aliasman search reindex`.
- MCP tool: `alias_search(query, limit?)`.

</specifics>

<deferred>
## Deferred Ideas

- In-process fastembed/candle embeddings.
- Remote embedding providers.
- Additional MCP tools such as `alias_lookup`.
- MCP resources exposing all aliases or index metadata.
- Codex semantic-search integration.

</deferred>

---

*Phase: 6-Local Semantic Alias Search With LanceDB And MCP*
*Context gathered: 2026-05-11*
