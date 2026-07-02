# SPIKE: Sentry vitest-evals for the SUI eval harness

**Question:** do `describeEval` + judges + `HarnessRun` normalization + REPLAY +
the reporter earn adoption for our eval (`eval/`)?

**Verdict: ADAPT** — adopt `describeEval` + judges + the reporter as the
*grading & reporting* layer over recorded lane output; keep our dispatch /
lane-watch / jsonl / record.ts pipeline as the *run & persistence* layer.
Reject `HarnessRun` as the canonical record and reject the replay module
(wrong axis for us). Details and costs below.

**Spike artifacts** (all on `job/evals-spike`):

- `evals/paired.eval.ts` — the `eval/tasks/paired` task wrapped in
  `describeEval`, with `template-purity.ts` + `primitive-adherence.ts` as
  judges, running against recorded lane snapshots
- `eval/fixtures/recorded-runs/paired/{lane-composed,lane-adhoc}/` — canned
  worktree snapshots standing in for two finished lanes
- `vitest.evals.config.ts` — eval-only vitest config wiring
  `vitest-evals/reporter` + the JSON artifact
- run: `bun x vitest run --config vitest.evals.config.ts`
  (`SUI_EVAL_GATE=1` flips the adhoc suite from measurement to gate)

Grounded in the installed package (`vitest-evals@0.14.0`, vitest `4.1.9`,
via Verdaccio) — **not** the docs site, which drifts (see below).

---

## What the installed API actually is (0.14.0)

The web-visible examples (`describeEval({ data, task, scorers, threshold })`)
are the **legacy** API, now exiled to `vitest-evals/legacy` with a "temporary
compatibility" label. The real 0.14 model is harness-first:

- `describeEval(name, { harness, judges?, judgeHarness?, judgeThreshold?, skipIf? }, (it) => …)`
  binds exactly one `Harness` to a suite; the callback gets a fixture-backed
  `it` whose context exposes `run(input)`.
- `createHarness<TInput, TOutput>({ name, run })` — `run` executes the system
  under test and returns a lightweight result (`{ output, events|messages,
  usage?, artifacts?, traces?, errors? }`) that gets normalized into a
  `HarnessRun`. **At least one transcript event is required.**
- `createJudge<TInput, TOutput>(name, assess)` — `assess(ctx)` receives
  `{ input, output, toolCalls, run, session, harness, runJudge? }` and returns
  `{ score: number|null, metadata?: { rationale?, … } }`.
- Automatic suite `judges` run after every `run(...)`; their **uniform
  average** is compared to `judgeThreshold` (default **1**; `null` disables
  fail-on-score). There's also an explicit matcher:
  `await expect(result).toSatisfyJudge(judge, { threshold })`.
- Reporter: `vitest-evals/reporter` (extends Vitest's VerboseReporter);
  JSON artifact via Vitest's own `json` reporter — scores and the full
  normalized run ride in each test's `meta`.
- CLI: `vitest-evals serve <results.json>` serves a local React report UI
  over the same JSON.

## What worked

**Our deterministic graders slot in as judges nearly unchanged.** Each is a
~25-line adapter around the existing exported functions — no changes to
`template-purity.ts` or `primitive-adherence.ts`:

```ts
const PrimitiveAdherenceJudge = createJudge<LaneInput, LaneOutput>(
  'PrimitiveAdherence',
  ({ output }) => {
    const report = gradePrimitiveAdherence(output.files.filter(f => f.file.endsWith('.sui')))
    return { score: report.overall, metadata: { rationale: …, perFile: … } }
  },
)
```

**Fixture-backed "recorded run" harness is trivial.** ~40 lines to read a
canned lane worktree and present it as the run output. Judges re-run against
the snapshot in milliseconds — re-judging without re-dispatching, which is the
property we care about given lanes cost 20–40 min of agent time.

**`judgeThreshold: null` is exactly grade.ts's "measurement, not a gate"
stance, natively.** One suite gates the composed lane at 0.9; the adhoc suite
records 0.00 scores without failing. Flip one option and the same suite becomes
a gate with a clean fail-on-score (`Score: 0.00 below threshold: 0.90`).

**The reporter is genuinely good.** Compact per-test score lines on pass;
rationale surfaced inline on fail (no stack-trace noise — it prints the judge's
rationale instead). Gated run output:

```
 ✓ evals/paired.eval.ts > paired task — recorded composed lane (gated) > lane-composed clears the SUI composition bar 17ms
   score   TemplatePurity 1.00
   score   PrimitiveAdherence 1.00
   score   PrimitiveAdherence 1.00
 × evals/paired.eval.ts > paired task — recorded adhoc lane (gated) > lane-adhoc records low scores for the hand-rolled shape 8ms
   score   TemplatePurity 0.00 · 3 violation(s) — views/space-editor/page.sui: Date.now() — non-determinism; seed in the controller | … declaration in a template … | … in-place mutation …
   score   PrimitiveAdherence 0.00 · views/space-editor/page.sui=0.00 (+prim=0 +tok=0 -layout=4 -literal=18)

 Test Files  1 failed (1)
      Tests  1 failed | 1 passed (2)
```

(exit code 1 gated / 0 in measurement mode — correct for CI both ways.)

**The JSON artifact preserves everything.** `.artifacts/vitest-evals/results.json`
is standard Vitest JSON where each assertion carries
`meta.eval.scores[]` (name, score, full metadata incl. our per-file adherence
breakdowns) and `meta.harness.run` (output, session, usage, artifacts, traces,
errors). Our per-lane facts (`laneDir`, `viewFiles`) rode along via
`setArtifact(...)`. `bun x vitest-evals serve` gives a drill-in UI over the
same file for free.

**Strict `tsc` passes over the eval file** — the generics
(`createHarness<LaneInput, LaneOutput>`, `toSatisfyJudge`) hold up.

## What fought us

1. **Docs drift.** The site examples are the legacy scorer API; the installed
   package deprecates it. Ground any adoption work in `dist/*.d.ts`.
2. **`HarnessRun` is chat-shaped; our runs are worktree-shaped.** The
   normalizer *requires* ≥1 transcript event (`message`/`tool_call`/
   `tool_result`), and `usage` wants token accounting. Our lanes produce
   files on disk and (today) no transcript or token counts — we synthesized a
   fake 2-message transcript to satisfy the contract. Harmless, but it's
   ceremony encoding a worldview mismatch: vitest-evals assumes the harness
   *executes an agent in-process*; ours finished hours ago in tmux.
3. **Default threshold is 1.** A suite with judges and no explicit
   `judgeThreshold` fails on any avg score < 1.0. Must pass `null`
   deliberately for measurement mode — easy to get bitten.
4. **Uniform judge averaging vs. our weighted composite.** `avgScore` weights
   every judge equally; grade.ts's composite is 0.30 compile / 0.10 lint /
   0.20 structural / 0.40 behavior. To keep our composite we'd add a single
   `CompositeJudge` that recomputes the weighted sum (and then the other
   judges must be score-only / threshold-exempt, i.e. thresholding moves into
   the composite judge). Workable, but the framework doesn't help.
5. **Type-alias trap:** `TOutput extends JsonValue` — `interface` outputs
   don't satisfy the index-signature constraint; `type` aliases do.
6. **Cosmetic:** a judge that runs both automatically and via
   `toSatisfyJudge` appears twice in the score list (see the double
   `PrimitiveAdherence 1.00` above).
7. **Footprint:** 13 packages (`@vitest-evals/core`, `report-ui`, `zod`, …).
   Peer range `vitest >=4 <5` matches our 4.1.9 today; it's one more coupling
   to our vitest major.

## REPLAY: not what the name suggests (for us)

`vitest-evals/replay` is a **per-tool VCR**: `executeWithReplay(...)` hashes
tool args, records `{ input, output|error }` JSON under
`.vitest-evals/recordings/<tool>/<hash>.json`, and replays on later runs
(`VITEST_EVALS_REPLAY_MODE=off|auto|strict|record`). It exists so an *agent
harness executing inside the test* can re-run cheaply with its tool I/O
canned.

It is **not** "re-judge a recorded eval run." Our lanes don't execute inside
vitest and don't route tool calls through a JS harness, so there is nothing
for this VCR to intercept. The property we actually want — judges re-scored
against a finished run without re-dispatching — is what the fixture-backed
harness in this spike does, and it's ~40 lines we'd write under any framework.
Reject this module; lose nothing.

## Does `HarnessRun` fit our `record.ts` shape?

Partially. Mapping our jsonl row (`LaneRow`) onto `HarnessRun`:

| ours (jsonl / record.ts)                  | HarnessRun            | fit |
| ----------------------------------------- | --------------------- | --- |
| `composite`, per-axis scores               | `meta.eval.scores[]` + `avgScore` | good, but avg ≠ weighted composite (see above) |
| `structural.*` breakdowns                  | judge `metadata`      | good |
| `worktree`, `viewFiles`                    | `artifacts`           | good |
| `compile.stderr`, discovery gaps           | `errors[]`            | good |
| lane transcript                            | `session.events`      | **we don't have one** — synthesized |
| token usage                                | `usage`               | **we don't have it** — empty |
| `runId`/`laneId` keying for collections    | nowhere first-class   | rode in `input`/`artifacts` |

So: `HarnessRun` can *carry* our record but doesn't *improve* it. The durable
record should stay our jsonl (purpose-shaped, `record.ts` → ship collections
unchanged); a thin adapter can synthesize `HarnessRun` at the vitest boundary
when we want the reporter/UI. Don't make `HarnessRun` canonical.

## Recommendation: ADAPT — and the concrete cut line

**Keep (run layer, untouched):** `dispatch-eval.sh`, `lane-watch.sh`, worktree
lifecycle, jsonl artifacts, `record.ts` → collections, `views/eval`.

**Adopt (grade/report layer):** an `evals/` suite where the harness *reads a
finished lane worktree* (reuse grade.ts's `git worktree list` discovery, ~50
lines) and the judges are our graders:

- `TemplatePurityJudge`, `PrimitiveAdherenceJudge` — done in this spike (~120
  lines total incl. harness)
- `CompileJudge` / `LintJudge` — spawn the prebuilt `sui-compiler` inside an
  async judge; deterministic (~60 lines)
- `BehaviorJudge` — wrap the existing `behavior-grade.ts` subprocess (it
  already spawns vitest itself, so it nests cleanly) (~30 lines)
- `CompositeJudge` — recompute the weighted composite; make it the only
  thresholded axis (~30 lines)
- adapter: test `meta` → our jsonl row shape so `record.ts` keeps working, or
  keep emitting jsonl from the judges directly (~60 lines)

**Estimated migration: roughly a day** for parity with `grade.ts`, additive
(grade.ts stays until the eval suite has run a few live cohorts).

**Reject:** replay module (wrong axis), `HarnessRun` as canonical record,
first-party harness packages (`harness-pi-ai` etc. — our SUT isn't an
in-process agent).

**The honest cost/benefit:** with only deterministic graders, adoption buys a
nicer terminal reporter, a browsable report UI, standard JSON artifacts, and
per-judge threshold plumbing — nice, not transformative; grade.ts already
works. The argument that tips it to ADAPT is **LLM judges**: the moment we add
a judged axis that deterministic regexes can't score (visual/spec fidelity,
"does this read as one system"), vitest-evals' `judgeHarness` / `ctx.runJudge`
machinery, score aggregation, and reporting are exactly the infrastructure
we'd otherwise hand-roll. Landing the eval suite now, on recorded lanes, makes
that door cheap to walk through.
