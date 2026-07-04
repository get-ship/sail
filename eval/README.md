
# SUI authoring eval

Measures how well an agent config authors a SUI view from a spec. One lane per
model. No pipeline, no daemon fanout — `dispatch.sh` spawns each lane in its own
worktree + wt tab, a poll loop waits for them to commit, then `grade.ts` scores
each worktree and `record.ts` makes the scores queryable in `views/eval`.

Why off-daemon: the daemon fanout path is unreliable (supervisor sweep murders
live lanes — #189 recurred; installed binary hardcodes :9315). dispatch.sh lanes
survive daemon hiccups and are fully observable in a real PTY.

## Files

- `scripts/dispatch-eval.sh` — fork N lanes (one per model) from the current branch
- `scripts/lane-watch.sh` — peer into each lane's tmux pane, detect idle, auto-grade
- `scripts/grade.ts` — compile + lint + structural rubric → `.artifacts/sui-eval-runs/<run-id>.jsonl`
- `scripts/record.ts` — jsonl → `collections.create_item` into `sui-eval-runs`
- `task-spec.md` + `mockups/` — the authoring task the lanes build

## Run it

**1. Dispatch.** Haiku-only is the default (cheap smoke). Add models for a live run:

```bash
bash eval/scripts/dispatch-eval.sh                       # haiku
bash eval/scripts/dispatch-eval.sh --models haiku,sonnet # live A/B
```

Note the printed `RUN_ID`. Lanes open as wt tabs, each in a tmux session named
after its slug (`eval-<model>-<RUN_ID>`), and work autonomously off `task-spec.md`.

**2. Watch + grade — one command, no commit race.** Completion is detected by
peering into the tmux pane, NOT by waiting for a commit. Agents routinely finish
the work and never commit; a commit-based watcher reads that as "no work" and the
cohort gets cleaned up. The pane is ground truth — when Claude Code stops printing
and its "esc to interrupt" indicator is gone, the lane is done:

```bash
bash eval/scripts/lane-watch.sh <RUN_ID>
```

It blocks until every lane is idle (120s quiet + no spinner) or the 40m timeout,
then grades each worktree and records — no external `/loop`, no `git` dependency.
Run it in a tmux pane of your own (or background it) so it survives. Pass
`--no-grade` to only signal (writes `.artifacts/sui-eval-runs/.watch/<RUN_ID>/<slug>.done`).

If a lane goes idle with **uncommitted work** on disk — the case that burns you
downstream (idle agent, real work, no commit → cleanup assumes nothing happened) —
the watcher `send-keys` a nudge to commit and gives it another round (up to 2
prods) before grading the dirty tree anyway. Grading reads files from disk, so an
uncommitted lane still scores; the prod is about leaving a clean artifact, not
about the score.

A lane that's idle but doesn't compile still gets graded — grade is a measurement,
not a gate. Low score is signal, not error.

**3. Read results** in `views/eval` (grouped by run_id) or the raw jsonl under
`.artifacts/sui-eval-runs/`.

### Manual / headless grade (override the task)

`grade.ts` discovers lanes via `git worktree list` by `-haiku`/`-sonnet`/`-opus`
suffix; with zero lane worktrees it falls back to the cwd as one anonymous lane,
so the whole grading pipeline runs headless against any candidate on disk — no
dispatch, no tmux, no daemon. Compile/lint use the prebuilt
`target/debug/sui-compiler` (override with `SUI_COMPILER=`), never `cargo run`.

```bash
SHIP_PIPELINE_RUN_ID=<run-id> \
SUI_EVAL_VIEW_PATH=views/authority \
SUI_EVAL_REQUIRED_FRAGMENTS='{#if,{#for,key={,on:click,{#while-loading,{#while-error' \
SUI_EVAL_BEHAVIOR_SPEC=eval/tasks/authority/authority.behavior.jsonc \
bun eval/scripts/grade.ts
```

`SUI_EVAL_BEHAVIOR_SPEC` adds the behavioral dimension: `behavior-grade.ts` runs
the spec's scenarios against the candidate in jsdom (vitest + the production WASM
compiler via `@ship/test-setup`'s `runScenarios`) and the composite reweights to
compile 0.30 / lint 0.10 / structural 0.20 / behavior 0.40. Without it the legacy
0.40/0.20/0.40 split applies. `behavior-grade.ts` also runs standalone:

```bash
bun eval/scripts/behavior-grade.ts <lane-dir> <view-path> <spec.behavior.jsonc>
```

### Primitive-adherence (composition axis)

`primitive-adherence.ts` scores whether each `.sui` **composes** the design
system — `<sui-grid>` / `<column>` / `<row>` + other `<sui-*>`
primitives, values spent as `var(--token)` — or **hand-rolls** it (ad-hoc
`display:grid`/`flex` off `:host`, hex/px/raw-font literals). It contributes
0.20 of the structural fraction (see `structuralFraction` in `grade.ts`) and
the overall lands in each jsonl row as `structural.primitiveAdherence` with a
per-file `structural.primitiveAdherenceByFile` breakdown. The formula is
documented at the top of the file. Run it standalone on any file or dir:

```bash
bun eval/scripts/primitive-adherence.ts <file.sui | dir> [...]
# self-check on the bundled fixtures (hand-rolled → 0.00, composed → 1.00):
bun eval/scripts/primitive-adherence.ts eval/scripts/fixtures/primitive-adherence/
bun test eval/scripts/primitive-adherence.test.ts
```

## Cleanup

After scoring, remove lane worktrees + branches:

```bash
for wt in ~/.ship/worktrees/eval-*-<id>; do git worktree remove --force "$wt"; done
git branch | grep "job/eval-.*-<id>" | xargs -r git branch -D
```
