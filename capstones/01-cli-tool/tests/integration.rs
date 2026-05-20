use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn finds_match_in_file() {
    let dir = tempdir().unwrap();
    let f = dir.path().join("a.txt");
    fs::write(&f, "hello\nworld\nfoo bar\n").unwrap();

    let mut cmd = Command::cargo_bin("mini-rg").unwrap();
    cmd.arg("foo").arg(dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("foo bar"));
}

#[test]
fn case_insensitive() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.txt"), "Hello World").unwrap();

    Command::cargo_bin("mini-rg").unwrap()
        .arg("-i").arg("hello").arg(dir.path())
        .assert().success()
        .stdout(predicate::str::contains("Hello World"));
}

#[test]
fn glob_filter() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.rs"), "fn x() {}").unwrap();
    fs::write(dir.path().join("a.txt"), "fn ignored").unwrap();

    Command::cargo_bin("mini-rg").unwrap()
        .arg("-g").arg("*.rs").arg("fn").arg(dir.path())
        .assert().success()
        .stdout(predicate::str::contains("a.rs"))
        .stdout(predicate::str::contains("a.txt").not());
}

#[test]
fn files_with_matches() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.txt"), "foo").unwrap();
    fs::write(dir.path().join("b.txt"), "bar").unwrap();

    Command::cargo_bin("mini-rg").unwrap()
        .arg("-l").arg("foo").arg(dir.path())
        .assert().success()
        .stdout(predicate::str::contains("a.txt"))
        .stdout(predicate::str::contains("b.txt").not());
}
