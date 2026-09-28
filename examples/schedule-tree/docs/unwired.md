---
id: unwired
---

# 尚未接线的调度性质

代码上下文尚不存在时，先写测试并省略 `file=`。检查应提示 `unwired test doc::unwired::empty_groups_are_rejected (missing file=)`，返回成功，但不执行此测试，也不把它计入通过数。

```rust docdoctor test=empty_groups_are_rejected
fn empty_groups_are_rejected() {
    assert!(ScheduleTree::try_new(vec![]).is_err());
}
```
