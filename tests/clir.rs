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
    let _files = mocks::MockFiles::new()
        .add_config("/tmp/.clir", vec!["/tmp/clir/test_files".to_owned()])?
        .add_dir("/tmp/clir/test_files")?
        .add_dir("/tmp/clir/test_files/c")?
        .add_dir("/tmp/clir/test_files/d")?
        .add_file("/tmp/clir/test_files/a.tmp", 1024)?
        .add_file("/tmp/clir/test_files/b.tmp", 1024)?
        .add_file("/tmp/clir/test_files/c/e.tmp", 1024)?
        .add_file("/tmp/clir/test_files/d/f.tmp", 1024);

    let mut cmd = Command::cargo_bin("clir").unwrap();

    cmd.arg("-c").arg("/tmp/.clir");
    let output = cmd.assert().success();
    let output = &output.get_output().stdout;
    let parser = OutputParser::from_stdout(output);

    assert_pattern!(parser, "/tmp/clir/test_files", "4.00K");
    assert_pattern_at!(parser, 0, "/tmp/clir/test_files");
    assert_pattern_summary!(parser, "4.00K");

    Ok(())
}

#[test]
fn overlapping_patterns() -> anyhow::Result<()> {
    let _files = mocks::MockFiles::new()
        .add_config(
            "/tmp/.clir",
            vec![
                "/tmp/clir/test_files/**/*.tmp".to_owned(),
                "/tmp/clir/test_files".to_owned(),
            ],
        )?
        .add_dir("/tmp/clir/test_files")?
        .add_file("/tmp/clir/test_files/a.tmp", 1024)?
        .add_file("/tmp/clir/test_files/b.tmp", 1024)?;

    let mut cmd = Command::cargo_bin("clir").unwrap();

    cmd.arg("-c").arg("/tmp/.clir");
    let output = cmd.assert().success();
    let output = &output.get_output().stdout;
    let parser = OutputParser::from_stdout(output);

    assert_pattern!(parser, "/tmp/clir/test_files", "2.00K");
    assert_pattern_at!(parser, 0, "/tmp/clir/test_files");
    assert_pattern_at!(parser, 1, None);
    assert_pattern_summary!(parser, "2.00K");

    Ok(())
}
