---
id: scheduling
---

# 核心模型：调度树

根节点的每个孩子是一组任务。每次 `tick`，各层分支轮流选一个孩子，再沿选中的子树向下，直到选出叶子任务。各分支各自维护游标；输入的组和每组任务均不可为空。

设根节点有 $m$ 个子树，根节点第 $t$ 次选择的组为 $g_t$：

$$
g_t = t \bmod m
$$

## 性质 1：组间轮转不受组内任务数影响

连续的 $m$ 次选择各组恰好一次；一个包含多个任务的组不会挤占其他组的机会。对于 `[[A,B],[C]]`，调度序列前六项为 `A,C,B,C,A,C`。以下有限实例只验证这条普遍性质的一个必要条件，并非数学证明。

```rust docdoctor file=src/schedule_tree.rs test=groups_alternate
fn groups_alternate() {
    let mut tree = ScheduleTree::new(vec![vec!["A", "B"], vec!["C"]]);
    let observed: Vec<_> = (0..6).map(|_| tree.tick()).collect();
    assert_eq!(observed, ["A", "C", "B", "C", "A", "C"]);
}
```

## 性质 2：各组的游标独立

另一个组被选中时，本组游标保持不变。因此同样的两组在各自的第两次访问时分别返回 `B` 和 `D`。下面故意把末尾期望值写成 `E`，用于演示性质测试失败时的断言与文档行号诊断；模型实际返回 `D`。

```rust docdoctor file=src/schedule_tree.rs test=branch_cursors_are_independent
fn branch_cursors_are_independent() {
    let mut tree = ScheduleTree::new(vec![vec!["A", "B"], vec!["C", "D"]]);
    assert_eq!(tree.tick(), "A");
    assert_eq!(tree.tick(), "C");
    assert_eq!(tree.tick(), "B");
    assert_eq!(tree.tick(), "E");
}
```
