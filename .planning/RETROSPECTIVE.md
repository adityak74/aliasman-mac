# Retrospective: aliasman

## Milestone: v0.0.1 — CLI Alias Manager MVP

**Shipped:** 2026-05-16
**Phases:** 6 | **Plans:** 6
**Timeline:** 5 days (May 10 → May 15)
**LOC:** 4,319 Rust

### What Was Built

- Rust CLI scaffold with clap-derived subcommands for all 10 CLI commands
- Full alias CRUD (add/update/delete/list) with validation and protected-name policy
- Shell detection for zsh/bash with idempotent config management
- Alias import from existing shell configs with deduplication
- History-based command frequency stats and alias suggestions with risky-command detection
- Claude Code hook with project-context filtering and 500-token budget
- Local semantic search via LanceDB + Ollama embeddings (768-dim, no data leaves machine)
- MCP stdio server for Claude alias_search integration

### What Worked

- **Phase-by-phase execution** — Each phase built on prior work cleanly; dependencies were well-defined
- **Atomic write pattern** — Single `write_atomic()` helper reused across store, shell config, and hook installation
- **Local-only embeddings** — Ollama on localhost keeps everything private, no API keys needed
- **std::thread::spawn for background tasks** — Clever workaround for sync CLI context, avoids tokio dependency
- **Managed block markers** — Simple `# >>> aliasman >>>` / `# <<< aliasman <<<` pattern makes idempotency trivial

### What Was Inefficient

- **Missing VERIFICATION.md for phases 2-5** — Verification was done but not documented until post-hoc; should create VERIFICATION.md during execute-phase, not after
- **REQUIREMENTS.md never updated during execution** — All 29 entries stayed "Pending" until audit; should update traceability as phases complete
- **SUMMARY.md missing for phases 2-6** — Only phase 1 had a summary; SDK could only extract one-liner from phase 1
- **Hardcoded `.aliases` path** — `import.rs` hardcodes `$HOME/.aliases` in managed block even when `--aliases-file` specifies custom path

### Patterns Established

- **VERIFICATION.md per phase** — 5-check table mapping requirements to evidence
- **SUMMARY.md per plan** — Frontmatter with `requirements-completed` list for automated cross-reference
- **Background index refresh** — `std::thread::spawn` + embedded Tokio runtime for async-in-sync pattern
- **Lexical fallback** — When semantic search is unavailable, fall back to keyword matching (applies to search and MCP)

### Key Lessons

- Always create VERIFICATION.md during phase execution, not retroactively
- Update REQUIREMENTS.md traceability table as each phase completes
- SUMMARY.md frontmatter is critical for automated milestone completion (SDK extracts accomplishments from it)

### Cost Observations

- Model mix: Primarily Opus for execution, balanced profile for planning
- Sessions: ~3 (initial setup, autonomous execution, audit/completion)
- Notable: Autonomous execution completed all 6 phases without user intervention; verification documentation was the only gap

---

## Cross-Milestone Trends

(Will be populated after v0.0.2+)
