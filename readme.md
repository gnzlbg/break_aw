Build with:

```
RUSTFLAGS="-C panic=abort -C debuginfo=0 -C lto=fat -C opt-level=3 -C codegen-units=1" cargo run --bin break_aw --release
```

on MacosX. On other systems you might need to add an `extern "C" {}` to the `bin.rs` that links the C library.

Add `--emit=llvm-ir` to instead the IR. 
