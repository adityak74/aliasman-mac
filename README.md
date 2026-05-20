# aliasman

A terminal alias manager for macOS with semantic search, shell history analytics, and Claude Code integration.

Manage your `zsh`/`bash` aliases from the CLI — add, update, delete, list, and search — without ever manually editing your shell config.

---

## Features

- **CRUD for aliases** — add, update, delete, list with tags and descriptions
- **Shell history analytics** — see your most-used commands and get alias suggestions
- **Semantic search** — find aliases by meaning using local embeddings (LanceDB + Ollama)
- **MCP server** — expose alias search to Claude and other MCP-compatible tools
- **Claude Code hook** — injects your aliases into Claude's session context intelligently (token-budget aware)
- **Safe shell injection** — writes a managed block into your `.zshrc`/`.bashrc`, never corrupts the file
- **Atomic persistence** — TOML data store with timestamped backups

---

## Installation

### Homebrew

```bash
brew tap adityak74/aliasman
brew install aliasman
```

Then initialize for your shell:

```bash
aliasman init
```

This auto-detects your shell, imports any existing aliases, and injects a managed block into your shell config.

### From source

```bash
cargo install --path .
```

Then initialize for your shell:

```bash
aliasman init
```

---

## Commands

### Alias management

```bash
# Add an alias
aliasman add --name gs --command "git status"
aliasman add --name k --command "kubectl" --description "k8s shorthand" --tag k8s

# Update an alias
aliasman update --name gs --command "git status --short"

# Delete an alias
aliasman delete --name gs

# List all aliases
aliasman list
aliasman list --shell zsh
```

### Shell history

```bash
# Show command frequency stats
aliasman stats
aliasman stats --top 30 --verbose

# Suggest aliases for frequent long commands
aliasman suggest

# Apply a suggestion
aliasman suggest --apply <alias-name>
```

### Semantic search

Requires [Ollama](https://ollama.com) running locally with an embedding model.

```bash
# Search aliases by meaning
aliasman search "kubernetes port forward"
aliasman search "git undo last commit" --limit 10

# Rebuild the search index
aliasman search reindex
```

### Claude Code integration

```bash
# Install the SessionStart hook into Claude Code settings
aliasman hook install

# Preview what the hook would inject
aliasman hook preview

# Run as hook (called automatically by Claude Code)
aliasman hook claude
```

The hook injects your aliases into Claude's context on session start, staying within a configurable token budget so it doesn't bloat every conversation.

### MCP server

```bash
# Start the MCP stdio server (for use with Claude or other MCP clients)
aliasman mcp serve
```

Exposes an `alias_search` tool that MCP clients can call to find relevant aliases by natural language query.

---

## Data files

| File | Purpose |
|---|---|
| `~/.config/aliasman/aliases.toml` | Canonical alias store |
| `~/.aliases` | Generated shell alias file (sourced by your shell config) |

---

## Requirements

- **Rust** 1.75+
- **Ollama** (optional) — for semantic search; run `ollama pull nomic-embed-text`

---

## License

MIT
