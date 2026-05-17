use std::path::PathBuf;
use std::process::Command;
use std::fs;

fn e2e_binary() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("x86_64-unknown-linux-musl");
    path.push("debug");
    path.push("elang");
    path
}

fn run_e2e(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("programs");
    path.push(name);
    path.set_extension("elang");

    let output = Command::new(e2e_binary())
        .arg("run")
        .arg(path.to_str().unwrap())
        .output()
        .expect("Failed to run elang binary");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("elang program '{}' failed:\n{}", name, stderr);
    }

    String::from_utf8_lossy(&output.stdout).to_string()
}

fn read_expected(name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("programs");
    path.push(name);
    path.set_extension("expected");
    fs::read_to_string(path.to_str().unwrap()).expect("Expected output file not found")
}

#[test]
fn e2e_fizzbuzz() {
    let output = run_e2e("fizzbuzz");
    let expected = read_expected("fizzbuzz");
    assert_eq!(output.trim(), expected.trim());
}

#[test]
fn e2e_fibonacci() {
    let output = run_e2e("fibonacci");
    let expected = read_expected("fibonacci");
    assert_eq!(output.trim(), expected.trim());
}

#[test]
fn e2e_basics() {
    let output = run_e2e("basics");
    let expected = read_expected("basics");
    assert_eq!(output.trim(), expected.trim());
}

#[test]
fn e2e_arithmetic() {
    let output = run_e2e("arithmetic");
    let expected = read_expected("arithmetic");
    assert_eq!(output.trim(), expected.trim());
}

#[test]
fn e2e_control_flow() {
    let output = run_e2e("control_flow");
    let expected = read_expected("control_flow");
    assert_eq!(output.trim(), expected.trim());
}

#[test]
fn e2e_data_structures() {
    let output = run_e2e("data_structures");
    let expected = read_expected("data_structures");
    assert_eq!(output.trim(), expected.trim());
}

#[test]
fn e2e_functions_and_recursion() {
    let output = run_e2e("functions_recursion");
    let expected = read_expected("functions_recursion");
    assert_eq!(output.trim(), expected.trim());
}

#[test]
fn e2e_classes() {
    let output = run_e2e("classes");
    let expected = read_expected("classes");
    assert_eq!(output.trim(), expected.trim());
}

#[test]
fn e2e_match_statement() {
    let output = run_e2e("match_statement");
    let expected = read_expected("match_statement");
    assert_eq!(output.trim(), expected.trim());
}

#[test]
fn e2e_lambdas() {
    let output = run_e2e("lambdas");
    let expected = read_expected("lambdas");
    assert_eq!(output.trim(), expected.trim());
}

#[test]
fn e2e_pipe_operator() {
    let output = run_e2e("pipe_operator");
    let expected = read_expected("pipe_operator");
    assert_eq!(output.trim(), expected.trim());
}

#[test]
fn e2e_stdlib_math() {
    let output = run_e2e("stdlib_math");
    let expected = read_expected("stdlib_math");
    assert_eq!(output.trim(), expected.trim());
}

#[test]
fn e2e_stdlib_string() {
    let output = run_e2e("stdlib_string");
    let expected = read_expected("stdlib_string");
    assert_eq!(output.trim(), expected.trim());
}
