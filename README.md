# docdoctor

将 Markdown 中关于 Rust 模型性质的可执行例子作为单元测试运行。文档陈述的是一般性质；有限测试只能验证具体实例，不能替代证明。

## 运行示例

在仓库根目录逐条运行（需先安装 `docdoctor` 命令）；每条命令的文档路径均相对于对应的 `--manifest-path` 所在目录：

```sh
# 省略 file=：成功退出，提示未接线；0 document properties passed
docdoctor check --manifest-path examples/schedule-tree/Cargo.toml docs/unwired.md

# workspace 根 manifest：成功退出，跨两个成员的 2 项测试通过
docdoctor check --manifest-path examples/workspace/Cargo.toml docs/api.md docs/shared.md

# 测试未通过：非零退出；报告 docs/schedule-tree.md:31 和实际值 D、预期值 E
docdoctor check --manifest-path examples/schedule-tree/Cargo.toml docs/schedule-tree.md

# 显式 file= 指向不存在的文件：非零退出；报告 src/not_yet_implemented.rs
docdoctor check --manifest-path examples/schedule-tree/Cargo.toml docs/missing-target.md
```

`--manifest-path` 指向被验证的 package 或 workspace 根目录的 `Cargo.toml`；文档路径相对于该 manifest 所在目录。可以传入多个文档路径。执行失败时 CLI 返回非零状态。

workspace 根目录文档的 `file=` 使用相对于 workspace 根目录的成员源码路径（如 `crates/api/src/lib.rs`）。也可以选择成员 manifest；此时文档和 `file=src/lib.rs` 均相对于成员目录。选择根 manifest 会测试整个 workspace；选择成员 manifest 只测试该成员，同时保留 workspace 继承设置及 workspace 内的相对路径依赖。

## 自举：检查本项目文档

`docs/fences.md` 描述哪些围栏会成为测试，以及围栏长度的规则；`docs/metadata.md` 描述文档和测试名称、源码目标路径的约束。两个文档的 `rust docdoctor` 围栏都接线到本项目的 `src/document.rs`，检查时会在临时副本中执行，共有 5 项性质测试：

```sh
docdoctor check docs/fences.md docs/metadata.md
```

在没有安装命令的环境中，也可以从本仓库运行 `cargo run -- check docs/fences.md docs/metadata.md`。

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

- `rust docdoctor`：标记一个测试的反引号围栏代码块；普通 Rust 代码块不会执行。
- `file=`：可暂时省略。接线后指向所选 manifest 目录内某个 package 的 `src/` 下已有的 `.rs` 模块文件，不能包含 `..` 或链接；例如 `src/schedule_tree.rs` 对应 `crate::schedule_tree`。选择 workspace 根 manifest 时，成员路径形如 `crates/api/src/lib.rs`。
- `test=`：必填，代码块中无参数函数的名字（ASCII Rust 标识符），例如 `groups_alternate`。工具生成 `#[test]` 包装函数并调用它；代码块不必自行添加 `#[test]`。

若代码上下文尚未存在，围栏头可以先写成 `rust docdoctor test=groups_alternate`，暂不填写 `file=`。检查时将按文档路径、围栏行号与 `doc::<id>::<test>` 在标准错误输出中提示未接线测试，跳过执行；它不计入通过数量。全部未接线时报告 `0 document properties passed`，不运行 Cargo 测试。填写 `file=` 后若目标文件不存在，则检查失败并报告缺失路径，而不是把它当作未接线测试。

工具把每个已接线文件的代码块追加到临时副本的 `#[cfg(test)] mod doc`，并按文档 `id` 创建子模块。因此例子能访问所在文件模块的私有成员，也可用 `crate::...` 引用其他模块。上述测试运行在 `crate::schedule_tree::doc::scheduling::groups_alternate`。工具先检查 Cargo 是否发现每个生成的测试，再调用 `cargo test --all-targets` 执行 `doc::` 过滤出的测试。原始源码和文档不被改写；临时副本及其构建产物在执行后清理。

通过时 Cargo 输出稳定的 `doc::<id>::<test>` 名称；失败时 docdoctor 保留 Cargo 的断言输出，并额外打印 `docs/schedule-tree.md:31: property test doc::scheduling::branch_cursors_are_independent failed`。行号是文档代码围栏的起始行；Cargo 的 `src/...:行号` 则指向临时副本里的注入代码。

工具复制 workspace 根目录内容到临时目录后执行 Cargo；不支持复制范围内的符号链接或特殊文件，以及 workspace 根目录外的成员和相对路径依赖。需要 `cargo` 可用。大型 workspace 的文件复制有额外开销。
