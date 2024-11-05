
```bash
cargo run
cargo run --features grapheme
```

---

The following can be similar to the `#ifdef` in C++ and is used for conditional compilation, allowing to compile code blocks based on specific conditions or features.

```rust
#[cfg(feature = "grapheme")]
```
