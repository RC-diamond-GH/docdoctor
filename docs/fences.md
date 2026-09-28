---
id: fences
---

# 只执行明确标记的代码围栏

普通 `rust` 围栏和波浪线围栏即使包含类似测试的内容，也不会被当成性质测试；只有反引号围栏的开头包含 `rust docdoctor` 才会提取。下面构造一段混合 Markdown，确认只发现明确标记的测试。

```rust docdoctor file=src/document.rs test=only_marked_backtick_fences_run
fn only_marked_backtick_fences_run() {
    let markdown = concat!(
        "---\nid: sample\n---\n",
        "~~~rust docdoctor file=src/document.rs test=not_executable\n",
        "fn not_executable() {}\n~~~\n",
        "```rust\nfn plain_rust() {}\n```\n",
        "```rust docdoctor file=src/document.rs test=executable\n",
        "fn executable() {}\n```\n",
    );
    let document = parse_document(markdown, Path::new("docs/sample.md")).unwrap();
    assert_eq!(document.blocks.len(), 1);
    assert_eq!(document.blocks[0].test, "executable");
}
```

围栏可以使用多于三个反引号；只有不少于开头长度的结束围栏才能关闭它。因此四反引号围栏内的三个反引号属于内容，而不是新测试。

```rust docdoctor file=src/document.rs test=shorter_closing_fence_stays_inside
fn shorter_closing_fence_stays_inside() {
    let markdown = concat!(
        "---\nid: sample\n---\n",
        "````rust docdoctor file=src/document.rs test=long_fence\n",
        "first line\n```\nlast line\n",
        "````\n",
    );
    let document = parse_document(&markdown, Path::new("docs/sample.md")).unwrap();
    assert_eq!(document.blocks.len(), 1);
    assert_eq!(document.blocks[0].test, "long_fence");
    assert_eq!(document.blocks[0].code, "first line\n```\nlast line\n");
}
```
