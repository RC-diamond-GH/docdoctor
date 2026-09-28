use crate::Result;
use crate::document::Block;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn run(root: &Path, parsed: &BTreeMap<String, Vec<Block>>) -> Result<()> {
    let blocks: Vec<_> = parsed
        .iter()
        .flat_map(|(id, blocks)| blocks.iter().map(move |block| (id.as_str(), block)))
        .collect();
    if blocks.is_empty() {
        return Err("no `rust docdoctor` blocks found".into());
    }
    for (_, block) in &blocks {
        let path = fs::canonicalize(root.join(&block.file))?;
        if !path.starts_with(root) || !path.is_file() {
            return Err(format!(
                "target is not a package source file: {}",
                block.file.display()
            )
            .into());
        }
    }
    let staged = StagedPackage::new()?;
    copy_package(root, &staged.path)?;
    let mut sources: BTreeMap<&Path, BTreeMap<&str, Vec<&Block>>> = BTreeMap::new();
    for &(id, block) in &blocks {
        sources
            .entry(&block.file)
            .or_default()
            .entry(id)
            .or_default()
            .push(block);
    }
    for (file, docs) in sources {
        let path = staged.path.join(file);
        let mut source = fs::read_to_string(&path)?;
        source.push_str("\n#[cfg(test)]\nmod doc {\n");
        for (id, snippets) in docs {
            source.push_str(&format!("    mod {id} {{\n        use super::super::*;\n"));
            for block in snippets {
                source.push_str(&format!(
                    "        // {}:{}\n        #[test]\n        fn {}() {{\n{}\n            {}();\n        }}\n",
                    block.source.display(),
                    block.line,
                    block.test,
                    block.code,
                    block.test
                ));
                println!(
                    "{}:{} -> doc::{id}::{}",
                    block.source.display(),
                    block.line,
                    block.test
                );
            }
            source.push_str("    }\n");
        }
        source.push_str("}\n");
        fs::write(path, source)?;
    }

    let target_dir = staged.path.join("target");
    let output = Command::new("cargo")
        .args(["test", "--all-targets", "--manifest-path"])
        .arg(staged.path.join("Cargo.toml"))
        .args(["doc::", "--", "--list"])
        .env("CARGO_TARGET_DIR", &target_dir)
        .output()?;
    if !output.status.success() {
        std::io::Write::write_all(&mut std::io::stderr(), &output.stderr)?;
        std::io::Write::write_all(&mut std::io::stdout(), &output.stdout)?;
        return Err("Cargo could not list the generated tests".into());
    }
    let listing = String::from_utf8(output.stdout)?;
    for &(id, block) in &blocks {
        let name = format!("::doc::{id}::{}: test", block.test);
        if !listing
            .lines()
            .any(|line| line.ends_with(&name) || line == &name[2..])
        {
            return Err(format!(
                "{}:{}: generated test doc::{id}::{} was not discovered by Cargo; check the target file",
                block.source.display(),
                block.line,
                block.test
            )
            .into());
        }
    }
    let output = Command::new("cargo")
        .args(["test", "--all-targets", "--manifest-path"])
        .arg(staged.path.join("Cargo.toml"))
        .arg("doc::")
        .env("CARGO_TARGET_DIR", target_dir)
        .stdin(Stdio::inherit())
        .output()?;
    std::io::Write::write_all(&mut std::io::stdout(), &output.stdout)?;
    std::io::Write::write_all(&mut std::io::stderr(), &output.stderr)?;
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut failed = BTreeSet::new();
        for line in stdout.lines() {
            if let Some(name) = line
                .strip_prefix("test ")
                .and_then(|line| line.strip_suffix(" ... FAILED"))
            {
                let mut parts = name.rsplit("::");
                if let (Some(test), Some(id), Some("doc")) =
                    (parts.next(), parts.next(), parts.next())
                {
                    failed.insert((id, test));
                }
            }
        }
        for &(id, block) in &blocks {
            if failed.contains(&(id, block.test.as_str())) {
                eprintln!(
                    "{}:{}: property test doc::{id}::{} failed",
                    block.source.display(),
                    block.line,
                    block.test
                );
            }
        }
        return Err(format!("document tests failed (cargo exit: {})", output.status).into());
    }
    println!("{} document properties passed", blocks.len());
    Ok(())
}

fn copy_package(source: &Path, destination: &Path) -> Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        if name == "target" || name == ".git" {
            continue;
        }
        let kind = entry.file_type()?;
        let path = destination.join(name);
        if kind.is_dir() {
            fs::create_dir(&path)?;
            copy_package(&entry.path(), &path)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), path)?;
        } else {
            return Err(format!(
                "cannot stage symlink or special file: {}",
                entry.path().display()
            )
            .into());
        }
    }
    Ok(())
}

static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);

struct StagedPackage {
    path: PathBuf,
}

impl StagedPackage {
    fn new() -> Result<Self> {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = env::temp_dir().join(format!(
            "docdoctor-{}-{stamp}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self { path })
    }
}

impl Drop for StagedPackage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
