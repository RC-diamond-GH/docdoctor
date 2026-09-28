---
id: api_properties
---

# 跨成员性质

在 workspace 根 manifest 下，`file=` 指向成员的源码。API 通过 workspace 依赖使用 shared。

```rust docdoctor file=crates/api/src/lib.rs test=uses_shared_base
fn uses_shared_base() {
    assert_eq!(answer(), 42);
}
```
