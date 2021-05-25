use assert_cmd::prelude::*;
use std::fs;
use std::path::Path;
use std::process::Command;

#[test]
fn cargo_readme_up_to_date() {
    println!("Checking that `cargo readme > README.md` is up to date...");

    let expected = Command::new("cargo")
        .arg("readme")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("should run `cargo readme` OK")
        .stdout;
    let expected = String::from_utf8(expected).unwrap();

    let actual = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))
        .expect("should open README.md file");

    if actual != expected {
        panic!("Run `cargo readme > README.md` to update README.md");
    }
}

fn assert_snip<P: AsRef<Path>>(cmd: &mut Command, expected_path: P) {
    let expected_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(expected_path);

    let actual_path = expected_path.with_extension("wasm.actual");

    cmd.arg("--skip-producers-section")
        .arg("-o")
        .arg(&actual_path)
        .assert()
        .success();

    let expected = fs::read(&expected_path).expect("should open expected wasm file");
    let actual = fs::read(&actual_path).expect("should open snipped.wasm file");

    if actual != expected {
        panic!(
            "snipping did not result in expected wasm file: {} != {}",
            expected_path.display(),
            actual_path.display(),
        );
    }
}

fn wasm_snip() -> Command {
    let mut cmd = Command::cargo_bin("wasm-snip").unwrap();
    cmd.arg(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("hello.wasm"),
    );
    cmd
}

#[test]
fn snip_me() {
    assert_snip(
        wasm_snip().arg("_ZN5hello7snip_me17hf15dbd799e7ad6aaE"),
        "snip_me.wasm",
    );
}

#[test]
fn patterns() {
    assert_snip(wasm_snip().arg("-p").arg(".*alloc.*"), "no_alloc.wasm");
}

#[test]
fn snip_rust_fmt_code() {
    assert_snip(wasm_snip().arg("--snip-rust-fmt-code"), "no_fmt.wasm");
}

#[test]
fn snip_rust_panicking_code() {
    assert_snip(
        wasm_snip().arg("--snip-rust-panicking-code"),
        "no_panicking.wasm",
    );
}

#[test]
fn keep_exports() {
    assert_snip(wasm_snip().arg("-k").arg("keep_me"), "kept_me.wasm");
}

#[test]
fn keep_export_patterns() {
    assert_snip(wasm_snip().arg("-x").arg("keep_me"), "kept_me_too.wasm");
}
