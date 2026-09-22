# Fuzzing the .sail parser

`fuzz/` is a [cargo-fuzz](https://rust-fuzz.github.io/book/cargo-fuzz.html)
harness for `sail_compiler::parse::parse`.

```sh
cargo install cargo-fuzz
rustup toolchain install nightly

# from crates/sail/compiler (or package path that owns fuzz/)
cargo +nightly fuzz run parse_fragment -- -max_total_time=60
```
