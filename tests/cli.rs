use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "docdoctor-cli-test-{}-{stamp}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
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

fn package_fixture() -> Fixture {
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
    fixture
}

#[test]
fn unwired_test_is_reported_while_wired_tests_run() {
    let fixture = package_fixture();
    fs::write(
        fixture.0.join("docs/properties.md"),
        "---\nid: properties\n---\n```rust docdoctor test=draft\nfn draft() { assert_eq!(value(), 2); }\n```\n```rust docdoctor file=src/lib.rs test=works\nfn works() { assert_eq!(value(), 1); }\n```\n",
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
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(
        stderr
            .contains("docs/properties.md:4: unwired test doc::properties::draft (missing file=)"),
        "{stderr}"
    );
    assert!(stdout.contains("doc::properties::works ... ok"), "{stdout}");
    assert!(stdout.contains("1 document properties passed"), "{stdout}");
    assert!(
        !stdout.contains("doc::properties::draft ... ok"),
        "{stdout}"
    );
}

#[test]
fn document_with_only_unwired_tests_does_not_run_cargo_tests() {
    let fixture = package_fixture();
    fs::write(
        fixture.0.join("docs/properties.md"),
        "---\nid: properties\n---\n```rust docdoctor test=draft\nfn draft() {}\n```\n",
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
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(
        stderr
            .contains("docs/properties.md:4: unwired test doc::properties::draft (missing file=)"),
        "{stderr}"
    );
    assert!(stdout.contains("0 document properties passed"), "{stdout}");
    assert!(!stdout.contains("running 0 tests"), "{stdout}");
}

#[test]
fn nonexistent_file_is_an_error_not_an_unwired_test() {
    let fixture = package_fixture();
    fs::write(
        fixture.0.join("docs/properties.md"),
        "---\nid: properties\n---\n```rust docdoctor file=src/absent.rs test=draft\nfn draft() {}\n```\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_docdoctor"))
        .args(["check", "--manifest-path"])
        .arg(fixture.0.join("Cargo.toml"))
        .arg("docs/properties.md")
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(!output.status.success());
    assert!(
        stderr.contains("docs/properties.md:4: file= target does not exist: src/absent.rs"),
        "{stderr}"
    );
    assert!(!stderr.contains("unwired test"), "{stderr}");
}

fn workspace_fixture() -> Fixture {
    let fixture = Fixture::new();
    for dir in [
        "docs",
        "crates/api/src",
        "crates/api/docs",
        "crates/shared/src",
    ] {
        fs::create_dir_all(fixture.0.join(dir)).unwrap();
    }
    fs::write(
        fixture.0.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/*\"]\nresolver = \"2\"\n[workspace.package]\nedition = \"2024\"\n[workspace.dependencies]\nshared = { path = \"crates/shared\" }\n",
    )
    .unwrap();
    fs::write(
        fixture.0.join("crates/api/Cargo.toml"),
        "[package]\nname = \"api\"\nversion = \"0.0.0\"\nedition.workspace = true\n[dependencies]\nshared.workspace = true\n",
    )
    .unwrap();
    fs::write(
        fixture.0.join("crates/shared/Cargo.toml"),
        "[package]\nname = \"shared\"\nversion = \"0.0.0\"\nedition.workspace = true\n",
    )
    .unwrap();
    fs::write(
        fixture.0.join("crates/api/src/lib.rs"),
        "pub fn value() -> i32 { shared::value() }\n",
    )
    .unwrap();
    fs::write(
        fixture.0.join("crates/shared/src/lib.rs"),
        "pub fn value() -> i32 { 42 }\n",
    )
    .unwrap();
    fixture
}

#[test]
fn virtual_workspace_manifest_runs_properties_in_multiple_members() {
    let fixture = workspace_fixture();
    fs::write(
        fixture.0.join("docs/api.md"),
        "---\nid: api_properties\n---\n```rust docdoctor file=crates/api/src/lib.rs test=reads_shared\nfn reads_shared() { assert_eq!(value(), 42); }\n```\n",
    )
    .unwrap();
    fs::write(
        fixture.0.join("docs/shared.md"),
        "---\nid: shared_properties\n---\n```rust docdoctor file=crates/shared/src/lib.rs test=has_value\nfn has_value() { assert_eq!(value(), 42); }\n```\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_docdoctor"))
        .args(["check", "--manifest-path"])
        .arg(fixture.0.join("Cargo.toml"))
        .args(["docs/api.md", "docs/shared.md"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(
        stdout.contains("doc::api_properties::reads_shared ... ok"),
        "{stdout}"
    );
    assert!(
        stdout.contains("doc::shared_properties::has_value ... ok"),
        "{stdout}"
    );
}

#[test]
fn member_manifest_preserves_workspace_dependencies() {
    let fixture = workspace_fixture();
    fs::write(
        fixture.0.join("crates/api/docs/value.md"),
        "---\nid: member_properties\n---\n```rust docdoctor file=src/lib.rs test=reads_shared\nfn reads_shared() { assert_eq!(value(), 42); }\n```\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_docdoctor"))
        .args(["check", "--manifest-path"])
        .arg(fixture.0.join("crates/api/Cargo.toml"))
        .arg("docs/value.md")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(
        stdout.contains("doc::member_properties::reads_shared ... ok"),
        "{stdout}"
    );
}
