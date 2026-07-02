// paired.eval.ts — SPIKE: the eval/tasks/paired task wrapped in Sentry's
// vitest-evals (describeEval + createHarness + createJudge).
//
// The harness does NOT dispatch an agent — that's the expensive part of our
// eval. It replays a RECORDED lane (a canned worktree snapshot under
// eval/fixtures/recorded-runs/paired/<laneId>/) and returns the produced view
// files as the run output, exactly the surface grade.ts reads off a real lane
// worktree. The judges are our existing deterministic graders
// (template-purity.ts + primitive-adherence.ts) lifted into vitest-evals'
// Judge shape unchanged.
//
// Run it:
//   bun x vitest run --config vitest.evals.config.ts
//   SUI_EVAL_GATE=1 bun x vitest run --config vitest.evals.config.ts   # gate the adhoc lane too
//   VITEST_EVALS_REPORT_LEVEL=info ... # per-tool/output detail in the reporter

import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs'
import { dirname, join, relative } from 'node:path'
import { fileURLToPath } from 'node:url'
import { expect } from 'vitest'
import { createHarness, createJudge, describeEval } from 'vitest-evals'
import { gradePrimitiveAdherence } from '../eval/scripts/primitive-adherence'
import { scanTemplatePurity } from '../eval/scripts/template-purity'

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..')
const RECORDED_RUNS = join(ROOT, 'eval/fixtures/recorded-runs/paired')
const VIEW_PATH = 'views/space-editor' // what eval/tasks/paired/task-spec.md asks for

// --- shapes -----------------------------------------------------------------
// TOutput must be JSON-safe (vitest-evals normalizes it into the HarnessRun
// that reporters + judges consume). Type aliases (not interfaces) so they
// structurally satisfy JsonValue's index signature.
type LaneInput = { laneId: string }
type LaneFile = { file: string; text: string }
type LaneOutput = { laneId: string; viewPath: string; files: LaneFile[] }

function collectFiles(dir: string, base: string): LaneFile[] {
  if (!existsSync(dir)) return []
  const out: LaneFile[] = []
  for (const entry of readdirSync(dir)) {
    const p = join(dir, entry)
    if (statSync(p).isDirectory()) out.push(...collectFiles(p, base))
    else out.push({ file: relative(base, p), text: readFileSync(p, 'utf8') })
  }
  return out
}

// --- harness: replay a recorded lane -----------------------------------------
// Our real "system under test" is a dispatched agent in a worktree; its
// durable output is the files it left on disk. The harness's job here is only
// to present that recorded output in vitest-evals' normalized shape. The
// transcript is synthesized (2 messages) because our lanes don't record a
// chat transcript — see SPIKE-vitest-evals.md for what that costs us.
const recordedPairedLane = createHarness<LaneInput, LaneOutput>({
  name: 'recorded-paired-lane',
  run: ({ input, setArtifact }) => {
    const laneDir = join(RECORDED_RUNS, input.laneId)
    if (!existsSync(laneDir)) {
      throw new Error(`no recorded lane at ${relative(ROOT, laneDir)} — known lanes: ${readdirSync(RECORDED_RUNS).join(', ')}`)
    }
    const files = collectFiles(join(laneDir, VIEW_PATH), laneDir)
    setArtifact('laneDir', relative(ROOT, laneDir))
    setArtifact('viewFiles', files.map((f) => f.file))
    return {
      output: { laneId: input.laneId, viewPath: VIEW_PATH, files },
      events: [
        {
          type: 'message',
          role: 'user',
          content: `Build ${VIEW_PATH} per eval/tasks/paired/task-spec.md (recorded lane: ${input.laneId})`,
        },
        {
          type: 'message',
          role: 'assistant',
          content: `recorded ${files.length} file(s): ${files.map((f) => f.file).join(', ')}`,
        },
      ],
      usage: {}, // recorded lanes carry no token accounting (yet)
    }
  },
})

// --- judges: our existing deterministic graders, unchanged ------------------

/** Template purity is a hard line: any capability/statement in a `.sui` → 0. */
const TemplatePurityJudge = createJudge<LaneInput, LaneOutput>(
  'TemplatePurity',
  ({ output }) => {
    const violations = output.files
      .filter((f) => f.file.endsWith('.sui'))
      .flatMap((f) => scanTemplatePurity(f.text).map((v) => `${f.file}: ${v}`))
    return {
      score: violations.length === 0 ? 1 : 0,
      metadata: {
        rationale:
          violations.length === 0
            ? 'every template expression is a pure function of scope signals'
            : `${violations.length} violation(s) — ${violations.join(' | ')}`,
        violations,
      },
    }
  },
)

/** Continuous [0,1] composition score straight from primitive-adherence.ts. */
const PrimitiveAdherenceJudge = createJudge<LaneInput, LaneOutput>(
  'PrimitiveAdherence',
  ({ output }) => {
    const report = gradePrimitiveAdherence(
      output.files.filter((f) => f.file.endsWith('.sui')),
    )
    return {
      score: report.overall,
      metadata: {
        rationale: report.perFile
          .map(
            (f) =>
              `${f.file}=${f.score.toFixed(2)} (+prim=${f.primitiveUses} +tok=${f.tokenRefs} -layout=${f.adHocLayout} -literal=${f.literalValues})`,
          )
          .join('; '),
        perFile: report.perFile.map((f) => ({ ...f })),
      },
    }
  },
)

const JUDGES = [TemplatePurityJudge, PrimitiveAdherenceJudge]

// --- suites ------------------------------------------------------------------

// Gated lane: automatic judges run after every run(); avg score must clear
// judgeThreshold or the test FAILS (vitest-evals asserts inside the fixture).
describeEval(
  'paired task — recorded composed lane (gated)',
  { harness: recordedPairedLane, judges: JUDGES, judgeThreshold: 0.9 },
  (it) => {
    it('lane-composed clears the SUI composition bar', async ({ run }) => {
      const result = await run({ laneId: 'lane-composed' })

      // plain vitest assertions on the app-facing output still work
      expect(result.output.files.map((f) => f.file)).toContain('views/space-editor/page.sui')
      expect(result.output.files.map((f) => f.file)).toContain('views/space-editor/manifest.jsonc')

      // explicit judge matcher (in addition to the automatic suite judges)
      await expect(result).toSatisfyJudge(PrimitiveAdherenceJudge, { threshold: 0.8 })
    })
  },
)

// Measurement lane: judgeThreshold: null records scores WITHOUT failing —
// this is grade.ts's "grade is a measurement, not a gate" stance, natively.
// Set SUI_EVAL_GATE=1 to flip it into a gate and watch fail-on-score work.
const GATE = process.env.SUI_EVAL_GATE === '1'
describeEval(
  `paired task — recorded adhoc lane (${GATE ? 'gated' : 'measurement, no gate'})`,
  { harness: recordedPairedLane, judges: JUDGES, judgeThreshold: GATE ? 0.9 : null },
  (it) => {
    it('lane-adhoc records low scores for the hand-rolled shape', async ({ run }) => {
      const result = await run({ laneId: 'lane-adhoc' })
      expect(result.output.files.length).toBeGreaterThan(0)
    })
  },
)
