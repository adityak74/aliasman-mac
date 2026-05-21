# aliasman

<p align="center">
  <img src="https://img.shields.io/github/v/release/adityak74/aliasman-mac?style=flat-square&color=orange&label=version" alt="Version" />
  <img src="https://img.shields.io/badge/rust-1.75%2B-orange?style=flat-square&logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/platform-macOS-lightgrey?style=flat-square&logo=apple" alt="macOS" />
  <img src="https://img.shields.io/github/license/adityak74/aliasman-mac?style=flat-square" alt="License" />
  <img src="https://img.shields.io/github/stars/adityak74/aliasman-mac?style=flat-square&color=yellow" alt="Stars" />
  <img src="https://img.shields.io/badge/homebrew-tap-brown?style=flat-square&logo=homebrew" alt="Homebrew" />
</p>

<p align="center">
  <strong>An AI-harness based alias manager for macOS — semantic search, shell history analytics, and native Claude Code integration.</strong>
</p>

<p align="center">
  Manage your <code>zsh</code>/<code>bash</code> aliases from the CLI — add, update, delete, list, and search — without ever manually editing your shell config.
</p>

---

<p align="center">
  <a href="#installation">Install</a> •
  <a href="#commands">Commands</a> •
  <a href="#claude-code-integration">Claude Code</a> •
  <a href="#mcp-server">MCP</a> •
  <a href="#requirements">Requirements</a>
</p>

---

## Features

| Feature | Description |
|---|---|
| **Alias CRUD** | Add, update, delete, and list aliases with tags and descriptions |
| **History analytics** | See your most-used commands and get alias suggestions |
| **Semantic search** | Find aliases by meaning using local embeddings (LanceDB + Ollama) |
| **MCP server** | Expose alias search to Claude and other MCP-compatible tools |
| **Claude Code hook** | Injects your aliases into Claude's session context (token-budget aware) |
| **Safe shell injection** | Writes a managed block into `.zshrc`/`.bashrc` — never corrupts the file |
| **Alias packs** | Install, share, and manage curated alias collections (k8s, docker, and more) |
| **Atomic persistence** | TOML data store with timestamped backups |

---

## Installation

### Homebrew (recommended)

```bash
brew tap adityak74/aliasman
brew install adityak74/aliasman/aliasman
```

### From source

```bash
cargo install --path .
```

### Initialize

After installing, run:

```bash
aliasman init
```

This auto-detects your shell, imports any existing aliases, and injects a managed block into your shell config.

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

### Alias packs

```bash
# Install the built-in k8s pack
aliasman pack install-builtin k8s

# Install a pack from a file or URL
aliasman pack install ./my-pack.toml
aliasman pack install https://example.com/packs/docker.toml

# List installed packs
aliasman pack list

# Remove a pack
aliasman pack remove k8s

# Create a pack from your current aliases
aliasman pack create --name my-aliases --tag work
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
| `~/.config/aliasman/packs/` | Installed pack files |
| `~/.aliases` | Generated shell alias file (sourced by your shell config) |

---

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=adityak74/aliasman-mac&type=Date)](https://star-history.com/#adityak74/aliasman-mac&Date)

---

## Requirements

- **Rust** 1.75+
- **Ollama** (optional) — for semantic search; run `ollama pull nomic-embed-text`

---

## Contributing

Issues and PRs are welcome. See [GitHub Issues](https://github.com/adityak74/aliasman/issues) to report bugs or request features.

---

## License

MIT

---

<p align="center">
  If <strong>aliasman</strong> saves you time, consider giving it a ⭐ on <a href="https://github.com/adityak74/aliasman-mac">GitHub</a> — it helps others find the project.
</p>
