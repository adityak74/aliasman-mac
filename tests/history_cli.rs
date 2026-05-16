use std::fs;
use std::path::PathBuf;

use assert_fs::prelude::*;
use assert_fs::TempDir;

fn tmp_home() -> TempDir {
    TempDir::new().unwrap()
}

fn data_file(home: &TempDir) -> PathBuf {
    home.child(".config").child("aliasman").child("aliases.toml").to_path_buf()
}

fn aliases_file(home: &TempDir) -> PathBuf {
    home.child(".aliases").to_path_buf()
}

fn history_file(home: &TempDir) -> PathBuf {
    home.child(".test_history").to_path_buf()
}

/// Write a zsh extended history fixture
fn write_zsh_history(home: &TempDir) {
    let content = "\
: 1715300000:0;git status
: 1715300001:0;cargo build --release
: 1715300002:0;git status
: 1715300003:0;cargo build --release
: 1715300004:0;cargo build --release
: 1715300005:0;ls
: 1715300006:0;echo $(pwd) && ls
: 1715300007:0;echo $(pwd) && ls
";
    fs::write(history_file(home), content).unwrap();
}

/// Write a bash history fixture with timestamps
fn write_bash_history(home: &TempDir) {
    let content = "\
#1715300000
git status
#1715300001
cargo build --release
#1715300002
git status
#1715300003
ls
";
    fs::write(history_file(home), content).unwrap();
}

#[test]
fn stats_shows_command_frequency() {
    let home = tmp_home();
    write_zsh_history(&home);

    let output = assert_cmd::Command::cargo_bin("aliasman")
          .unwrap()
          .arg("stats")
          .arg("--history-file")
          .arg(history_file(&home))
          .output()
          .unwrap();

    assert!(output.status.success(), "stats should succeed: {:?}", String::from_utf8(output.stderr.clone()));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Count"));
    assert!(stdout.contains("Command"));
    assert!(stdout.contains("cargo build --release"));
}

#[test]
fn stats_verbose_shows_percentages() {
    let home = tmp_home();
    write_zsh_history(&home);

    let output = assert_cmd::Command::cargo_bin("aliasman")
          .unwrap()
          .arg("stats")
          .arg("--history-file")
          .arg(history_file(&home))
          .arg("--verbose")
          .output()
          .unwrap();

    assert!(output.status.success(), "verbose stats should succeed");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("%"), "verbose output should show percentages: {}", stdout);
    assert!(stdout.contains("Tool"), "verbose output should show tool grouping: {}", stdout);
}

#[test]
fn bash_timestamps_not_counted_as_commands() {
    let home = tmp_home();
    write_bash_history(&home);

    let output = assert_cmd::Command::cargo_bin("aliasman")
          .unwrap()
          .arg("stats")
          .arg("--history-file")
          .arg(history_file(&home))
          .output()
          .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("#1715300000"), "bash timestamps should not appear as commands");
}

#[test]
fn suggest_shows_suggestions() {
    let home = tmp_home();
    write_zsh_history(&home);
    let df = data_file(&home);
    let af = aliases_file(&home);

       // Create empty store
    fs::create_dir_all(df.parent().unwrap()).unwrap();
    fs::write(&df, "[aliases]\n").unwrap();

    let output = assert_cmd::Command::cargo_bin("aliasman")
          .unwrap()
          .arg("suggest")
          .arg("--history-file")
          .arg(history_file(&home))
          .arg("--data-file")
          .arg(&df)
          .arg("--aliases-file")
          .arg(&af)
          .arg("--min-count")
          .arg("2")
          .output()
          .unwrap();

    assert!(output.status.success(), "suggest should succeed: {:?}", String::from_utf8(output.stderr.clone()));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("suggestion"), "should show suggestions: {}", stdout);
}

#[test]
fn suggestions_are_display_only_by_default() {
    let home = tmp_home();
    write_zsh_history(&home);
    let df = data_file(&home);
    let af = aliases_file(&home);

       // Create empty store
    fs::create_dir_all(df.parent().unwrap()).unwrap();
    fs::write(&df, "[aliases]\n").unwrap();

    let output = assert_cmd::Command::cargo_bin("aliasman")
          .unwrap()
          .arg("suggest")
          .arg("--history-file")
          .arg(history_file(&home))
          .arg("--data-file")
          .arg(&df)
          .arg("--aliases-file")
          .arg(&af)
          .arg("--min-count")
          .arg("2")
          .output()
          .unwrap();

    assert!(output.status.success());

       // Verify no aliases were added
    let content = fs::read_to_string(&df).unwrap();
    assert!(!content.contains("cargo"), "no aliases should be added without --apply");
}

#[test]
fn risky_suggestion_flagged_and_not_auto_appliable() {
    let home = tmp_home();
    write_zsh_history(&home);
    let df = data_file(&home);
    let af = aliases_file(&home);

    fs::create_dir_all(df.parent().unwrap()).unwrap();
    fs::write(&df, "[aliases]\n").unwrap();

       // Try to auto-apply a risky suggestion
    let output = assert_cmd::Command::cargo_bin("aliasman")
          .unwrap()
          .arg("suggest")
          .arg("--history-file")
          .arg(history_file(&home))
          .arg("--data-file")
          .arg(&df)
          .arg("--aliases-file")
          .arg(&af)
          .arg("--min-count")
          .arg("1")
          .arg("--apply")
          .arg("echo") // matches the risky "echo $(pwd) && ls"
          .output()
          .unwrap();

     // Should fail because it's a risky command
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        !output.status.success() || stderr.contains("Review carefully") || stderr.contains("risky"),
        "risky suggestion should not be auto-applied: {}", stderr
       );
}

#[test]
fn explicit_apply_routes_through_crud_validation() {
    let home = tmp_home();
    write_zsh_history(&home);
    let df = data_file(&home);
    let af = aliases_file(&home);

    fs::create_dir_all(df.parent().unwrap()).unwrap();
    fs::write(&df, "[aliases]\n").unwrap();

       // Apply a non-risky suggestion
    let output = assert_cmd::Command::cargo_bin("aliasman")
          .unwrap()
          .arg("suggest")
          .arg("--history-file")
          .arg(history_file(&home))
          .arg("--data-file")
          .arg(&df)
          .arg("--aliases-file")
          .arg(&af)
          .arg("--min-count")
          .arg("2")
          .arg("--apply")
          .arg("cargo") // matches "cargo build --release"
          .output()
          .unwrap();

    assert!(output.status.success(), "non-risky apply should succeed: {:?}", String::from_utf8(output.stderr.clone()));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Applied"), "should confirm application: {}", stdout);

       // Verify alias was actually added
    let content = fs::read_to_string(&df).unwrap();
    assert!(content.contains("cargo build --release"), "alias should be in store");
}
