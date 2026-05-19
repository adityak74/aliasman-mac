use std::fs;

use aliasman::model::{AliasRecord, AliasShell, AliasSource};
use aliasman::store::{write_managed_aliases, AliasStore};
use tempfile::TempDir;

fn make_record(name: &str, command: &str) -> AliasRecord {
    AliasRecord {
        name: name.to_string(),
        command: command.to_string(),
        description: None,
        tags: vec![],
        shell: AliasShell::All,
        source: AliasSource::User,
        created_at: 0,
        updated_at: 0,
    modified_by_user: false,
    }
}

/// write_managed_aliases writes expected contents to a tempdir target path.
#[test]
fn writes_managed_aliases_to_tempdir_path() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join(".aliases");

    let store = AliasStore {
        aliases: vec![make_record("gs", "git status")],
    };

    write_managed_aliases(&target, &store).unwrap();

    let contents = fs::read_to_string(&target).unwrap();
    assert!(contents.contains("# aliasman managed - do not edit manually"));
    assert!(contents.contains("alias gs='git status'"));
}

/// write_managed_aliases overwrites an existing tempdir target path with new rendered contents.
#[test]
fn overwrites_existing_managed_aliases_file() {
    let dir = TempDir::new().unwrap();
    let target = dir.path().join(".aliases");

    // Write initial content
    let store1 = AliasStore {
        aliases: vec![make_record("old", "old-command")],
    };
    write_managed_aliases(&target, &store1).unwrap();

    // Overwrite with new content
    let store2 = AliasStore {
        aliases: vec![make_record("new", "new-command")],
    };
    write_managed_aliases(&target, &store2).unwrap();

    let contents = fs::read_to_string(&target).unwrap();
    assert!(!contents.contains("old"));
    assert!(contents.contains("alias new='new-command'"));
}
