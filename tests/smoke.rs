use std::io::Write;
use std::process::{Command, Stdio};

fn run_cogshell(args: &[&str], stdin: &str) -> (String, String, i32) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cogshell"))
        .args(&["-", "-o", "-"])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn cogshell");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    let output = child.wait_with_output().expect("wait failed");
    (
        String::from_utf8(output.stdout).expect("stdout not utf8"),
        String::from_utf8(output.stderr).expect("stderr not utf8"),
        output.status.code().unwrap_or(-1),
    )
}

#[test]
fn replaces_block_output_with_command_output() {
    let input = "\
<!--[[[cogsh echo hello]]]-->
STALE OUTPUT
<!--[[[end]]]-->
";
    let expected = "\
<!--[[[cogsh echo hello]]]-->
hello
<!--[[[end]]]-->
";
    let (stdout, stderr, code) = run_cogshell(&[], input);
    assert_eq!(code, 0, "non-zero exit; stderr:\n{stderr}");
    assert_eq!(stdout, expected);
}

#[test]
fn preserves_content_outside_blocks() {
    let input = "\
header line
<!--[[[cogsh echo middle]]]-->
old
<!--[[[end]]]-->
footer line
";
    let expected = "\
header line
<!--[[[cogsh echo middle]]]-->
middle
<!--[[[end]]]-->
footer line
";
    let (stdout, stderr, code) = run_cogshell(&[], input);
    assert_eq!(code, 0, "non-zero exit; stderr:\n{stderr}");
    assert_eq!(stdout, expected);
}

#[test]
fn appends_suffix_to_each_output_line() {
    let input = "\
<!--[[[cogsh echo one; echo two]]]-->
<!--[[[end]]]-->
";
    let expected = "\
<!--[[[cogsh echo one; echo two]]]-->
one #
two #
<!--[[[end]]]-->
";
    let (stdout, stderr, code) = run_cogshell(&["--suffix", " #"], input);
    assert_eq!(code, 0, "non-zero exit; stderr:\n{stderr}");
    assert_eq!(stdout, expected);
}
