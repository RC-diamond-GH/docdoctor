use crate::Result;
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

pub(crate) struct Block {
    pub(crate) source: PathBuf,
    pub(crate) line: usize,
    pub(crate) file: PathBuf,
    pub(crate) test: String,
    pub(crate) code: String,
}

pub(crate) struct Document {
    pub(crate) id: String,
    pub(crate) blocks: Vec<Block>,
}

pub(crate) fn parse_document(text: &str, source: &Path) -> Result<Document> {
    let mut blocks = Vec::new();
    let lines: Vec<_> = text.lines().collect();
    let (id, mut index) = parse_document_id(&lines, source)?;
    while index < lines.len() {
        let line = lines[index];
        let trimmed = line.trim_start_matches(' ');
        if line.len() - trimmed.len() > 3 {
            index += 1;
            continue;
        }
        let marker = trimmed.as_bytes().first().copied();
        if !matches!(marker, Some(b'`' | b'~')) {
            index += 1;
            continue;
        }
        let marker = marker.unwrap();
        let fence_len = trimmed.bytes().take_while(|byte| *byte == marker).count();
        if fence_len < 3 {
            index += 1;
            continue;
        }
        let info = trimmed[fence_len..].trim();
        let start = index + 1;
        index += 1;
        let mut code = String::new();
        while index < lines.len() {
            let closing = lines[index].trim_start_matches(' ');
            let spaces = lines[index].len() - closing.len();
            let count = closing.bytes().take_while(|byte| *byte == marker).count();
            if spaces <= 3 && count >= fence_len && closing[count..].trim().is_empty() {
                break;
            }
            code.push_str(lines[index]);
            code.push('\n');
            index += 1;
        }
        let mut tokens = info.split_whitespace();
        if marker == b'`' && tokens.next() == Some("rust") && tokens.next() == Some("docdoctor") {
            if index == lines.len() {
                return Err(
                    format!("{}:{start}: unclosed docdoctor fence", source.display()).into(),
                );
            }
            let (file, test) = parse_metadata(info, source, start)?;
            blocks.push(Block {
                source: source.to_path_buf(),
                line: start,
                file,
                test,
                code,
            });
        }
        index += 1;
    }
    let mut tests = BTreeSet::new();
    for block in &blocks {
        if !tests.insert(block.test.as_str()) {
            return Err(format!(
                "{}:{}: duplicate test=: {}",
                source.display(),
                block.line,
                block.test
            )
            .into());
        }
    }
    Ok(Document {
        id: id.to_owned(),
        blocks,
    })
}

fn parse_document_id<'a>(lines: &[&'a str], source: &Path) -> Result<(&'a str, usize)> {
    if lines.first().is_none_or(|line| line.trim() != "---") {
        return Err(format!(
            "{}:1: expected front matter starting with --- and id:",
            source.display()
        )
        .into());
    }
    let mut id = None;
    for (index, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            let id = id.ok_or_else(|| format!("{}: missing front matter id:", source.display()))?;
            return Ok((id, index + 1));
        }
        if let Some((key, value)) = line.split_once(':') {
            if key.trim() == "id" {
                if id.is_some() {
                    return Err(format!(
                        "{}:{}: duplicate front matter id:",
                        source.display(),
                        index + 1
                    )
                    .into());
                }
                let value = value.trim();
                if !valid_identifier(value) {
                    return Err(format!(
                        "{}:{}: id: must be a Rust identifier",
                        source.display(),
                        index + 1
                    )
                    .into());
                }
                id = Some(value);
            }
        }
    }
    Err(format!("{}: unclosed front matter", source.display()).into())
}

fn parse_metadata(info: &str, source: &Path, line: usize) -> Result<(PathBuf, String)> {
    let mut file = None;
    let mut test = None;
    for entry in info.split_whitespace().skip(2) {
        let (key, value) = entry
            .split_once('=')
            .ok_or_else(|| format!("{}:{line}: expected key=value: {entry}", source.display()))?;
        match key {
            "file" if file.is_none() => file = Some(PathBuf::from(value)),
            "test" if test.is_none() => test = Some(value.to_owned()),
            _ => {
                return Err(format!(
                    "{}:{line}: unknown or duplicate metadata: {key}",
                    source.display()
                )
                .into());
            }
        }
    }
    let file = file.ok_or_else(|| format!("{}:{line}: missing file=", source.display()))?;
    if !file.starts_with("src")
        || file.extension().and_then(|ext| ext.to_str()) != Some("rs")
        || file
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(format!(
            "{}:{line}: file= must be a relative .rs path under src/",
            source.display()
        )
        .into());
    }
    let test = test.ok_or_else(|| format!("{}:{line}: missing test=", source.display()))?;
    if !valid_identifier(&test) {
        return Err(format!(
            "{}:{line}: test= must be a Rust identifier",
            source.display()
        )
        .into());
    }
    Ok((file, test))
}

fn valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some('a'..='z' | 'A'..='Z' | '_'))
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_multiple_fences_without_treating_plain_rust_as_tests() {
        let markdown = "---\nid: scheduling\n---\n~~~rust\n```rust docdoctor file=src/model.rs test=ignored\nfn ignored() {}\n```\n~~~\n```rust\nfn ordinary() {}\n```\n```rust docdoctor file=src/model.rs test=law\nfn law() {}\n````\n";
        let document = parse_document(markdown, Path::new("model.md")).unwrap();
        assert_eq!(document.id, "scheduling");
        assert_eq!(document.blocks.len(), 1);
        assert_eq!(document.blocks[0].test, "law");
        assert_eq!(document.blocks[0].line, 12);
    }

    #[test]
    fn rejects_escaping_source_path() {
        let error = parse_document(
            "---\nid: scheduling\n---\n```rust docdoctor file=src/../escape.rs test=law\nfn law() {}\n```",
            Path::new("model.md"),
        )
        .err()
        .unwrap()
        .to_string();
        assert!(error.contains("relative .rs path"));
    }

    #[test]
    fn rejects_missing_document_id() {
        let error = parse_document(
            "---\ntitle: Scheduler\n---\n```rust docdoctor file=src/model.rs test=law\nfn law() {}\n```",
            Path::new("model.md"),
        )
        .err()
        .unwrap()
        .to_string();
        assert!(error.contains("missing front matter id:"), "{error}");
    }
}
