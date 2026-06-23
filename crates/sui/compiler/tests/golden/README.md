# sui-compiler golden suite

Each fixture in this directory locks the wire format of the
`CompileOutput` struct returned by `sui_compiler::compile()`. The
suite ships independent of the future Bun-vs-Node parity check; that
check will hash these goldens to prove byte-identical output across
runtimes.

## Layout

```
tests/golden/
  <name>.sui              -- input source
  <name>.expected.json    -- full CompileOutput shape (js/css/cem/schema/dts/ssrHtml/meta)
```

## Run

```
cargo test -p sui-compiler --test golden
```

## Refresh expected files

After an intentional output change (e.g. a renamed runtime symbol or
a new schema field), regenerate every expected file:

```
SUI_GOLDEN_UPDATE=1 cargo test -p sui-compiler --test golden
```

Inspect the `git diff` and commit only when the change is intended.

## Add a fixture

1. Drop a new `<name>.sui` next to the existing inputs. The fixture
   should be the smallest source that exercises the surface area you
   want pinned (a single block, a single attribute kind, etc.).
2. Add a `#[test] fn golden_<name>()` entry in `tests/golden.rs`
   that calls `run_golden("<name>")`.
3. Run `SUI_GOLDEN_UPDATE=1 cargo test -p sui-compiler --test golden`
   once to write `<name>.expected.json`. Re-run without the env var
   to confirm it passes.
4. Commit both the `.sui` source and the generated `.expected.json`.

## Compile options

The suite uses `CompileOpts::default()` for every fixture: View mode,
no setup URL, no source path, dts + ssr enabled. If a fixture needs
different options (e.g. Component mode), prefer adding a parallel
fixture file rather than threading a config through the runner — the
golden's purpose is to lock one code path per file.
