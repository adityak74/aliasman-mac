use std::fs;
use std::path::PathBuf;

use assert_fs::prelude::*;
use assert_fs::TempDir;

mod helpers {
    pub fn bin_path() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_BIN_EXE_aliasman"))
    }
}

fn tmp_home() -> TempDir {
    TempDir::new().unwrap()
}

fn data_file(home: &TempDir) -> PathBuf {
    home.child(".config").child("aliasman").child("aliases.toml").to_path_buf()
}

fn aliases_file(home: &TempDir) -> PathBuf {
    home.child(".aliases").to_path_buf()
}

#[test]
fn adds_alias_with_named_flags() {
    let home = tmp_home();
    let df = data_file(&home);
    let af = aliases_file(&home);

    let output = assert_cmd::Command::cargo_bin("aliasman")
         .unwrap()
         .arg("add")
         .arg("--name")
         .arg("gs")
         .arg("--command")
         .arg("git status")
         .arg("--data-file")
         .arg(&df)
         .arg("--aliases-file")
         .arg(&af)
         .output()
         .unwrap();

    assert!(output.status.success(), "add should succeed: {:?}", String::from_utf8(output.stderr.clone()));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Added alias:"));
    assert!(stdout.contains("gs"));

      // Verify aliases file was created
    assert!(af.exists(), "managed aliases file should exist");
    let content = fs::read_to_string(&af).unwrap();
    assert!(content.contains("alias gs='git status'"));
}

#[test]
fn duplicate_add_fails() {
    let home = tmp_home();
    let df = data_file(&home);
    let af = aliases_file(&home);

      // Add first alias
    assert_cmd::Command::cargo_bin("aliasman")
         .unwrap()
         .arg("add")
         .arg("--name")
         .arg("gs")
         .arg("--command")
         .arg("git status")
         .arg("--data-file")
         .arg(&df)
         .arg("--aliases-file")
         .arg(&af)
         .assert()
         .success();

      // Try duplicate
    let output = assert_cmd::Command::cargo_bin("aliasman")
         .unwrap()
         .arg("add")
         .arg("--name")
         .arg("gs")
         .arg("--command")
         .arg("git log")
         .arg("--data-file")
         .arg(&df)
         .arg("--aliases-file")
         .arg(&af)
         .output()
         .unwrap();

    assert!(!output.status.success(), "duplicate add should fail");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("already exists"), "error should mention 'already exists': {}", stderr);
    assert!(stderr.contains("aliasman update --name"), "error should suggest 'update': {}", stderr);
}

#[test]
fn updates_existing_alias() {
    let home = tmp_home();
    let df = data_file(&home);
    let af = aliases_file(&home);

      // Add first
    assert_cmd::Command::cargo_bin("aliasman")
         .unwrap()
         .arg("add")
         .arg("--name")
         .arg("gs")
         .arg("--command")
         .arg("git status")
         .arg("--data-file")
         .arg(&df)
         .arg("--aliases-file")
         .arg(&af)
         .assert()
         .success();

      // Update
    let output = assert_cmd::Command::cargo_bin("aliasman")
         .unwrap()
         .arg("update")
         .arg("--name")
         .arg("gs")
         .arg("--command")
         .arg("git status -s")
         .arg("--data-file")
         .arg(&df)
         .arg("--aliases-file")
         .arg(&af)
         .output()
         .unwrap();

    assert!(output.status.success(), "update should succeed");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Updated alias:"));

      // Verify aliases file
    let content = fs::read_to_string(&af).unwrap();
    assert!(content.contains("alias gs='git status -s'"));
}

#[test]
fn deletes_existing_alias() {
    let home = tmp_home();
    let df = data_file(&home);
    let af = aliases_file(&home);

      // Add first
    assert_cmd::Command::cargo_bin("aliasman")
         .unwrap()
         .arg("add")
         .arg("--name")
         .arg("gs")
         .arg("--command")
         .arg("git status")
         .arg("--data-file")
         .arg(&df)
         .arg("--aliases-file")
         .arg(&af)
         .assert()
         .success();

      // Delete
    let output = assert_cmd::Command::cargo_bin("aliasman")
         .unwrap()
         .arg("delete")
         .arg("--name")
         .arg("gs")
         .arg("--data-file")
         .arg(&df)
         .arg("--aliases-file")
         .arg(&af)
         .output()
         .unwrap();

    assert!(output.status.success(), "delete should succeed");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Deleted alias:"));

      // Verify alias removed from file
    let content = fs::read_to_string(&af).unwrap();
    assert!(!content.contains("alias gs="));
}

#[test]
fn lists_aliases_in_table() {
    let home = tmp_home();
    let df = data_file(&home);
    let af = aliases_file(&home);

      // Add two aliases
    assert_cmd::Command::cargo_bin("aliasman")
         .unwrap()
         .arg("add")
         .arg("--name")
         .arg("gs")
         .arg("--command")
         .arg("git status")
         .arg("--data-file")
         .arg(&df)
         .arg("--aliases-file")
         .arg(&af)
         .assert()
         .success();

    assert_cmd::Command::cargo_bin("aliasman")
         .unwrap()
         .arg("add")
         .arg("--name")
         .arg("ll")
         .arg("--command")
         .arg("ls -la")
         .arg("--data-file")
         .arg(&df)
         .arg("--aliases-file")
         .arg(&af)
         .assert()
         .success();

      // List
    let output = assert_cmd::Command::cargo_bin("aliasman")
         .unwrap()
         .arg("list")
         .arg("--data-file")
         .arg(&df)
         .output()
         .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Name"));
    assert!(stdout.contains("Command"));
    assert!(stdout.contains("Source"));
    assert!(stdout.contains("gs"));
    assert!(stdout.contains("ll"));
}

#[test]
fn protected_alias_requires_force() {
    let home = tmp_home();
    let df = data_file(&home);
    let af = aliases_file(&home);

    let output = assert_cmd::Command::cargo_bin("aliasman")
         .unwrap()
         .arg("add")
         .arg("--name")
         .arg("rm")
         .arg("--command")
         .arg("rm -i")
         .arg("--data-file")
         .arg(&df)
         .arg("--aliases-file")
         .arg(&af)
         .output()
         .unwrap();

    assert!(!output.status.success(), "protected alias without --force should fail");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("--force") || stderr.contains("protected"), "should mention --force or protected: {}", stderr);
}

#[test]
fn crud_mutations_print_shell_specific_reload_hint() {
    let home = tmp_home();
    let df = data_file(&home);
    let af = aliases_file(&home);

      // Add should print reload hint
    let output = assert_cmd::Command::cargo_bin("aliasman")
         .unwrap()
         .arg("add")
         .arg("--name")
         .arg("gs")
         .arg("--command")
         .arg("git status")
         .arg("--data-file")
         .arg(&df)
         .arg("--aliases-file")
         .arg(&af)
         .output()
         .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("source") && stdout.contains(".aliases"), "add should print reload hint");

      // Update should print reload hint
    let output = assert_cmd::Command::cargo_bin("aliasman")
         .unwrap()
         .arg("update")
         .arg("--name")
         .arg("gs")
         .arg("--command")
         .arg("git status -s")
         .arg("--data-file")
         .arg(&df)
         .arg("--aliases-file")
         .arg(&af)
         .output()
         .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("source") && stdout.contains(".aliases"), "update should print reload hint");

      // Delete should print reload hint
    let output = assert_cmd::Command::cargo_bin("aliasman")
         .unwrap()
         .arg("delete")
         .arg("--name")
         .arg("gs")
         .arg("--data-file")
         .arg(&df)
         .arg("--aliases-file")
         .arg(&af)
         .output()
         .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("source") && stdout.contains(".aliases"), "delete should print reload hint");
}
