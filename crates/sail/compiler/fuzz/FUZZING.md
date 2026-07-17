# Fuzzing the .sail parser

`fuzz/` hosts a [cargo-fuzz](https://rust-fuzz.github.io/book/cargo-fuzz.html)
harness for `sail_compiler::parse::parse`. The bug class to catch is the
infinite-loop / unbounded-allocation family — exactly what the May 2026
"stray `<` operator inside a template expression" hotfix patched at
`parse/html/parser.rs:55`. Any AFL-style fuzzer would have surfaced it in
seconds; this harness makes such regressions automatic.

## Setup (one time)

```sh
cargo install cargo-fuzz
rustup toolchain install nightly   # libfuzzer-sys requires nightly
```

## Running

Quick 60-second smoke run, seeded from every `.sail` in
`packages/components/ui-components/src`:

```sh
just fuzz-parser
```

Longer runs (override the duration in seconds):

```sh
scripts/fuzz/run-parser-fuzz.sh 600        # 10 minutes
scripts/fuzz/run-parser-fuzz.sh 86400      # 24 hours
```

Or invoke `cargo fuzz` directly from `crates/sail-compiler/`:

```sh
cargo +nightly fuzz run parse_fragment -- -max_total_time=600
cargo +nightly fuzz run parse_fragment -- -jobs=8 -workers=8     # parallel
```

## Triage

Crashes land in `fuzz/artifacts/parse_fragment/`. Reproduce a single
input with:

```sh
cargo +nightly fuzz run parse_fragment fuzz/artifacts/parse_fragment/<file>
```

Minimize before filing:

```sh
cargo +nightly fuzz tmin parse_fragment fuzz/artifacts/parse_fragment/<file>
```

## Corpus

`fuzz/corpus/parse_fragment/` is gitignored. The runner script reseeds it
from `packages/components/ui-components/src/**/*.sail` on every invocation so the
fuzzer mutates from real inputs rather than random bytes. Inputs the
fuzzer discovers stay in the corpus dir between runs (until you
`rm -rf fuzz/corpus`).

## Why a separate crate manifest

`fuzz/Cargo.toml` declares its own `[workspace]` table so the parent
workspace at `Cargo.toml` does not try to compile it under stable —
`libfuzzer-sys` requires nightly + `-Zsanitizer`.
