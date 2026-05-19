# Phase 7: Pack Foundation

**Milestone:** v0.1 Alias Library
**Status:** In progress
**Started:** 2026-05-17
**Depends on:** Phase 6

## Goal
Users can create, populate, and export alias packs as shareable TOML files.

## Requirements
PACK-01, PACK-02, PACK-03, PACK-04, MGMT-04

## Success Criteria
1. `aliasman pack create mypack` creates a structured directory with valid `pack.toml` manifest
2. `aliasman pack add mypack kget "kubectl get pods"` persists alias inside pack directory
3. `aliasman pack export mypack` produces a single shareable `.toml` file
4. Every alias in pack tracked with `AliasSource::Pack("mypack")`

## Key Decisions
- Single TOML file format (confirmed by user)
- Flat names + collision detection (from research)
- Packs live in `~/.config/aliasman/packs/`

## Scope
- `pack_manifest.rs` — PackManifest struct with name, version, description, author, format_version
- `model.rs` — Add `AliasSource::Pack(String)` variant
- `pack_manager.rs` — create_pack(), add_alias_to_pack(), export_pack()
- `main.rs` — Pack subcommand with create/add/export subcommands
- `lib.rs` — Declare new modules
