# docdoctor

将 Markdown 中关于 Rust 模型性质的可执行例子作为单元测试运行。文档陈述的是一般性质；有限测试只能验证具体实例，不能替代证明。

```sh
cargo run -- check --manifest-path examples/schedule-tree/Cargo.toml docs/schedule-tree.md
```

`--manifest-path` 指向被验证的 package；文档路径相对于该 package 根目录。可以传入多个文档路径。执行失败时 CLI 返回非零状态。

## 文档与代码块元数据

文档开头须有 `id` 字段；它是有效的 ASCII Rust 标识符，在一次检查所传的文档之间不可重复。代码块中的 `test=` 在同一文档内也不可重复。

````markdown
---
id: scheduling
---

```rust docdoctor file=src/schedule_tree.rs test=groups_alternate
fn groups_alternate() {
    let mut tree = ScheduleTree::new(vec![vec!["A", "B"], vec!["C"]]);
    assert_eq!(tree.tick(), "A");
    assert_eq!(tree.tick(), "C");
}
```
````

- `rust docdoctor`：标记一个待执行的反引号围栏代码块；普通 Rust 代码块不会执行。
- `file=`：package 内 `src/` 下已有的 `.rs` 模块文件，不能包含 `..` 或链接；例如 `src/schedule_tree.rs` 对应 `crate::schedule_tree`。
- `test=`：代码块中无参数函数的名字（ASCII Rust 标识符），例如 `groups_alternate`。工具生成 `#[test]` 包装函数并调用它；代码块不必自行添加 `#[test]`。

工具把每个文件的代码块追加到临时副本的 `#[cfg(test)] mod doc`，并按文档 `id` 创建子模块。因此例子能访问所在文件模块的私有成员，也可用 `crate::...` 引用其他模块。上述测试运行在 `crate::schedule_tree::doc::scheduling::groups_alternate`。工具先检查 Cargo 是否发现每个生成的测试，再调用 `cargo test --all-targets` 执行 `doc::` 过滤出的测试。原始源码和文档不被改写；临时副本及其构建产物在执行后清理。

通过时 Cargo 输出稳定的 `doc::<id>::<test>` 名称；失败时 docdoctor 保留 Cargo 的断言输出，并额外打印 `docs/schedule-tree.md:31: property test doc::scheduling::branch_cursors_are_independent failed`。行号是文档代码围栏的起始行；Cargo 的 `src/...:行号` 则指向临时副本里的注入代码。

目前面向单个 package：复制 package 内容到临时目录后执行 Cargo；不支持 package 内符号链接，以及依赖 package 根目录外相对路径的 workspace / path dependency。需要 `cargo` 可用。大型 package 的文件复制有额外开销。
