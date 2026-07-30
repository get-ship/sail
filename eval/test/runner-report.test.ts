import { describe, expect, it } from 'vitest'
import {
  mkdtempSync,
  mkdirSync,
  writeFileSync,
  readFileSync,
  rmSync,
  existsSync,
} from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { classifyRun, runCorpus } from '../src/runner.ts'
import { formatResultsMd } from '../src/report.ts'
import { recordSession } from '../src/recorder.ts'
import type { HostTooling } from '../src/host.ts'
import { RecordedDispatch } from '../src/dispatch/recorded.ts'
import type { CorpusSummary, RunResult } from '../src/types.ts'
import { tasksDir } from '../src/paths.ts'

function mockHost(): HostTooling {
  return {
    sailBin: '/mock/sail',
    async check() {
      return { ok: true, diagnostics: [], autoFixed: [] }
    },
    async inspectTree() {
      return {
        ok: true,
        nodes: [
          { tag: 'column', layer: 'l1-keyword' },
          { tag: 'card', layer: 'l1-element' },
          { tag: 'h1', layer: 'l1-element' },
          { tag: 'button', layer: 'l1-element' },
          { tag: 'input', layer: 'l1-element' },
          { tag: 'label', layer: 'l1-element' },
        ],
      }
    },
    async layout() {
      return { ok: true, findings: [] }
    },
  }
}

function noSailHost(): HostTooling {
  return {
    sailBin: null,
    async check() {
      return { ok: false, diagnostics: [], autoFixed: [], error: 'no sail' }
    },
    async inspectTree() {
      return { ok: false, nodes: [], error: 'no sail' }
    },
    async layout() {
      return { ok: true, findings: [] }
    },
  }
}

describe('runCorpus cheap recorded', () => {
  it('runs full corpus and writes RESULTS.md', async () => {
    const runsRoot = mkdtempSync(join(tmpdir(), 'eval-runs-'))
    const resultsOut = join(runsRoot, 'RESULTS.md')
    try {
      const summary = await runCorpus({
        lane: 'cheap',
        dispatch: 'recorded',
        host: mockHost(),
        adapter: new RecordedDispatch(),
        writeReport: false,
        keepWorkdir: true,
        runsRoot,
        allowSkip: false,
      })
      writeFileSync(resultsOut, formatResultsMd(summary))
      expect(summary.runs.length).toBe(10)
      expect(summary.failCount).toBe(0)
      expect(summary.passCount).toBe(10)
      const md = readFileSync(resultsOut, 'utf8')
      expect(md).toContain('login-form')
      expect(md).toContain('Pass / Fail')
      expect(md).not.toMatch(/\/home\//)
      const loginRun = summary.runs.find((r) => r.task === 'login-form')
      expect(loginRun?.grades.compile?.ok).toBe(true)
      expect(loginRun?.grades.vocab_share?.ok).toBe(true)
      // cheap does not run behavior by default
      expect(loginRun?.grades.behavior?.skipped).toBe(true)
    } finally {
      rmSync(runsRoot, { recursive: true, force: true })
    }
  })

  it('records error when golden missing for a synthetic task root', async () => {
    const tasksRoot = mkdtempSync(join(tmpdir(), 'eval-tasks-'))
    const runsRoot = mkdtempSync(join(tmpdir(), 'eval-runs-'))
    try {
      const slug = 'broken-task'
      mkdirSync(join(tasksRoot, slug), { recursive: true })
      writeFileSync(
        join(tasksRoot, slug, 'task.jsonc'),
        JSON.stringify({
          prompt: 'do something non-trivial for the harness',
          scaffold: 'blank',
          lanes: ['cheap'],
          grading: { compile: true, vocab_share: true, gates: true },
        }),
      )
      const summary = await runCorpus({
        lane: 'cheap',
        dispatch: 'recorded',
        tasks: [slug],
        tasksRoot,
        runsRoot,
        host: mockHost(),
        writeReport: false,
      })
      expect(summary.runs).toHaveLength(1)
      expect(summary.runs[0]!.error).toMatch(/fixture|golden/i)
      expect(summary.failCount).toBe(1)
    } finally {
      rmSync(tasksRoot, { recursive: true, force: true })
      rmSync(runsRoot, { recursive: true, force: true })
    }
  })

  it('sailBin null + allowSkip does not count as pass (skip, not pass)', async () => {
    const runsRoot = mkdtempSync(join(tmpdir(), 'eval-runs-skip-'))
    try {
      const summary = await runCorpus({
        lane: 'cheap',
        dispatch: 'recorded',
        tasks: ['login-form'],
        host: noSailHost(),
        adapter: new RecordedDispatch(),
        writeReport: false,
        runsRoot,
        allowSkip: true,
        enableCheapBehavior: true, // would greenwash if classifyRun ignored core skips
      })
      expect(summary.runs).toHaveLength(1)
      expect(summary.passCount).toBe(0)
      expect(summary.skipCount).toBe(1)
      expect(summary.failCount).toBe(0)
      expect(summary.runs[0]!.grades.compile?.skipped).toBe(true)
    } finally {
      rmSync(runsRoot, { recursive: true, force: true })
    }
  })

  it('sailBin null + strict (allowSkip false) fails', async () => {
    const runsRoot = mkdtempSync(join(tmpdir(), 'eval-runs-strict-'))
    try {
      const summary = await runCorpus({
        lane: 'cheap',
        dispatch: 'recorded',
        tasks: ['login-form'],
        host: noSailHost(),
        adapter: new RecordedDispatch(),
        writeReport: false,
        runsRoot,
        allowSkip: false,
      })
      expect(summary.passCount).toBe(0)
      expect(summary.failCount).toBe(1)
    } finally {
      rmSync(runsRoot, { recursive: true, force: true })
    }
  })
})

describe('classifyRun', () => {
  const base = (grades: RunResult['grades'], lane: RunResult['lane'] = 'cheap'): RunResult => ({
    task: 't',
    lane,
    dispatch: 'recorded',
    model: 'm',
    workdir: '<w>',
    runDir: '<r>',
    trajectory: {
      task: 't',
      model: 'm',
      dispatch: 'recorded',
      turns: [],
      turnCount: 1,
      startedAt: '',
      finishedAt: '',
      filesWritten: [],
    },
    grades,
    startedAt: '',
    finishedAt: '',
  })

  const greenCompile = {
    ok: true as const,
    errorCount: 0,
    warningCount: 0,
    diagnostics: [] as [],
    repairCount: 0,
    iterationsToGreen: 1 as number | null,
  }
  const greenVocab = {
    ok: true as const,
    l1Count: 4,
    primitiveCount: 0,
    share: 1,
    minShare: 0.25,
  }
  const greenGates = {
    ok: true as const,
    lintErrors: 0,
    layoutFindings: 0,
    composeFindings: 0,
    repairCount: 0,
  }
  const greenBehavior = {
    ok: true as const,
    passed: 1,
    failed: 0,
    results: [{ name: 's', passed: true }],
  }

  it('skips when core legs soft-skipped even if grades.ok', () => {
    const r = base({
      ok: true,
      compile: { ...greenCompile, skipped: true, iterationsToGreen: null },
      vocab_share: { ...greenVocab, skipped: true, l1Count: 0, share: 0 },
      gates: { ...greenGates, skipped: true },
      behavior: { ok: true, passed: 1, failed: 0, results: [] },
    })
    expect(classifyRun(r, 'cheap')).toBe('skip')
  })

  it('frontier with incomplete beauty (no score) is fail, not skip', () => {
    const r = base(
      {
        ok: false,
        compile: greenCompile,
        vocab_share: greenVocab,
        gates: greenGates,
        behavior: greenBehavior,
        beauty: {
          ok: false,
          score: null,
          maxScore: 10,
          source: 'skipped',
          skipped: true,
          detail: 'frontier beauty requires judge/operator score',
        },
      },
      'frontier',
    )
    expect(classifyRun(r, 'frontier')).toBe('fail')
  })

  it('frontier + allowSkipBeauty soft beauty + green legs is pass', () => {
    const r = base(
      {
        ok: true,
        compile: greenCompile,
        vocab_share: greenVocab,
        gates: greenGates,
        behavior: greenBehavior,
        beauty: {
          ok: true,
          score: null,
          maxScore: 10,
          source: 'skipped',
          skipped: true,
          detail: 'frontier beauty skipped (--allow-skip-beauty)',
        },
      },
      'frontier',
    )
    expect(classifyRun(r, 'frontier')).toBe('pass')
  })
})

describe('formatResultsMd', () => {
  it('renders aggregates only', () => {
    const summary: CorpusSummary = {
      lane: 'cheap',
      dispatch: 'recorded',
      startedAt: '2026-07-30T00:00:00.000Z',
      finishedAt: '2026-07-30T00:01:00.000Z',
      passCount: 1,
      failCount: 0,
      skipCount: 0,
      runs: [
        {
          task: 'login-form',
          lane: 'cheap',
          dispatch: 'recorded',
          model: 'recorded',
          workdir: '/home/secret/work',
          runDir: '/home/secret/run',
          trajectory: {
            task: 'login-form',
            model: 'recorded',
            dispatch: 'recorded',
            turns: [],
            turnCount: 1,
            startedAt: '',
            finishedAt: '',
            filesWritten: [],
          },
          grades: {
            ok: true,
            compile: {
              ok: true,
              errorCount: 0,
              warningCount: 0,
              diagnostics: [],
              repairCount: 0,
              iterationsToGreen: 1,
            },
            vocab_share: {
              ok: true,
              l1Count: 4,
              primitiveCount: 0,
              share: 1,
              minShare: 0.25,
            },
          },
          startedAt: '',
          finishedAt: '',
        },
      ],
    }
    const md = formatResultsMd(summary)
    expect(md).toContain('| login-form |')
    expect(md).toContain('vocab:100%')
    // Honesty: recorded 100% must not look like live agent success.
    expect(md).toMatch(/UNPROVEN|fixture replay|goldens/i)
    expect(md).not.toMatch(/\/home\//)
  })
})

describe('recorder promote', () => {
  it('snapshots a directory into runs/', async () => {
    const session = mkdtempSync(join(tmpdir(), 'eval-session-'))
    try {
      mkdirSync(join(session, 'routes'), { recursive: true })
      writeFileSync(
        join(session, 'routes', 'page.sail'),
        '<column gap="2"><h1>Recorded</h1></column>\n',
      )
      const result = await recordSession({
        dir: session,
        task: 'recorder-smoke',
        prompt: 'rebuild studio surface',
        promote: false,
      })
      expect(result.task).toBe('recorder-smoke')
      expect(result.runDirDisplay).not.toMatch(/^\/home\//)
      expect(existsSync(join(result.runDir, 'grades.json'))).toBe(true)
      expect(existsSync(join(result.runDir, 'trajectory.jsonl'))).toBe(true)
      expect(existsSync(join(result.runDir, 'files', 'routes', 'page.sail'))).toBe(true)
    } finally {
      rmSync(session, { recursive: true, force: true })
    }
  })

  it('promote replaces golden (stale files do not linger)', async () => {
    const session = mkdtempSync(join(tmpdir(), 'eval-promote-'))
    const taskSlug = 'promote-replace-test'
    const goldenRoot = join(tasksDir(), taskSlug, 'golden')
    try {
      // Seed an old golden with a stale file
      mkdirSync(join(goldenRoot, 'routes'), { recursive: true })
      writeFileSync(join(goldenRoot, 'routes', 'page.sail'), '<h1>Old</h1>\n')
      writeFileSync(join(goldenRoot, 'stale.txt'), 'should not survive promote\n')

      mkdirSync(join(session, 'routes'), { recursive: true })
      writeFileSync(
        join(session, 'routes', 'page.sail'),
        '<column gap="2"><h1>New golden</h1></column>\n',
      )

      const result = await recordSession({
        dir: session,
        task: taskSlug,
        prompt: 'promote replace semantics',
        promote: true,
      })
      expect(result.promotedTo).toBeTruthy()
      expect(existsSync(join(goldenRoot, 'routes', 'page.sail'))).toBe(true)
      expect(readFileSync(join(goldenRoot, 'routes', 'page.sail'), 'utf8')).toContain('New golden')
      expect(existsSync(join(goldenRoot, 'stale.txt'))).toBe(false)
      expect(result.promotedToDisplay).not.toMatch(/^\/home\//)
    } finally {
      rmSync(session, { recursive: true, force: true })
      rmSync(join(tasksDir(), taskSlug), { recursive: true, force: true })
    }
  })
})
