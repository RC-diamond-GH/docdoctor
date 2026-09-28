---
id: missing_target
---

# 指向不存在的目标文件

这里显式给出 `file=`，但该文件尚不存在。检查应报错退出，不能将其当作省略 `file=` 的未接线测试。

```rust docdoctor file=src/not_yet_implemented.rs test=future_property
fn future_property() {
    assert!(FutureModel::is_valid());
}
```
