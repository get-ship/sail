# sail-compiler golden suite

Fixtures lock `CompileOutput` wire format from `sail_compiler::compile()`.

```
tests/golden/
  <name>.sail
  <name>.expected.json
```

```sh
cargo test -p sail-compiler --test golden
SAIL_GOLDEN_UPDATE=1 cargo test -p sail-compiler --test golden   # refresh expected
```

Add fixture: drop `<name>.sail`, add `golden_<name>` in `tests/golden.rs`,
refresh with `SAIL_GOLDEN_UPDATE=1`, commit both files.
