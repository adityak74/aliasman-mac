# Phase 6 Research: Local Semantic Alias Search With LanceDB And MCP

**Phase:** 06 - Local Semantic Alias Search With LanceDB And MCP
**Researched:** 2026-05-12
**Status:** Ready for planning

## Phase Goal

Add local semantic alias search using Ollama embeddings, LanceDB vector storage, CLI natural-language search, and one MCP `alias_search` tool that lets Claude search aliases on demand.

## Requirements Covered

- **SEM-01:** User aliases are embedded locally and indexed in a LanceDB-backed vector store.
- **SEM-02:** aliasman can refresh the semantic alias index when aliases are added, updated, deleted, imported, or suggested.
- **SEM-03:** User can search aliases semantically from the CLI using natural-language queries.
- **SEM-04:** Claude can search aliases semantically through a local MCP tool/server instead of relying only on hook-injected context.
- **SEM-05:** Semantic search runs locally by default and does not send alias commands or metadata to remote embedding services unless the user explicitly configures that later.

## Primary Source Findings

### LanceDB

Sources:
- https://docs.rs/lancedb
- https://docs.lancedb.com/tables/create
- https://docs.lancedb.com/search/vector-search

Relevant findings:
- LanceDB OSS runs locally as an embedded library and has a native Rust SDK.
- Rust connects with `lancedb::connect("path").execute().await`.
- LanceDB stores data in Arrow-backed tables; vector columns should use `FixedSizeList<Float32>` or `FixedSizeList<Float16>`.
- Rust table creation uses Arrow `RecordBatch` / `RecordBatchIterator`.
- Search uses `table.query().nearest_to(&query_vector)?.limit(n).execute().await?`.
- LanceDB supports local persistent database paths, which fits an aliasman-owned data directory.

Implementation implication:
- Add async runtime support with Tokio if not already present.
- Store semantic index under aliasman config/data, not in the shell alias file.
- Use a table such as `aliases` with columns for alias name, command, searchable text, tags/source/shell metadata, updated timestamp, and vector.

### Ollama Embeddings

Sources:
- https://docs.ollama.com/api
- https://docs.ollama.com/capabilities/embeddings

Relevant findings:
- Ollama’s local API default base URL is `http://localhost:11434/api`.
- Embeddings are generated with `POST /api/embed`.
- The embeddings request accepts `model` and `input`.
- Recommended embedding models include `embeddinggemma`, `qwen3-embedding`, and `all-minilm`.
- Embedding vector length depends on model, typically 384-1024 dimensions.

Implementation implication:
- Use `reqwest` to call local Ollama rather than adding an unofficial Rust Ollama dependency.
- Default model should be `embeddinggemma` unless the implementation finds a stronger local default.
- Store the embedding dimension with index metadata or infer it when creating the table.
- Treat Ollama unavailability as recoverable: warn and fallback to lexical search.

### MCP Tool Server

Sources:
- https://modelcontextprotocol.io/specification/2025-06-18/server/tools
- https://modelcontextprotocol.io/docs/sdk
- https://github.com/modelcontextprotocol/rust-sdk
- https://docs.rs/rmcp

Relevant findings:
- MCP servers expose tools; clients discover them via `tools/list` and invoke them with `tools/call`.
- Tool definitions include a unique `name`, human-readable `description`, and JSON Schema `inputSchema`.
- Official SDKs include a Rust SDK; docs classify Rust as Tier 2.
- The Rust SDK crate is `rmcp`; official examples use `#[tool]`, `#[tool_router]`, `#[tool_handler]`, `ServiceExt`, and `transport::stdio`.
- A stdio server is appropriate for a local CLI-integrated MCP tool.

Implementation implication:
- Add a `mcp` or `mcp serve` command that starts a stdio MCP server.
- Expose one tool named `alias_search` with `query: string` and optional `limit: integer`.
- Return structured results with alias name, command, score/rank, reason, and optional warning.
- Keep the MCP tool read-only. It should search aliases, not mutate shell config or alias data.

## Recommended Dependencies

Tentative additions:

```toml
lancedb = "0.27"
arrow-array = "57"
arrow-schema = "57"
futures = "0.3"
reqwest = { version = "0.12", features = ["json"] }
rmcp = { version = "1", features = ["server", "macros", "schemars"] }
schemars = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

The executor should verify exact compatible versions during implementation. If Arrow version constraints are pulled transitively by LanceDB, prefer versions compatible with the selected `lancedb` crate.

## Data Model Recommendation

Table: `aliases`

Columns:
- `name`: alias name, unique logical key
- `command`: alias command
- `search_text`: text embedded for semantic search, e.g. `"{name}\n{command}\n{description}\n{tags}"`
- `tags`: serialized or list metadata depending on Arrow schema support
- `source`: user/imported/suggested
- `shell`: all/zsh/bash
- `updated_at`: u64 timestamp from canonical alias metadata
- `vector`: fixed-size float32 vector

Index metadata:
- embedding provider: `ollama`
- embedding model: e.g. `embeddinggemma`
- vector dimension
- last indexed alias updated timestamp or store fingerprint

## CLI Recommendation

Required commands:
- `aliasman search "natural language query"`
- `aliasman search --limit 10 "query"`
- `aliasman search reindex`
- `aliasman mcp serve` or `aliasman mcp --serve`

Default search output:

```text
Alias  Command       Score  Reason
gs     git status    0.84   matched git status/check working tree intent
```

If Ollama or LanceDB fails:
- CLI prints a warning.
- CLI falls back to lexical search across alias name, command, description, and tags.

## MCP Recommendation

Tool: `alias_search`

Input schema:
- `query`: string, required
- `limit`: integer, optional, default 5

Output:
- Structured results with alias, command, score/rank, reason, and warning when fallback was used.
- Text content can include the same JSON serialized for backwards compatibility.

## Testing Strategy

Do not require live Ollama for default tests. Use dependency injection:
- Mock embedding provider returning deterministic vectors.
- Tempdir LanceDB path.
- Fixture alias store.
- Lexical fallback tests with forced embedding/index errors.
- MCP tool handler tests can call the handler directly without launching a real Claude client.

Required tests:
- Reindex creates/updates a LanceDB table from aliases using mocked embeddings.
- Add/update/delete/import/suggestion-apply hooks call index refresh helper.
- Search returns ranked results with alias, command, score, and reason.
- `--limit` limits result count.
- Ollama failure triggers warning plus lexical fallback in CLI path.
- MCP `alias_search` returns structured warning plus fallback results on embedding/index failure.
- No test sends alias content to a remote network endpoint.

## Risks And Mitigations

- **Version/API churn:** LanceDB and RMCP Rust APIs may evolve. Mitigate by keeping integration localized in `search.rs` and `mcp.rs`.
- **Ollama not installed/running:** Treat as recoverable and provide setup guidance plus lexical fallback.
- **Vector dimension mismatch after model change:** Store model/dimension metadata and rebuild the index when it changes.
- **Index staleness:** Refresh on every alias mutation and provide `search reindex`.
- **MCP tool overreach:** Keep `alias_search` read-only in this phase.

