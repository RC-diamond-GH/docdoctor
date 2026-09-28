use crate::Result;
use crate::document::{Document, parse_document};
use crate::runner;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const HELP: &str = "Usage: docdoctor check [--manifest-path PATH] DOC.md [DOC.md ...]\n\
    Run Markdown fences marked `rust docdoctor file=src/module.rs test=name` as Cargo unit tests.\n\
    Paths are relative to the selected manifest; test names are functions defined in their fence.";

pub(crate) fn run() -> Result<()> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("--help" | "-h") => {
            println!("{HELP}");
            return Ok(());
        }
        Some("check") => {}
        _ => return Err(HELP.into()),
    }
    let mut manifest = PathBuf::from("Cargo.toml");
    let mut documents = Vec::new();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--manifest-path" => {
                manifest = PathBuf::from(args.next().ok_or("missing manifest path")?)
            }
            "--help" | "-h" => {
                println!("{HELP}");
                return Ok(());
            }
            _ if arg.starts_with('-') => return Err(format!("unknown option: {arg}").into()),
            _ => documents.push(PathBuf::from(arg)),
        }
    }
    if documents.is_empty() {
        return Err("provide at least one Markdown document".into());
    }
    let manifest = fs::canonicalize(manifest)?;
    if manifest.file_name().and_then(|name| name.to_str()) != Some("Cargo.toml") {
        return Err("--manifest-path must point to Cargo.toml".into());
    }
    let root = manifest.parent().ok_or("manifest has no parent")?;
    let mut parsed = BTreeMap::new();
    for document in documents {
        let path = fs::canonicalize(root.join(document))?;
        let relative = path
            .strip_prefix(root)
            .map_err(|_| "document is outside the selected manifest directory")?;
        if !path.is_file() {
            return Err(format!("not a file: {}", path.display()).into());
        }
        let Document { id, blocks } = parse_document(&fs::read_to_string(&path)?, relative)?;
        if parsed.contains_key(&id) {
            return Err(format!("{}: duplicate document id: {id}", relative.display()).into());
        }
        parsed.insert(id, blocks);
    }

    let workspace = Command::new("cargo")
        .args([
            "locate-project",
            "--workspace",
            "--message-format",
            "plain",
            "--manifest-path",
        ])
        .arg(&manifest)
        .output()?;
    if !workspace.status.success() {
        return Err(format!(
            "could not locate Cargo workspace: {}",
            String::from_utf8_lossy(&workspace.stderr).trim()
        )
        .into());
    }
    let workspace_manifest =
        fs::canonicalize(Path::new(String::from_utf8(workspace.stdout)?.trim()))?;
    let workspace_root = workspace_manifest
        .parent()
        .ok_or("workspace manifest has no parent")?;
    if !root.starts_with(workspace_root) {
        return Err("selected package is outside the workspace root and cannot be staged".into());
    }
    runner::run(root, workspace_root, &parsed)
}
