use std::fs;
use std::path::{PathBuf, Path};

use aliasman::history::{
    command_frequencies, detect_history_file, format_stats, format_verbose_stats,
    generate_suggestions, read_history_file,
};
use aliasman::hook::{
    create_install_preview, detect_project_context, format_alias_context, get_relevant_aliases,
    has_aliasman_hook, HookOutput, install_claude_hook, merge_aliasman_hook, run_claude_hook,
    ClaudeSettings, DEFAULT_MAX_TOKENS,
};
use aliasman::import::{
    build_imported_records, ensure_managed_block, get_reload_hint, has_managed_block,
    merge_imported_aliases, parse_alias_lines, shell_kind_to_alias_shell,
};
use aliasman::mcp::run_mcp_server;
use aliasman::model::{AliasRecord, AliasShell, AliasSource};
use aliasman::pack_registry::PackRegistry;
use aliasman::pack_installer::{
    install_pack, parse_pack_file, download_pack,
    create_install_preview as create_pack_install_preview,
};
use aliasman::pack_manager::{
    add_alias_to_pack, create_pack, export_pack_to_file,
    pack_exists,
};
use aliasman::search::{
    default_index_path, lexical_search, reindex_aliases, search_aliases,
    OllamaEmbeddingProvider, SearchResult,
};
use chrono::TimeZone;
use aliasman::shell::{detect_shell_and_config, DetectResult, ShellKind};
use aliasman::store::{
    backup_file, prune_backups, store_add_alias, store_delete_alias, store_list_aliases,
    store_update_alias, write_managed_aliases, AliasStore,
};
use aliasman::validation::validate_alias_name_for_write;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "aliasman")]
#[command(version)]
#[command(about = "Manage shell aliases safely")]
enum Cli {
    /// Initialize aliasman for your shell — detect, import, and configure
    Init {
        /// Force a specific shell instead of auto-detecting
        #[arg(long)]
        shell: Option<String>,
        /// Path to the canonical aliasman data file (default: ~/.config/aliasman/aliases.toml)
        #[arg(long)]
        data_file: Option<PathBuf>,
        /// Path to the managed aliases output file (default: ~/.aliases)
        #[arg(long)]
        aliases_file: Option<PathBuf>,
        /// Home directory override (for testing)
        #[arg(long, hide = true)]
        home: Option<PathBuf>,
        /// The $SHELL value override (for testing)
        #[arg(long, hide = true)]
        shell_env: Option<String>,
        /// Apply changes without prompting (non-interactive)
        #[arg(long)]
        apply: bool,
    },

    /// Add a new alias
    Add {
        /// Alias name
        #[arg(long)]
        name: String,
        /// Alias command
        #[arg(long)]
        command: String,
        /// Optional description
        #[arg(long)]
        description: Option<String>,
        /// Optional tags (can be repeated)
        #[arg(long)]
        tag: Vec<String>,
        /// Force alias creation for protected command names
        #[arg(long)]
        force: bool,
        /// Path to the canonical aliasman data file
        #[arg(long)]
        data_file: Option<PathBuf>,
        /// Path to the managed aliases output file
        #[arg(long)]
        aliases_file: Option<PathBuf>,
    },

    /// Update an existing alias
    Update {
        /// Alias name to update
        #[arg(long)]
        name: String,
        /// New alias command
        #[arg(long)]
        command: Option<String>,
        /// New description (use "" to clear)
        #[arg(long)]
        description: Option<Option<String>>,
        /// Replace all tags with these (can be repeated)
        #[arg(long)]
        tag: Option<Vec<String>>,
        /// Force update for protected command names
        #[arg(long)]
        force: bool,
        /// Path to the canonical aliasman data file
        #[arg(long)]
        data_file: Option<PathBuf>,
        /// Path to the managed aliases output file
        #[arg(long)]
        aliases_file: Option<PathBuf>,
    },

    /// Delete an alias by name
    Delete {
        /// Alias name to delete
        #[arg(long)]
        name: String,
        /// Path to the canonical aliasman data file
        #[arg(long)]
        data_file: Option<PathBuf>,
        /// Path to the managed aliases output file
        #[arg(long)]
        aliases_file: Option<PathBuf>,
    },

    /// List all aliases in a table
    List {
        /// Filter by shell (zsh, bash, all)
        #[arg(long)]
        shell: Option<String>,
        /// Path to the canonical aliasman data file
        #[arg(long)]
        data_file: Option<PathBuf>,
    },

    /// Show command frequency statistics from shell history
    Stats {
        /// Path to shell history file (default: auto-detect from $HISTFILE)
        #[arg(long)]
        history_file: Option<PathBuf>,
        /// Show verbose stats with percentages and tool grouping
        #[arg(long)]
        verbose: bool,
        /// Number of top commands to show (default: 20)
        #[arg(long, default_value = "20")]
        top: usize,
    },

    /// Suggest aliases for frequent long commands
    Suggest {
        /// Path to shell history file (default: auto-detect from $HISTFILE)
        #[arg(long)]
        history_file: Option<PathBuf>,
        /// Path to the canonical aliasman data file (to check existing aliases)
        #[arg(long)]
        data_file: Option<PathBuf>,
        /// Path to the managed aliases output file
        #[arg(long)]
        aliases_file: Option<PathBuf>,
        /// Apply a specific suggestion by alias name
        #[arg(long)]
        apply: Option<String>,
        /// Minimum command frequency to suggest (default: 2)
        #[arg(long, default_value = "2")]
        min_count: usize,
    },

    /// Claude Code hook integration
    Hook {
        /// Subcommand: "install", "preview", or "claude" (run as hook)
        mode: String,
        /// Target (e.g., "claude")
        #[arg(long)]
        shell: Option<String>,
        /// Path to Claude settings.json
        #[arg(long)]
        settings_file: Option<PathBuf>,
        /// Path to canonical aliasman data file
        #[arg(long)]
        data_file: Option<PathBuf>,
        /// Token budget for hook output (default: 500)
        #[arg(long)]
        max_tokens: Option<usize>,
    },

    /// Search aliases semantically using natural language
    Search {
        /// The search query (use "reindex" to rebuild the index)
        query: String,
        /// Maximum number of results (default: 5)
        #[arg(long, default_value = "5")]
        limit: usize,
        /// Path to canonical aliasman data file
        #[arg(long)]
        data_file: Option<PathBuf>,
    },

    /// Run the MCP (Model Context Protocol) server for alias search
    Mcp {
        /// Subcommand: "serve"
        mode: String,
        /// Path to canonical aliasman data file
        #[arg(long)]
        data_file: Option<PathBuf>,
     },

     /// Manage alias packs — create, add, export
    Pack {
         /// Pack subcommand
        #[command(subcommand)]
        subcmd: PackSubcommand,
         /// Path to canonical aliasman data file
         #[arg(long, global = true)]
        data_file: Option<PathBuf>,
         /// Path to the managed aliases output file
         #[arg(long, global = true)]
        aliases_file: Option<PathBuf>,
    }
}

#[derive(clap::Parser, Debug, Clone)]
pub enum PackSubcommand {
     /// Create a new alias pack
    Create {
         /// Pack name (alphanumeric, hyphens, underscores)
        name: String,
         /// Pack version (semver, default: 0.1.0)
         #[arg(long, default_value = "0.1.0")]
        version: String,
         /// Pack description
         #[arg(long)]
        description: Option<String>,
         /// Author name
         #[arg(long)]
        author: Option<String>,
    },

     /// Add an alias to an existing pack
    Add {
         /// Pack name
        pack: String,
         /// Alias name
         #[arg(long)]
        name: String,
         /// Alias command
         #[arg(long)]
        command: String,
         /// Optional description
         #[arg(long)]
        description: Option<String>,
         /// Optional tags (can be repeated)
         #[arg(long)]
        tag: Vec<String>,
    },

    /// List all installed packs
    List,

     /// Export a pack as a single shareable TOML file
    Export {
         /// Pack name to export
        pack: String,
         /// Output file path (default: /tmp/{pack}-pack.toml)
         #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Install a pack from a file or URL with safety scanning
    Install {
        /// Path to the pack export TOML file (mutually exclusive with --url)
        file: Option<PathBuf>,
        /// URL to download the pack from (mutually exclusive with --file)
        #[arg(long)]
        url: Option<String>,
        /// Force install despite safety warnings or collisions
        #[arg(long)]
        force: bool,
    },

     /// Remove an installed pack and its aliases
    Remove {
         /// Pack name to remove
        name: String,
     },

      /// Install builtin packs (k8s, docker)
    InstallBuiltin,
}


fn default_home() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
}

fn default_data_file(home: &PathBuf) -> PathBuf {
    home.join(".config").join("aliasman").join("aliases.toml")
}

fn print_reload_hint() {
    println!();
    println!("To activate, run:");
    println!("    source $HOME/.aliases");
    println!("\nOr open a new terminal.");
}

fn load_store(data_file: &Path) -> Result<AliasStore, Box<dyn std::error::Error>> {
    if data_file.exists() {
        let toml_content = fs::read_to_string(data_file)?;
        Ok(AliasStore::from_toml(&toml_content).unwrap_or_default())
     } else {
        Ok(AliasStore::default())
     }
}

fn save_store(
    store: &AliasStore,
    data_file: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = data_file.parent() {
        fs::create_dir_all(parent)?;
     }
    let toml_out = store.to_toml()?;
    aliasman::store::write_atomic(data_file, &toml_out)?;
    Ok(())
}

fn regenerate_aliases(
    aliases_file: &Path,
    store: &AliasStore,
) -> Result<(), Box<dyn std::error::Error>> {
     // Merge pack aliases at render time
    let mut merged = store.aliases.clone();
    let registry = aliasman::pack_registry::PackRegistry::load().unwrap_or_default();
    for entry in registry.list_packs() {
        let pack_dir = aliasman::pack_manager::get_pack_dir(&entry.name).unwrap_or_else(|_| {
            PathBuf::from("/dev/null")
          });
        if let Ok(pack_aliases) = aliasman::pack_manager::load_pack_aliases(&entry.name) {
            merged.extend(pack_aliases);
         }
        let _ = pack_dir; // suppress unused warning
      }

      // Render merged aliases
    let mut output = String::new();
    output.push_str("# aliasman managed - do not edit manually\n");
    output.push_str("# Run `aliasman list` to view aliases.\n");

    let mut sorted = merged;
    sorted.sort_by(|a, b| a.name.cmp(&b.name));

      // Deduplicate: later entries win (pack aliases overwrite user aliases of same name)
    let mut seen = std::collections::HashMap::new();
    for record in sorted {
        seen.insert(record.name.clone(), record);
      }

    let mut final_list: Vec<aliasman::model::AliasRecord> = seen.into_values().collect();
    final_list.sort_by(|a, b| a.name.cmp(&b.name));

    for record in final_list {
        let escaped_command = record.command.replace('\'', "'\\''");
        output.push_str(&format!("alias {}='{}'\n", record.name, escaped_command));
      }

      // Write atomically
    aliasman::store::write_atomic(aliases_file, &output)?;
    Ok(())
}

/// Refresh the semantic search index after a mutation.
/// Runs in a background thread — failure is non-blocking.
fn refresh_index(store: AliasStore) {
    let db_path = default_index_path();
    let db_str = db_path.to_string_lossy().to_string();
    let provider = OllamaEmbeddingProvider::default();

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("Warning: could not start runtime for index refresh: {}", e);
                return;
                }
            };
        if let Err(e) = rt.block_on(reindex_aliases(&db_str, &store, &provider)) {
            eprintln!("Warning: index refresh failed: {}", e);
            }
        });
}

fn run_init(
    home: &Path,
    shell_env: &str,
    force_shell: Option<&str>,
    data_file: &Path,
    aliases_file: &Path,
    apply: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let detect_result = if let Some(shell_name) = force_shell {
        let kind = match shell_name {
            "zsh" => ShellKind::Zsh,
            "bash" => ShellKind::Bash,
            _ => {
                eprintln!("Error: unsupported shell '{}'. Use 'zsh' or 'bash'.", shell_name);
                std::process::exit(1);
            }
        };
        let config = match kind {
            ShellKind::Zsh => home.join(".zshrc"),
            ShellKind::Bash => {
                if home.join(".bash_profile").exists() {
                    home.join(".bash_profile")
                 } else {
                    home.join(".bashrc")
                 }
            }
        };
        DetectResult::Found(kind, config)
     } else {
        detect_shell_and_config(home, shell_env)
     };

    let (kind, config_path) = match detect_result {
        DetectResult::Found(k, p) => (k, p),
        DetectResult::Ambiguous => {
            eprintln!("Error: Could not detect your shell. Use --shell zsh or --shell bash to specify.");
            std::process::exit(1);
         }
     };

    let config_content = if config_path.exists() {
        fs::read_to_string(&config_path)?
     } else {
        String::new()
     };

    let parsed_aliases = parse_alias_lines(&config_content);
    let mut store = load_store(data_file)?;
    let alias_shell = shell_kind_to_alias_shell(kind);
    let (_, skipped) = merge_imported_aliases(&store, parsed_aliases.clone());
    let needs_block = !has_managed_block(&config_content);

    let new_count = std::cmp::max(
        parsed_aliases.len() as i64 - skipped.len() as i64 - store.aliases.len() as i64,
        0,
     );

    let shell_label = match kind {
        ShellKind::Zsh => "zsh",
        ShellKind::Bash => "bash",
     };

    println!("═══ aliasman init preview ═══");
    println!("Shell: {}", shell_label);
    println!("Config: {}", config_path.display());
    println!("Data file: {}", data_file.display());
    println!("Aliases to import: {}", new_count);

    if !skipped.is_empty() {
        println!("\nSkipped aliases:");
        for s in &skipped {
            println!("     - {} ({})", s.name, s.reason);
         }
     }

    if needs_block {
        println!("\nManaged source block will be added to: {}", config_path.display());
     }

    println!("\nReload after init:");
    println!("     {}", get_reload_hint(&config_path).trim());

    if !apply {
        println!("\nRun with --apply to apply these changes.");
        return Ok(());
     }

    let (new_records, _) = build_imported_records(&store, parsed_aliases, alias_shell);
    for record in new_records {
        store.aliases.push(record.clone());
     }

    if config_path.exists() {
        let bp = backup_file(&config_path)?;
        println!("\nBackup created: {}", bp.display());
        prune_backups(&config_path, 3)?;
     }

    let new_config = ensure_managed_block(&config_content);
    aliasman::store::write_atomic(&config_path, &new_config)?;
    save_store(&store, data_file)?;
    regenerate_aliases(aliases_file, &store)?;
    refresh_index(store.clone());

    // Install builtin packs
    install_builtin_packs(data_file, aliases_file)?;

    println!("\n═══ init complete ═══");
    println!("Data file written: {}", data_file.display());
    println!("Managed aliases written: {}", aliases_file.display());
    print_reload_hint();

    Ok(())
}

fn install_builtin_packs(
    data_file: &Path,
    aliases_file: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
     // Find builtin packs directory (same dir as binary)
    let self_bin = std::env::current_exe().map_err(|e| {
        format!("Failed to get exe path: {}", e)
     })?;
    let bin_dir = self_bin.parent().ok_or("Cannot determine binary directory")?;

     // Look for builtin_packs in common locations
    let builtin_dirs = vec![
        bin_dir.join("builtin_packs"),
        bin_dir.join("..").join("share").join("aliasman").join("builtin_packs"),
        PathBuf::from("/usr/local/share/aliasman/builtin_packs"),
    ];

    let builtin_dir = builtin_dirs.into_iter().find(|d| d.exists());
    let Some(bd) = builtin_dir else {
        return Ok(()); // No builtin packs available, that's fine
    };

     // Ensure config builtin_packs directory exists
    if let Some(home) = dirs::home_dir() {
        let config_builtin_dir = home.join(".config").join("aliasman").join("builtin_packs");
        fs::create_dir_all(&config_builtin_dir)?;

        let mut installed_count = 0usize;
        for entry in fs::read_dir(&bd)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension() == Some(std::ffi::OsStr::new("toml")) {
                let file_name = path.file_name().unwrap_or_default();
                let dest = config_builtin_dir.join(file_name);

                 // Only install if not already installed
                if !dest.exists() {
                    fs::copy(&path, &dest)?;

                     // Install via pack installer
                    if let Ok(export) = aliasman::pack_installer::parse_pack_file(&dest) {
                        let store = load_store(data_file)?;
                        let source = format!("builtin: {}", path.display());

                         // Skip if pack already registered
                        let registry = aliasman::pack_registry::PackRegistry::load().unwrap_or_default();

                        if registry.get_pack(&export.manifest.name).is_some() {
                            continue;
                             }

                         // Install without safety check (builtin packs are trusted)
                        if let Ok(result) = aliasman::pack_installer::install_pack(
                            export,
                            true, // force
                            &store,
                            source,
                        ) {
                            installed_count += 1;
                            println!("  Builtin pack '{}' installed ({} aliases)", result.pack_name, result.installed_count);
                        }
                    }
                }
            }
        }

        if installed_count > 0 {
             // Regenerate aliases after builtin pack installation
            let store = load_store(data_file)?;
            regenerate_aliases(aliases_file, &store)?;
            refresh_index(store);
        }
    }

    Ok(())
}

fn run_add(
    name: &str,
    command: &str,
    description: Option<String>,
    tags: Vec<String>,
    force: bool,
    data_file: &Path,
    aliases_file: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    validate_alias_name_for_write(name, force).map_err(|e| {
        if aliasman::validation::is_protected_name(name) {
            format!("Protected command name '{}'. Use --force to shadow it.", name)
         } else {
            format!("Invalid alias name '{}': {}", name, e)
         }
     })?;

    let mut store = load_store(data_file)?;
    store_add_alias(&mut store, name.to_string(), command.to_string(), description, tags, AliasShell::All)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    save_store(&store, data_file)?;
    regenerate_aliases(aliases_file, &store)?;
    refresh_index(store.clone());

    println!("Added alias: {} = '{}'", name, command);
    print_reload_hint();
    Ok(())
}

fn run_update(
    name: &str,
    command: Option<String>,
    description: Option<Option<String>>,
    tags: Option<Vec<String>>,
    force: bool,
    data_file: &Path,
    aliases_file: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if aliasman::validation::is_protected_name(name) && !force {
        eprintln!("Protected command name '{}'. Use --force to update it.", name);
        std::process::exit(1);
     }

    let mut store = load_store(data_file)?;
    store_update_alias(&mut store, name, command, description, tags)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    save_store(&store, data_file)?;
    regenerate_aliases(aliases_file, &store)?;
    refresh_index(store.clone());

    println!("Updated alias: {}", name);
    print_reload_hint();
    Ok(())
}

fn run_delete(
    name: &str,
    data_file: &Path,
    aliases_file: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut store = load_store(data_file)?;
    store_delete_alias(&mut store, name)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    save_store(&store, data_file)?;
    regenerate_aliases(aliases_file, &store)?;
    refresh_index(store.clone());

    println!("Deleted alias: {}", name);
    print_reload_hint();
    Ok(())
}

fn run_list(
    shell_filter: Option<String>,
    data_file: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let store = load_store(data_file)?;

    let shell_filter_parsed = shell_filter.as_deref().map(|s| match s {
        "zsh" => AliasShell::Zsh,
        "bash" => AliasShell::Bash,
        _ => AliasShell::All,
    });

    let aliases = store_list_aliases(&store, shell_filter_parsed);

    if aliases.is_empty() {
        println!("No aliases found.");
        return Ok(());
    }

     // Group by source category
    // Categorize by source
    let mut user_aliases: Vec<AliasRecord> = Vec::new();
    let mut imported_aliases: Vec<AliasRecord> = Vec::new();
    let mut suggested_aliases: Vec<AliasRecord> = Vec::new();
    let mut pack_map: std::collections::HashMap<String, Vec<AliasRecord>> = std::collections::HashMap::new();

    for record in aliases {
        let pack_name = match &record.source {
            AliasSource::Pack(p) => Some(p.clone()),
            _ => None,
         };

        match record.source {
            AliasSource::User => user_aliases.push(record.clone()),
            AliasSource::Imported => imported_aliases.push(record.clone()),
            AliasSource::Suggested => suggested_aliases.push(record.clone()),
            AliasSource::Pack(_) => {
                if let Some(p) = pack_name {
                    pack_map.entry(p).or_default().push(record.clone());
                 }
             },
         }
     }

    let mut first = true;

    if !user_aliases.is_empty() {
        if !first { println!(); }
        println!("── user ({}) ──", user_aliases.len());
        print_alias_table(&user_aliases);
        first = false;
     }

    if !imported_aliases.is_empty() {
        if !first { println!(); }
        println!("── imported ({}) ──", imported_aliases.len());
        print_alias_table(&imported_aliases);
        first = false;
     }

    if !suggested_aliases.is_empty() {
        if !first { println!(); }
        println!("── suggested ({}) ──", suggested_aliases.len());
        print_alias_table(&suggested_aliases);
        first = false;
     }

    for (pack_name, pack_list) in &pack_map {
        if !first { println!(); }
        println!("── pack: {} ({}) ──", pack_name, pack_list.len());
        print_alias_table(pack_list);
        first = false;
     }

    Ok(())
}

fn print_alias_table(aliases: &[AliasRecord]) {
    println!("{:<20} {:<40} {:<10}", "Name", "Command", "Source");
    println!("{:-<20} {:-<40} {:-<10}", "", "", "");

    for record in aliases {
        let source = match record.source {
            AliasSource::User => "user",
            AliasSource::Imported => "imported",
            AliasSource::Suggested => "suggested",
            AliasSource::Pack(ref p) => p.as_str(),
         };
        println!(
             "{:<20} {:<40} {:<10}",
            record.name,
            truncate(&record.command, 40),
            source
         );
     }
}
fn run_stats(
    history_file: Option<PathBuf>,
    verbose: bool,
    top: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let hist_path =
        history_file.or(detect_history_file()).ok_or("No history file found. Set $HISTFILE or use --history-file")?;

    let commands = read_history_file(&hist_path)?;
    let frequencies = command_frequencies(&commands);
    let top_freq: Vec<_> = frequencies.into_iter().take(top).collect();

    if verbose {
        println!("{}", format_verbose_stats(&top_freq));
     } else {
        println!("{}", format_stats(&top_freq));
     }

    Ok(())
}

fn run_suggest(
    history_file: Option<PathBuf>,
    data_file: &Path,
    aliases_file: &Path,
    apply: Option<String>,
    min_count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let hist_path =
        history_file.or(detect_history_file()).ok_or("No history file found. Set $HISTFILE or use --history-file")?;

    let commands = read_history_file(&hist_path)?;
    let frequencies = command_frequencies(&commands);

    let store = load_store(data_file)?;
    let existing: Vec<String> = store.aliases.iter().map(|a| a.name.clone()).collect();

    let suggestions = generate_suggestions(&frequencies, 10, min_count, &existing);

    if suggestions.is_empty() {
        println!("No suggestions found. Try --min-count 1 for more results.");
        return Ok(());
     }

    if let Some(alias_name) = &apply {
        let s = suggestions.iter().find(|s| s.alias_name == *alias_name);
        match s {
            Some(sugg) => {
                if sugg.is_risky {
                    eprintln!(
                        "Error: '{}' is a risky command (Review carefully). Risky suggestions cannot be auto-applied.",
                        alias_name
                     );
                    std::process::exit(1);
                 }

                validate_alias_name_for_write(&sugg.alias_name, false).map_err(|e| {
                    format!("Invalid alias name: {}", e)
                 })?;

                let mut store = load_store(data_file)?;
                store_add_alias(
                    &mut store,
                    sugg.alias_name.clone(),
                    sugg.command.clone(),
                    Some(format!("Suggested from history (used {} times)", sugg.count)),
                    vec!["suggested".to_string()],
                    AliasShell::All,
                 )
                 .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

                save_store(&store, data_file)?;
                regenerate_aliases(aliases_file, &store)?;
                refresh_index(store.clone());

                println!("Applied suggestion: {} = '{}'", sugg.alias_name, sugg.command);
                print_reload_hint();
             }
            None => {
                eprintln!("Error: No suggestion found for alias '{}'", alias_name);
                std::process::exit(1);
             }
         }
        return Ok(());
     }

    println!("═══ alias suggestions ═══");
    println!("{:<15} {:<5} {:<6} {:<}", "Alias", "Count", "Risky", "Command");
    println!("{:-<15} {:-<5} {:-<6} {:-<}", "", "", "", "");

    for s in &suggestions {
        let risky_flag = if s.is_risky {
            "* Review carefully"
         } else {
            ""
         };
        println!(
            "{:<15} {:>5}     {:<18} {}",
            s.alias_name, s.count, risky_flag, s.command
         );
     }

    println!("\nRun `aliasman suggest --apply <alias>` to add a suggestion.");
    println!("Suggestions marked 'Review carefully' contain shell metacharacters and cannot be auto-applied.");

    Ok(())
}

fn run_hook(
    mode: &str,
    _shell: Option<&str>,
    settings_file: Option<&Path>,
    data_file: &Path,
    max_tokens: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    match mode {
        "install" => {
            let home = default_home();
            let default_sf = home.join(".claude").join("settings.json");
            let sf = settings_file.unwrap_or_else(|| &default_sf);

            let self_bin = std::env::args()
                .next()
                .unwrap_or_else(|| "aliasman".to_string());

            let preview = create_install_preview(sf, &PathBuf::from(&self_bin))?;

            if preview.has_existing_settings {
                println!("Settings file: {}", sf.display());
                if let Some(bp) = &preview.backup_path {
                    println!("Backup will be created: {}", bp.display());
                 }
                println!("Hook command: {}", preview.hook_command);
                if preview.has_existing_hooks {
                    println!("Note: only SessionStart hook will be added/updated by aliasman. Other hooks are preserved.");
                 } else {
                    println!("No existing hooks detected.");
                 }
             } else {
                println!("Will create new settings file: {}", sf.display());
                println!("Hook command: {}", preview.hook_command);
             }

            // In non-interactive mode, just install
            install_claude_hook(sf, &PathBuf::from(&self_bin))?;
            println!("\nHook installed successfully.");
         }

        "preview" => {
            let home = default_home();
            let default_sf = home.join(".claude").join("settings.json");
            let sf = settings_file.unwrap_or_else(|| &default_sf);

            let self_bin = std::env::args()
                .next()
                .unwrap_or_else(|| "aliasman".to_string());

            let preview = create_install_preview(sf, &PathBuf::from(&self_bin))?;
            println!("═══ hook install preview ═══");
            println!("Settings: {}", sf.display());
            println!("Hook: {}", preview.hook_command);
            println!("Existing hooks: {}", preview.has_existing_hooks);
            println!("Existing settings: {}", preview.has_existing_settings);
         }

        "claude" => {
            // Run as Claude hook
            let store = load_store(data_file)?;
            let cwd = std::env::current_dir()?;

            match run_claude_hook(&store, &cwd, max_tokens) {
                Ok(Some(output)) => {
                    let json = serde_json::to_string(&output)?;
                    println!("{}", json);
                 }
                Ok(None) => {
                    // No context to inject — silent no-op
                 }
                Err(e) => {
                    // Recoverable errors are silent by default
                    eprintln!("Debug: hook error: {}", e);
                 }
             }
         }

        other => {
            eprintln!("Unknown hook mode '{}'. Use 'install', 'preview', or 'claude'.", other);
            std::process::exit(1);
         }
     }

    Ok(())
}

fn run_search(
    query: &str,
    limit: usize,
    data_file: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
     // Handle "reindex" command
    if query == "reindex" {
        let store = load_store(data_file)?;
        let db_path = default_index_path();
        let db_str = db_path.to_string_lossy().to_string();
        let provider = OllamaEmbeddingProvider::default();

        let rt = tokio::runtime::Runtime::new()?;
        let meta = rt.block_on(reindex_aliases(&db_str, &store, &provider))
            .map_err(|e| format!("Reindex failed: {}", e))?;

        println!("═══ index rebuilt ═══");
        println!("Provider: {}", meta.embedding_provider);
        println!("Model: {}", meta.embedding_model);
        println!("Dimensions: {}", meta.vector_dimensions);
        println!("Aliases indexed: {}", meta.alias_count);
        return Ok(());
     }

    let store = load_store(data_file)?;
    let db_path = default_index_path();
    let db_str = db_path.to_string_lossy().to_string();
    let provider = OllamaEmbeddingProvider::default();

    let rt = tokio::runtime::Runtime::new()?;
    let mut results: Vec<SearchResult> = rt.block_on(
        search_aliases(&db_str, query, &provider, limit)
    )?;

    let used_fallback = if results.is_empty() {
        results = lexical_search(&store, query, limit);
        true
     } else {
        false
     };

    if used_fallback {
        eprintln!("Warning: Semantic search unavailable (Ollama not running or index empty). Using lexical fallback.");
    }

    if results.is_empty() {
        println!("No aliases found for: {}", query);
        return Ok(());
     }

    println!("{:<15} {:<40} {:<8} {}", "Alias", "Command", "Score", "Reason");
    println!("{:-<15} {:-<40} {:-<8} {:-<}", "", "", "", "");

    for r in &results {
        println!(
            "{:<15} {:<40} {:<8.2} {}",
            r.alias_name,
            truncate(&r.command, 40),
            r.score,
            r.reason
         );
     }

    Ok(())
}

fn run_mcp(
    mode: &str,
    data_file: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    match mode {
        "serve" => {
            let df = data_file.to_path_buf();
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                run_mcp_server(df).await;
             });
         }
        other => {
            eprintln!("Unknown MCP mode '{}'. Use 'serve'.", other);
            std::process::exit(1);
         }
     }
    Ok(())
}

fn run_pack(
    subcmd: PackSubcommand,
    data_file: &Path,
    aliases_file: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    match subcmd {
        PackSubcommand::Create {
            name,
            version,
            description,
            author,
        } => {
            let pack_dir = create_pack(name.clone(), version, description, author)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            println!("Created pack '{}' in {}", name, pack_dir.display());
            println!("Add aliases with: aliasman pack add {} --name <alias> --command <cmd>", name);
        }
        PackSubcommand::Add {
            pack,
            name,
            command,
            description,
            tag,
        } => {
            if !pack_exists(&pack) {
                eprintln!("Error: Pack '{}' does not exist. Create it first with: aliasman pack create {}", pack, pack);
                std::process::exit(1);
            }
            add_alias_to_pack(
                 &pack, name.clone(), command.clone(), description.clone(), tag.clone(), AliasShell::All,
            ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            // Also add to main store and regenerate
            let mut store = load_store(data_file)?;
            aliasman::store::store_add_alias(
                 &mut store, name.clone(), command.clone(), description, tag, AliasShell::All,
            ).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            save_store(&store, data_file)?;
            regenerate_aliases(aliases_file, &store)?;
            refresh_index(store.clone());
            println!("Added alias '{}' to pack '{}'", name, pack);
            print_reload_hint();
        }
        PackSubcommand::List => {
            let registry = PackRegistry::load().map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, e)
             })?;
            let packs = registry.list_packs();
            if packs.is_empty() {
                println!("No installed packs. Use: aliasman pack install <file>");
                 return Ok(());
             }
            println!("{:<20} {:<10} {:<15} {:<10}", "Name", "Version", "Installed", "Aliases");
            println!("{:-<20} {:<10} {:<15} {:<10}", "", "", "", "");
            for p in packs {
                let date = match chrono::Local.timestamp_opt(p.install_time as i64, 0) {
                        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d").to_string(),
                        _ => "unknown".to_string(),
                    };
                println!("{:<20} {:<10} {:<15} {:<10}", p.name, p.version, date, p.alias_count);
                    }
        }
        PackSubcommand::Export { pack, output } => {
            if !pack_exists(&pack) {
                eprintln!("Error: Pack '{}' does not exist.", pack);
                std::process::exit(1);
            }
            let out = output.unwrap_or_else(|| {
                std::path::PathBuf::from("/tmp").join(format!("{}-pack.toml", pack))
            });
            export_pack_to_file(&pack, &out)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            println!("Exported pack '{}' to {}", pack, out.display());
        }
        PackSubcommand::Install { file, url, force } => {
            // Validate: exactly one of --file or --url must be provided
            let (export, source) = if let Some(ref fpath) = file {
                let export = parse_pack_file(fpath)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                (export, format!("file: {}", fpath.display()))
            } else if let Some(ref u) = url {
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                let content = rt.block_on(download_pack(u))
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                let export: aliasman::pack_manager::PackExport =
                    toml::from_str(&content)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                (export, format!("url: {}", u))
            } else {
                eprintln!("Error: Provide either a file path or --url");
                eprintln!("Usage: aliasman pack install <file.toml>");
                eprintln!("       aliasman pack install --url https://example.com/pack.toml");
                std::process::exit(1);
            };

            let store = load_store(data_file)?;
            let preview = create_pack_install_preview(&export, &store, source.clone());
            preview.display();

            // In non-interactive mode (or if no warnings/collisions), proceed
            if !preview.warnings.is_empty() && !force {
                eprintln!("\nInstall blocked due to safety warnings. Use --force to override.");
                std::process::exit(1);
            }

            let result = install_pack(export, force, &store, source)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            // Post-install: regenerate aliases and refresh index
            regenerate_aliases(aliases_file, &load_store(data_file)?)?;
            refresh_index(load_store(data_file)?);

            println!("\nPack '{}' installed successfully!", result.pack_name);
            println!("  Aliases installed: {}", result.installed_count);
            if !result.skipped_collisions.is_empty() {
                println!("  Skipped collisions: {}", result.skipped_collisions.len());
            }
            print_reload_hint();
         }
        PackSubcommand::Remove { name } => {
             // Check if pack is registered
            let mut registry = PackRegistry::load()
                 .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

            if registry.get_pack(&name).is_none() {
                eprintln!("Error: Pack '{}' is not installed.", name);
                std::process::exit(1);
             }

             // Load store and remove pack aliases (preserve modified_by_user)
            let mut store = load_store(data_file)?;
            let mut removed_count = 0usize;
            let mut preserved_count = 0usize;

            let mut new_aliases = Vec::new();
            for mut a in store.aliases {
                if matches!(&a.source, AliasSource::Pack(p) if p == &name) {
                    if a.modified_by_user {
                        a.source = AliasSource::User;
                        preserved_count += 1;
                        new_aliases.push(a);
                     } else {
                        removed_count += 1;
                     }
                 } else {
                    new_aliases.push(a);
                 }
              }
            store.aliases = new_aliases;

             // Save updated store
            save_store(&store, data_file)?;

             // Remove pack directory
            let pack_dir = aliasman::pack_manager::get_pack_dir(&name)
                 .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            if pack_dir.exists() {
                fs::remove_dir_all(&pack_dir)?;
             }

             // Unregister from registry
            registry.unregister_pack(&name);
            registry.save()?;

             // Post-remove: regenerate aliases and refresh index
            regenerate_aliases(aliases_file, &store)?;
            refresh_index(store);

            println!("Pack '{}' removed successfully!", name);
            println!("  Aliases removed: {}", removed_count);
            if preserved_count > 0 {
                println!("  User-modified aliases preserved: {}", preserved_count);
             }
            print_reload_hint();
          }
        PackSubcommand::InstallBuiltin => {
            install_builtin_packs(data_file, aliases_file)?;
            println!("Builtin packs installation complete.");
          }
    }
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
     } else {
        format!("{}...", &s[..max - 3])
     }
}

fn main() {
    let cli = Cli::parse();

    let result: Result<(), Box<dyn std::error::Error>> = match cli {
        Cli::Init {
            shell,
            data_file,
            aliases_file,
            home,
            shell_env,
            apply,
         } => {
            let home_dir = home.unwrap_or_else(default_home);
            let shell_env_str =
                shell_env.unwrap_or_else(|| std::env::var("SHELL").unwrap_or_default());
            let df = data_file.unwrap_or_else(|| default_data_file(&home_dir));
            let af = aliases_file.unwrap_or_else(|| home_dir.join(".aliases"));

            run_init(&home_dir, &shell_env_str, shell.as_deref(), &df, &af, apply)
         }

        Cli::Add {
            name,
            command,
            description,
            tag,
            force,
            data_file,
            aliases_file,
         } => {
            let home_dir = default_home();
            let df = data_file.unwrap_or_else(|| default_data_file(&home_dir));
            let af = aliases_file.unwrap_or_else(|| home_dir.join(".aliases"));

            run_add(&name, &command, description, tag, force, &df, &af)
         }

        Cli::Update {
            name,
            command,
            description,
            tag,
            force,
            data_file,
            aliases_file,
         } => {
            let home_dir = default_home();
            let df = data_file.unwrap_or_else(|| default_data_file(&home_dir));
            let af = aliases_file.unwrap_or_else(|| home_dir.join(".aliases"));

            run_update(&name, command, description, tag, force, &df, &af)
         }

        Cli::Delete {
            name,
            data_file,
            aliases_file,
         } => {
            let home_dir = default_home();
            let df = data_file.unwrap_or_else(|| default_data_file(&home_dir));
            let af = aliases_file.unwrap_or_else(|| home_dir.join(".aliases"));

            run_delete(&name, &df, &af)
         }

        Cli::List { shell, data_file } => {
            let home_dir = default_home();
            let df = data_file.unwrap_or_else(|| default_data_file(&home_dir));

            run_list(shell, &df)
         }

        Cli::Stats {
            history_file,
            verbose,
            top,
         } => run_stats(history_file, verbose, top),

        Cli::Suggest {
            history_file,
            data_file,
            aliases_file,
            apply,
            min_count,
         } => {
            let home_dir = default_home();
            let df = data_file.unwrap_or_else(|| default_data_file(&home_dir));
            let af = aliases_file.unwrap_or_else(|| home_dir.join(".aliases"));

            run_suggest(history_file, &df, &af, apply, min_count)
         }

        Cli::Hook {
            mode,
            shell,
            settings_file,
            data_file,
            max_tokens,
         } => {
            let home_dir = default_home();
            let df = data_file.unwrap_or_else(|| default_data_file(&home_dir));
            let sf = settings_file.as_ref().map(|p| p.as_path());

            run_hook(
                &mode,
                shell.as_deref(),
                sf,
                &df,
                max_tokens,
             )
         }

        Cli::Search {
            query,
            limit,
            data_file,
         } => {
            let home_dir = default_home();
            let df = data_file.unwrap_or_else(|| default_data_file(&home_dir));

            run_search(&query, limit, &df)
         }

        Cli::Mcp {
            mode,
            data_file,
         } => {
            let home_dir = default_home();
            let df = data_file.unwrap_or_else(|| default_data_file(&home_dir));

            run_mcp(&mode, &df)
         }
        Cli::Pack {
            subcmd,
            data_file,
            aliases_file,
           } => {
            let home_dir = default_home();
            let df = data_file.unwrap_or_else(|| default_data_file(&home_dir));
            let af = aliases_file.unwrap_or_else(|| home_dir.join(".aliases"));

            run_pack(subcmd, &df, &af)
           }
     };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
     }
}
