use assert_cmd::prelude::*;
use mocks::OutputParser;
use predicates::prelude::*;
use std::{path::Path, process::Command};

mod mocks;

#[test]
fn create_config_if_not_found() {
    let mut cmd = Command::cargo_bin("clir").unwrap();

    cmd.arg("-c").arg("/tmp/.clir");
    cmd.assert().success();
    assert!(predicates::path::is_file().eval(Path::new("/tmp/.clir")));
}

#[test]
fn list_patterns() -> anyhow::Result<()> {
    let mocks = mocks::MockFiles::new()
        .add_config(".clir", vec!["test_files"])?
        .add_dir("test_files")?
        .add_dir("test_files/c")?
        .add_dir("test_files/d")?
        .add_file("test_files/a.tmp", 1024)?
        .add_file("test_files/b.tmp", 1024)?
        .add_file("test_files/c/e.tmp", 1024)?
        .add_file("test_files/d/f.tmp", 1024)?;

    let mut cmd = Command::cargo_bin("clir").unwrap();

    cmd.arg("-c").arg(mocks.config_path());
    let output = cmd.assert().success();
    let output = &output.get_output().stdout;
    let parser = OutputParser::from_stdout(output);

    assert_pattern_entries!(
        parser,
        [("test_files", "4.00KiB", num_dirs = 1, num_files = 0)],
    );
    assert_pattern_summary!(parser, "4.00KiB", num_dirs = 1, num_files = 0);

    Ok(())
}

#[test]
fn list_multiple_patterns() -> anyhow::Result<()> {
    let mocks = mocks::MockFiles::new()
        .add_config(
            ".clir",
            vec![
                "dir1/**/*",
                "dir2",
                "file1",
                "file2",
                "non-existing-pattern/**/*",
            ],
        )?
        .add_dir("dir1")?
        .add_dir("dir2")?
        .add_file("dir1/a.tmp", 1024)?
        .add_file("dir1/b.tmp", 1024)?
        .add_file("dir2/e.tmp", 1024)?
        .add_file("dir2/f.tmp", 1024)?
        .add_file("file1", 1024)?
        .add_file("file2", 1024)?;

    let mut cmd = Command::cargo_bin("clir").unwrap();

    cmd.arg("-c").arg(mocks.config_path());
    let output = cmd.assert().success();
    let output = &output.get_output().stdout;
    let parser = OutputParser::from_stdout(output);

    assert_pattern_entries!(
        parser,
        [
            ("dir1/**/*", "2.00KiB", num_dirs = 0, num_files = 2),
            ("dir2", "2.00KiB", num_dirs = 1, num_files = 0),
            ("file1", "1.00KiB", num_dirs = 0, num_files = 1),
            ("file2", "1.00KiB", num_dirs = 0, num_files = 1)
        ],
    );
    assert_pattern_summary!(parser, "6.00KiB", num_dirs = 1, num_files = 4);

    Ok(())
}

#[test]
fn overlapping_patterns() -> anyhow::Result<()> {
    let mocks = mocks::MockFiles::new()
        .add_config(".clir", vec!["test_files/**/*.tmp", "test_files"])?
        .add_dir("test_files")?
        .add_file("test_files/a.tmp", 1024)?
        .add_file("test_files/b.tmp", 1024)?;

    let mut cmd = Command::cargo_bin("clir").unwrap();

    cmd.arg("-c").arg(mocks.config_path());
    let output = cmd.assert().success();
    let output = &output.get_output().stdout;
    let parser = OutputParser::from_stdout(output);

    assert_pattern_entries!(
        parser,
        [("test_files", "2.00KiB", num_dirs = 1, num_files = 0)],
    );
    assert_pattern_summary!(parser, "2.00KiB", num_dirs = 1, num_files = 0);

    Ok(())
}
