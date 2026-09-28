---
id: metadata
---

# 名称和目标文件的约束

文档开头的 `id` 是测试模块名，必须是 ASCII Rust 标识符。每个围栏的 `test=` 是该文档内唯一的函数名；同一文档不能用同一个名字声明两次。

```rust docdoctor file=src/document.rs test=duplicate_test_names_are_rejected
fn duplicate_test_names_are_rejected() {
    let markdown = concat!(
        "---\nid: sample\n---\n",
        "```rust docdoctor file=src/document.rs test=law\nfn law() {}\n```\n",
        "```rust docdoctor file=src/document.rs test=law\nfn law() {}\n```\n",
    );
    let error = parse_document(markdown, Path::new("docs/sample.md"))
        .err().unwrap().to_string();
    assert!(error.contains("duplicate test=: law"), "{error}");
}
```

缺失或非法 `id` 在运行测试前被拒绝，避免生成无效的 Rust 模块名。

```rust docdoctor file=src/document.rs test=document_id_must_be_a_rust_identifier
fn document_id_must_be_a_rust_identifier() {
    for id in ["bad-name", "9starts_with_digit", "含中文"] {
        let markdown = format!("---\nid: {id}\n---\n");
        let error = parse_document(&markdown, Path::new("docs/sample.md"))
            .err().unwrap().to_string();
        assert!(error.contains("id: must be a Rust identifier"), "{id}: {error}");
    }
}
```

`file=` 必须是相对路径、以 `.rs` 结尾，并经过某个 package 的 `src/`；不能用 `..` 绕出选定目录。目标文件是否实际存在则在检查执行阶段验证。

```rust docdoctor file=src/document.rs test=source_paths_cannot_escape_the_package
fn source_paths_cannot_escape_the_package() {
    for path in ["src/../outside.rs", "/tmp/outside.rs", "tests/example.rs"] {
        let markdown = format!(
            "---\nid: sample\n---\n```rust docdoctor file={path} test=law\nfn law() {{}}\n```\n"
        );
        let error = parse_document(&markdown, Path::new("docs/sample.md"))
            .err().unwrap().to_string();
        assert!(error.contains("relative .rs path under a package's src/"), "{path}: {error}");
    }
}
```
