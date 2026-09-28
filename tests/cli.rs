use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("docdoctor-cli-test-{}-{stamp}", std::process::id()));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn failure_reports_the_document_and_only_the_failed_property() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.0.join("src")).unwrap();
    fs::create_dir(fixture.0.join("docs")).unwrap();
    fs::write(
        fixture.0.join("Cargo.toml"),
        "[package]\nname = \"docdoctor-cli-fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    fs::write(
        fixture.0.join("src/lib.rs"),
        "pub fn value() -> i32 { 1 }\n",
    )
    .unwrap();
    fs::write(
        fixture.0.join("docs/properties.md"),
        "---\nid: properties\n---\n# Properties\n```rust docdoctor file=src/lib.rs test=works\nfn works() { assert_eq!(value(), 1); }\n```\n```rust docdoctor file=src/lib.rs test=broken\nfn broken() { assert_eq!(value(), 2); }\n```\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_docdoctor"))
        .args(["check", "--manifest-path"])
        .arg(fixture.0.join("Cargo.toml"))
        .arg("docs/properties.md")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(!output.status.success());
    assert!(stdout.contains("doc::properties::works ... ok"), "{stdout}");
    assert!(
        stdout.contains("doc::properties::broken ... FAILED"),
        "{stdout}"
    );
    assert!(
        stderr.contains("docs/properties.md:8: property test doc::properties::broken failed"),
        "{stderr}"
    );
    assert!(
        !stderr.contains("property test doc::properties::works failed"),
        "{stderr}"
    );
}
