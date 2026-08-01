# bench/ — agent lab standards

Committed standards only. Run data never lives here.

## Hard rule

**In this directory (committed):** task definitions, rubrics, reference
assets, context-profile definitions, the run-record schema, setup scripts,
this README.

**Never in the sail repo:** run directories, screenshots, agent-authored
code, trajectories, event logs, binaries, sqlite files, run-by-run data of
any kind.

The lab lives outside the repo at `~/dev/sail-lab/`. After any batch,
`git status` in the sail repo must stay clean of run artifacts.

(`eval/` from EVAL-0 is a separate, older corpus — leave it alone.)

## HARNESS-2 (typed path — preferred for new runs)

```bash
cd bench/eval
bun test
bun src/cli/sail-bench.ts run --task parallax-landing --arm sail --toolchain current --seed
bun src/cli/sail-bench.ts report --batch <id>
```

- **Contract:** `bench/eval/CONTRACT.md` (RUN CONTRACT v1)
- **Modules:** config, runner, gates, render, store, report, serve, grade, adapters
- **FS:** `$SAIL_LAB/runs/<toolchain>/<task>/<model>/<word-pair>/`
- **Store:** `$SAIL_LAB/lab-bench.sqlite` (slim; blobs on disk)
- Legacy `lab-kit/` shell/python remains for the existing corpus (read-only /
  migration); do not extend it for new work.

## Layout

```
bench/
  eval/                      # HARNESS-2 typed harness (sail-bench + eval-adapter)
  schema.sql                 # legacy runs + run_events (lab-kit path)
  profiles/*.jsonc           # context profiles (P0 baseline, …)
  agents/agents-v2.md        # AGENTS.md variant referenced by profiles
  tasks/<slug>/
    task.jsonc               # id, prompt, systems, scaffold, grading
    rubric.md                # per-task criteria
    reference/               # optional brand brief / target crop
    overlay/                 # optional scaffold overlay files
  scripts/
    sync-to-lab.sh           # copy standards (+ lab-kit) → ~/dev/sail-lab/
    bootstrap-lab.sh         # materialize harness/payload from lab-kit
    check-repo-clean.sh      # fail if run artifacts leaked into sail repo
  lab-kit/                   # legacy harness + payload (still used by corpus)
    harness/                 # stage/run/harvest/batch scripts
    payload/                 # sail base + react-base templates
  fixtures/
    canned-trajectory/       # events.jsonl + expected.json for harvest replay
```

## Tasks (baseline five)

| slug | system stressed |
|---|---|
| `adjustable-shell` | layout + `--*` geometry channel |
| `parallax-landing` | motion |
| `dependent-form` | state/controller |
| `brand-transplant` | tokens/sx/theming |
| `mini-app-routing` | routing (overlay scaffold) |

## Profiles

- **P0** (`profiles/p0.jsonc`): generated skill + AGENTS.md v2 + stdlib policy + `mcp: false`

## Lab

Harness, toolchains, sandboxes, sqlite, reports: `~/dev/sail-lab/`.
See that tree's README for build/stage/run/harvest/batch commands.

Quick start:

```bash
# once: materialize lab harness from committed lab-kit + sync standards
./bench/scripts/bootstrap-lab.sh

# in the lab
cd ~/dev/sail-lab
./harness/build_toolchain.sh          # stamps sail + sail-mcp + sail-compiler
./harness/prepare_react_base.sh
./harness/replay_test.py              # canned trajectory → expected row
./harness/smoke.sh                    # dependent-form × sail × n=1 (live grok)
./harness/batch.sh                    # 5 tasks × 2 arms × n=3 (long)
```

Harness source of truth is `bench/lab-kit/`; live data stays in `~/dev/sail-lab/`.
If the worktree gitdir is broken, cleanliness is enforced by
`check-repo-clean.sh` (filesystem scan), not `git status`.

## Failure classification taxonomy

`invented-syntax` · `missing-example` · `toolchain-friction` ·
`language-gap` · `strictness-wall` · `premature-victory`

## Known gaps

- Screenshot L1 paint fidelity is incomplete (RENDER-1); capture screenshots
  anyway and treat beauty grades as suspect until that lands.
- MCP arms are deferred (sandbox cannot exec staged MCP binary).
- Beauty grading is human (operator); mechanical grades are automated.
  `beauty_json` stays NULL until graded.
