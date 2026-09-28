---
id: shared_properties
---

# 共享成员性质

与 API 的文档一起传入，可验证根 manifest 选择多个成员时分别注入并发现测试。

```rust docdoctor file=crates/shared/src/lib.rs test=provides_base
fn provides_base() {
    assert_eq!(base(), 40);
}
```
