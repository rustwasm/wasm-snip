use std::fs::File;
use std::io::Read;
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
    let expected = String::from_utf8_lossy(&expected);

    let actual = {
        let mut file = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))
            .expect("should open README.md file");
        let mut s = String::new();
        file.read_to_string(&mut s)
            .expect("should read contents of file to string");
        s
    };

    if actual != expected {
        panic!("Run `cargo readme > README.md` to update README.md");
    }
}

#[test]
fn snip_me() {
    let status = Command::new("cargo")
        .args(&["run", "--", "-o"])
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snipped.wasm"))
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/hello.wasm"))
        .arg("_ZN5hello7snip_me17hf15dbd799e7ad6aaE")
        .status()
        .unwrap();
    assert!(status.success());

    let expected = {
        let mut file = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/expected.wasm"))
            .expect("should open expected.wasm file");
        let mut e = Vec::new();
        file.read_to_end(&mut e)
            .expect("should read contents of file to vec");
        e
    };

    let actual = {
        let mut file = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snipped.wasm"))
            .expect("should open snipped.wasm file");
        let mut a = Vec::new();
        file.read_to_end(&mut a)
            .expect("should read contents of file to vec");
        a
    };

    if actual != expected {
        panic!("snipping `snip_me` did not result in expected wasm file");
    }
}

#[test]
fn patterns() {
    let status = Command::new("cargo")
        .args(&["run", "--", "-o"])
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/no_alloc_actual.wasm"
        ))
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/hello.wasm"))
        .arg("-p")
        .arg(".*alloc.*")
        .status()
        .unwrap();
    assert!(status.success());

    let expected = {
        let mut file = File::open(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/no_alloc_expected.wasm"
        )).expect("should open no_alloc_expected.wasm file");
        let mut e = Vec::new();
        file.read_to_end(&mut e)
            .expect("should read contents of file to vec");
        e
    };

    let actual = {
        let mut file = File::open(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/no_alloc_actual.wasm"
        )).expect("should open no_alloc_actual.wasm file");
        let mut a = Vec::new();
        file.read_to_end(&mut a)
            .expect("should read contents of file to vec");
        a
    };

    if actual != expected {
        panic!("snipping `.*alloc.*` did not result in expected wasm file");
    }
}

#[test]
fn snip_rust_fmt_code() {
    let status = Command::new("cargo")
        .args(&["run", "--", "-o"])
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/no_fmt_actual.wasm"
        ))
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/hello.wasm"))
        .arg("--snip-rust-fmt-code")
        .status()
        .unwrap();
    assert!(status.success());

    let expected = {
        let mut file = File::open(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/no_fmt_expected.wasm"
        )).expect("should open no_fmt_expected.wasm file");
        let mut e = Vec::new();
        file.read_to_end(&mut e)
            .expect("should read contents of file to vec");
        e
    };

    let actual = {
        let mut file = File::open(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/no_fmt_actual.wasm"
        )).expect("should open no_fmt_actual.wasm file");
        let mut a = Vec::new();
        file.read_to_end(&mut a)
            .expect("should read contents of file to vec");
        a
    };

    if actual != expected {
        panic!("`--snip-rust-fmt-code` did not result in expected wasm file");
    }
}
