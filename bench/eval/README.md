# bench/eval — HARNESS-2 typed harness

Typed, documented eval harness for the agent lab. Replaces the shell/python
`lab-kit` path for new runs (legacy lab-kit remains for read-only corpus).

## Split

| Layer | Owner | Location |
|---|---|---|
| **Core** (lifecycle, FS tree, store, report, serve) | fleet shape; interim here | this package (`runner`, `store`, `report`, `serve`) |
| **Evaluation pack** (gates, render, grok hooks, graders) | sail | `adapters/` |
| **Seam** | RUN CONTRACT v1 | `CONTRACT.md`, `eval-adapter <phase>` |

## Quick start

```bash
cd bench/eval

# unit + contract tests
bun test

# seed run (no live agent) — full run dir + db row
bun src/cli/sail-bench.ts run \
  --task parallax-landing \
  --arm sail \
  --toolchain current \
  --seed

# live agent (requires grok + toolchain)
bun src/cli/sail-bench.ts run \
  --task parallax-landing \
  --arm sail \
  --toolchain current \
  --model grok

# report (zero copied bytes — references run dirs)
bun src/cli/sail-bench.ts report --batch <batch_id>

# grade
bun src/cli/sail-bench.ts grade <run_id> beauty=3 note="parallax works"

# serve surfaces
bun src/cli/sail-bench.ts serve
```

Env:

| var | default | meaning |
|---|---|---|
| `SAIL_LAB` | `~/dev/sail-lab` | lab root (runs/, lab-bench.sqlite, reports/) |
| `SAIL_REPO` | repo root | standards + binaries |
| `SAIL_BIN` | `target/release|debug/sail` | sail CLI for gates |
| `CHROME_PATH` | playwright cache | chromium for render |
| `SAIL_BENCH_MODEL` | _(required for live)_ | model slug |

## FS tree

```
$SAIL_LAB/
  lab-bench.sqlite          # slim indexes only
  runs/<toolchain>/<task>/<model>/<word-pair>/
    manifest.json
    events.jsonl
    gate.json
    render.png
    sandbox/…               # agent source
  reports/<batch_id>/
    index.html              # generated; references, never copies
    batch.json
  quarantine/               # fixtures / aborted runs
```

## Modules

- `contract` — versioned manifest/result types + adapter spawn
- `config` — Task / Profile / Toolchain loaders + content hashes
- `naming` — word-pair run ids, path layout
- `store` — bun:sqlite, schema migrate, grades API
- `runner` — LocalRunner (prepare → execute → gate → render → collect)
- `adapters` — prepare / gate / render / collect phase handlers
- `report` — batch/task indexes (reference only)
- `serve` — scoped static server
- `grade` — CLI grade write path
- `cli` — `sail-bench`, `eval-adapter`

## Acceptance (phase 1)

- `sail-bench run --task parallax-landing --arm sail --toolchain current` produces
  complete run dir + db row
- `sail-bench report --batch <id>` regenerates index with zero copied bytes
- No shell-out to python on the new path
