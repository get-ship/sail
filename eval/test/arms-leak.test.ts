/**
 * EVAL-1: context-profile leak assertions.
 * Non-sail arms (react-tw, vanilla) must not receive sail artifacts.
 */
import { describe, expect, it } from 'vitest'
import {
  mkdtempSync,
  rmSync,
  writeFileSync,
  mkdirSync,
  existsSync,
  readFileSync,
} from 'node:fs'
import { join } from 'node:path'
import { tmpdir } from 'node:os'
import {
  findSailArtifactLeaks,
  loadArm,
  loadArms,
  listArms,
  materializeArm,
  gradingForArm,
} from '../src/arm.ts'
import {
  parseInterval,
  isTripleComplete,
  findCompletedTriple,
  modelsMatch,
  modelSlug,
  tripleRunDir,
  loadNonSailHtml,
  runPaced,
} from '../src/pace.ts'
import { formatResultsMd } from '../src/report.ts'
import type { CorpusSummary, RunResult } from '../src/types.ts'
import { scaffoldWorkdir } from '../src/scaffold.ts'

describe('arms corpus', () => {
  it('lists sail, react-tw, vanilla', () => {
    const ids = listArms()
    expect(ids).toEqual(expect.arrayContaining(['sail', 'react-tw', 'vanilla']))
    expect(ids).toHaveLength(3)
  })

  it('loads each arm.jsonc with expected context flags', () => {
    const sail = loadArm('sail')
    expect(sail.def.context.sailArtifacts).toBe(true)
    expect(sail.def.context.skillFile).toBe(true)
    expect(sail.def.context.mcp).toBe(true)
    expect(sail.def.scaffold).toBe('blank')

    const rtw = loadArm('react-tw')
    expect(rtw.def.context.sailArtifacts).toBe(false)
    expect(rtw.def.context.skillFile).toBe(false)
    expect(rtw.def.context.mcp).toBe(false)
    expect(rtw.def.scaffold).toBe('react-tw')

    const van = loadArm('vanilla')
    expect(van.def.context.sailArtifacts).toBe(false)
    expect(van.def.scaffold).toBe('vanilla')
  })
})

describe('leak assertion: non-sail arms', () => {
  it('react-tw materialize has zero sail artifacts', () => {
    const dir = mkdtempSync(join(tmpdir(), 'eval-arm-rtw-'))
    try {
      const arm = loadArm('react-tw')
      materializeArm(dir, arm)
      const leaks = findSailArtifactLeaks(dir)
      expect(leaks).toEqual([])
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })

  it('vanilla materialize has zero sail artifacts', () => {
    const dir = mkdtempSync(join(tmpdir(), 'eval-arm-van-'))
    try {
      const arm = loadArm('vanilla')
      materializeArm(dir, arm)
      const leaks = findSailArtifactLeaks(dir)
      expect(leaks).toEqual([])
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })

  it('sail arm intentionally has sail artifacts', () => {
    const dir = mkdtempSync(join(tmpdir(), 'eval-arm-sail-'))
    try {
      const arm = loadArm('sail')
      materializeArm(dir, arm)
      const leaks = findSailArtifactLeaks(dir)
      // Sail arm is allowed to have these — "leaks" detector still finds them.
      expect(leaks.some((l) => l.endsWith('.sail') || l.includes('page.sail'))).toBe(
        true,
      )
      expect(leaks.some((l) => l.includes('sail.config') || l.includes('SAIL_SKILL'))).toBe(
        true,
      )
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })

  it('detector catches injected .sail on a control arm', () => {
    const dir = mkdtempSync(join(tmpdir(), 'eval-arm-leak-'))
    try {
      const arm = loadArm('vanilla')
      materializeArm(dir, arm)
      writeFileSync(join(dir, 'sneaky.sail'), '<column>x</column>\n')
      const leaks = findSailArtifactLeaks(dir)
      expect(leaks.some((l) => l.includes('sneaky.sail'))).toBe(true)
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })

  it('detector catches sail MCP config on a control arm', () => {
    const dir = mkdtempSync(join(tmpdir(), 'eval-arm-mcp-'))
    try {
      const arm = loadArm('react-tw')
      materializeArm(dir, arm)
      writeFileSync(
        join(dir, '.mcp.json'),
        JSON.stringify({ mcpServers: { sail: { command: 'sail-mcp' } } }),
      )
      const leaks = findSailArtifactLeaks(dir)
      expect(leaks.length).toBeGreaterThan(0)
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })
})

describe('gradingForArm', () => {
  it('disables sail legs for control arms', () => {
    const task = {
      compile: true,
      vocab_share: true,
      gates: true,
      behavior: true,
      beauty: true,
    }
    const rtw = loadArm('react-tw')
    const g = gradingForArm(task, rtw.def)
    expect(g.compile).toBe(false)
    expect(g.vocab_share).toBe(false)
    expect(g.gates).toBe(false)
    expect(g.behavior).toBe(true)
    expect(g.beauty).toBe(true)
  })

  it('keeps sail legs for sail arm', () => {
    const task = {
      compile: true,
      vocab_share: { min: 0.25 },
      gates: true,
      behavior: true,
      beauty: true,
    }
    const sail = loadArm('sail')
    const g = gradingForArm(task, sail.def)
    expect(g.compile).toBe(true)
    expect(g.vocab_share).toBeTruthy()
    expect(g.gates).toBe(true)
  })
})

describe('pace helpers', () => {
  it('parseInterval accepts 10m / 30s / 1h / raw ms', () => {
    expect(parseInterval('10m')).toBe(600_000)
    expect(parseInterval('30s')).toBe(30_000)
    expect(parseInterval('1h')).toBe(3_600_000)
    expect(parseInterval('500')).toBe(500)
    expect(parseInterval('500ms')).toBe(500)
  })

  it('parseInterval rejects garbage (no silent default)', () => {
    expect(() => parseInterval('')).toThrow(/invalid --interval/)
    expect(() => parseInterval('ten-minutes')).toThrow(/invalid --interval/)
    expect(() => parseInterval('10x')).toThrow(/invalid --interval/)
  })

  it('modelsMatch is one-way; empty recorded never matches', () => {
    expect(modelsMatch('fuzzy-baseline', '')).toBe(false)
    expect(modelsMatch('', 'fuzzy-baseline')).toBe(false)
    expect(modelsMatch('fuzzy-baseline', 'provider/fuzzy-baseline-v1')).toBe(true)
    expect(modelsMatch('fuzzy-baseline-v1', 'fuzzy-baseline')).toBe(false)
    // Prefix siblings: operator grok-2 must not match recorded grok
    expect(modelsMatch('grok-2', 'grok')).toBe(false)
    // Operator fuzzy substring of resolved label is OK
    expect(modelsMatch('grok', 'xai/grok-3-mini')).toBe(true)
    expect(modelsMatch('grok', 'grok')).toBe(true)
  })

  it('modelSlug / tripleRunDir isolate models on disk', () => {
    const slug = modelSlug('Grok-3 Mini!')
    expect(slug).toMatch(/^grok-3-mini-[a-f0-9]{8}$/)
    // Same raw model → stable slug
    expect(modelSlug('Grok-3 Mini!')).toBe(slug)
    const a = tripleRunDir('/runs', '2026-07-30', 'login-form', 'sail', 'grok')
    const b = tripleRunDir('/runs', '2026-07-30', 'login-form', 'sail', 'grok-2')
    expect(a).not.toBe(b)
    expect(a).toContain('/sail/grok-')
    expect(b).toContain('/sail/grok-2-')
  })

  it('modelSlug distinguishes punctuated synonyms (foo/bar vs foo-bar)', () => {
    expect(modelSlug('foo/bar')).not.toBe(modelSlug('foo-bar'))
    // Both readable base, different hash suffixes
    expect(modelSlug('foo/bar')).toMatch(/^foo-bar-[a-f0-9]{8}$/)
    expect(modelSlug('foo-bar')).toMatch(/^foo-bar-[a-f0-9]{8}$/)
  })

  it('loadNonSailHtml prefers App.jsx over empty Vite index.html shell', () => {
    const dir = mkdtempSync(join(tmpdir(), 'eval-nonsail-html-'))
    try {
      mkdirSync(join(dir, 'src'), { recursive: true })
      writeFileSync(
        join(dir, 'index.html'),
        '<!doctype html><html><body><div id="root"></div></body></html>\n',
      )
      writeFileSync(
        join(dir, 'src', 'App.jsx'),
        `export default function App() {
  return (
    <div>
      <h1>Sign in</h1>
      <label>Email</label>
      <label>Password</label>
      <button type="submit">Sign in</button>
      <a href="/forgot">Forgot password?</a>
    </div>
  )
}
`,
      )
      const html = loadNonSailHtml(dir)
      expect(html).toBeTruthy()
      expect(html).toContain('Sign in')
      expect(html).toContain('Email')
      expect(html).toContain('Forgot password?')
      // Must not return the empty Vite shell alone
      expect(html).not.toMatch(/id="root"/)
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })

  it('loadNonSailHtml falls back to index.html for vanilla', () => {
    const dir = mkdtempSync(join(tmpdir(), 'eval-nonsail-van-'))
    try {
      writeFileSync(
        join(dir, 'index.html'),
        '<!doctype html><html><body><h1>Sign in</h1></body></html>\n',
      )
      const html = loadNonSailHtml(dir)
      expect(html).toContain('Sign in')
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })

  it('isTripleComplete / findCompletedTriple resume-safe', () => {
    const root = mkdtempSync(join(tmpdir(), 'eval-pace-resume-'))
    try {
      const date = '2026-07-30'
      const task = 'login-form'
      const arm = 'sail'
      // Operator fuzzy is path key; recorded may be a longer resolved label.
      const operator = 'fuzzy-baseline'
      const recorded = 'provider/fuzzy-baseline-v1'
      const runDir = tripleRunDir(root, date, task, arm, operator)
      mkdirSync(runDir, { recursive: true })
      writeFileSync(join(runDir, 'grades.json'), JSON.stringify({ ok: true }))
      writeFileSync(
        join(runDir, 'result.json'),
        JSON.stringify({
          task,
          arm,
          model: recorded,
          grades: { ok: true },
          trajectory: { model: recorded, turnCount: 1, turns: [] },
        }),
      )
      expect(isTripleComplete(root, date, task, arm, operator).complete).toBe(true)
      expect(findCompletedTriple(root, task, arm, operator)).toBe(runDir)
      // Different operator model → different path / not complete
      expect(isTripleComplete(root, date, task, arm, 'other-model').complete).toBe(
        false,
      )
    } finally {
      rmSync(root, { recursive: true, force: true })
    }
  })

  it('isTripleComplete treats empty recorded model as incomplete', () => {
    const root = mkdtempSync(join(tmpdir(), 'eval-pace-empty-model-'))
    try {
      const date = '2026-07-30'
      const runDir = tripleRunDir(root, date, 'login-form', 'sail', 'grok')
      mkdirSync(runDir, { recursive: true })
      writeFileSync(join(runDir, 'grades.json'), JSON.stringify({ ok: true }))
      writeFileSync(
        join(runDir, 'result.json'),
        JSON.stringify({
          task: 'login-form',
          arm: 'sail',
          model: '',
          grades: { ok: true },
          trajectory: { model: '', turns: [] },
        }),
      )
      expect(isTripleComplete(root, date, 'login-form', 'sail', 'grok').complete).toBe(
        false,
      )
    } finally {
      rmSync(root, { recursive: true, force: true })
    }
  })

  it('isTripleComplete does not treat grok as complete for grok-2 (prefix sibling)', () => {
    const root = mkdtempSync(join(tmpdir(), 'eval-pace-prefix-'))
    try {
      const date = '2026-07-30'
      // Recorded under grok path with model "grok"
      const runDir = tripleRunDir(root, date, 'login-form', 'sail', 'grok')
      mkdirSync(runDir, { recursive: true })
      writeFileSync(join(runDir, 'grades.json'), JSON.stringify({ ok: true }))
      writeFileSync(
        join(runDir, 'result.json'),
        JSON.stringify({
          model: 'grok',
          grades: { ok: true },
          trajectory: { model: 'grok' },
        }),
      )
      expect(isTripleComplete(root, date, 'login-form', 'sail', 'grok').complete).toBe(
        true,
      )
      // Operator grok-2 must NOT resume from grok artifacts
      expect(
        isTripleComplete(root, date, 'login-form', 'sail', 'grok-2').complete,
      ).toBe(false)
      expect(findCompletedTriple(root, 'login-form', 'sail', 'grok-2')).toBeNull()
    } finally {
      rmSync(root, { recursive: true, force: true })
    }
  })

  it('isTripleComplete treats result.error as incomplete (retriable)', () => {
    const root = mkdtempSync(join(tmpdir(), 'eval-pace-err-'))
    try {
      const date = '2026-07-30'
      const task = 'login-form'
      const arm = 'sail'
      const runDir = tripleRunDir(root, date, task, arm, 'fuzzy-baseline')
      mkdirSync(runDir, { recursive: true })
      writeFileSync(join(runDir, 'grades.json'), JSON.stringify({ ok: false }))
      writeFileSync(
        join(runDir, 'result.json'),
        JSON.stringify({
          task,
          arm,
          model: 'fuzzy-baseline',
          error: 'omp timed out',
          grades: { ok: false },
        }),
      )
      expect(isTripleComplete(root, date, task, arm, 'fuzzy-baseline').complete).toBe(
        false,
      )
      expect(findCompletedTriple(root, task, arm, 'fuzzy-baseline')).toBeNull()
    } finally {
      rmSync(root, { recursive: true, force: true })
    }
  })

  it('isTripleComplete false when grades.json missing', () => {
    const root = mkdtempSync(join(tmpdir(), 'eval-pace-partial-'))
    try {
      const date = '2026-07-30'
      const runDir = tripleRunDir(root, date, 'login-form', 'sail', 'fuzzy')
      mkdirSync(runDir, { recursive: true })
      writeFileSync(
        join(runDir, 'result.json'),
        JSON.stringify({ model: 'fuzzy', grades: { ok: true } }),
      )
      expect(isTripleComplete(root, date, 'login-form', 'sail', 'fuzzy').complete).toBe(
        false,
      )
    } finally {
      rmSync(root, { recursive: true, force: true })
    }
  })

  it('distinct models do not clobber each other on disk', async () => {
    const runsRoot = mkdtempSync(join(tmpdir(), 'eval-pace-multi-model-'))
    let calls = 0
    const mockAdapter = {
      mode: 'omp' as const,
      async run(req: {
        prompt: string
        workdir: string
        model?: string
        task: string
      }) {
        calls++
        writeFileSync(
          join(req.workdir, 'routes', 'page.sail'),
          `<column><h1>m=${req.model}</h1></column>\n`,
        )
        return {
          task: req.task,
          model: req.model ?? 'x',
          dispatch: 'omp' as const,
          turns: [{ role: 'assistant' as const, content: 'ok' }],
          turnCount: 1,
          startedAt: new Date().toISOString(),
          finishedAt: new Date().toISOString(),
          filesWritten: ['routes/page.sail'],
        }
      },
    }
    const host = {
      sailBin: null as string | null,
      async check() {
        return { ok: true, diagnostics: [], autoFixed: [] as string[] }
      },
      async inspectTree() {
        return { ok: true, nodes: [{ tag: 'column', layer: 'l1-keyword' as const }] }
      },
      async layout() {
        return { ok: true, findings: [] as unknown[] }
      },
    }
    try {
      await runPaced({
        model: 'model-a',
        intervalMs: 0,
        noSleep: true,
        arms: ['sail'],
        tasks: ['login-form'],
        lane: 'cheap',
        host,
        adapter: mockAdapter,
        runsRoot,
        writeReport: false,
        keepWorkdir: true,
        allowSkip: true,
        allowSkipBeauty: true,
      })
      await runPaced({
        model: 'model-b',
        intervalMs: 0,
        noSleep: true,
        arms: ['sail'],
        tasks: ['login-form'],
        lane: 'cheap',
        host,
        adapter: mockAdapter,
        runsRoot,
        writeReport: false,
        keepWorkdir: true,
        allowSkip: true,
        allowSkipBeauty: true,
      })
      expect(calls).toBe(2)
      const date = new Date().toISOString().slice(0, 10)
      const a = tripleRunDir(runsRoot, date, 'login-form', 'sail', 'model-a')
      const b = tripleRunDir(runsRoot, date, 'login-form', 'sail', 'model-b')
      expect(existsSync(join(a, 'result.json'))).toBe(true)
      expect(existsSync(join(b, 'result.json'))).toBe(true)
      const ra = JSON.parse(readFileSync(join(a, 'result.json'), 'utf8')) as {
        model: string
      }
      const rb = JSON.parse(readFileSync(join(b, 'result.json'), 'utf8')) as {
        model: string
      }
      expect(ra.model).toContain('model-a')
      expect(rb.model).toContain('model-b')
    } finally {
      rmSync(runsRoot, { recursive: true, force: true })
    }
  })
})

describe('materialize leak guard', () => {
  it('throws when non-sail arm template would introduce sail artifacts', () => {
    const armsRoot = mkdtempSync(join(tmpdir(), 'eval-arms-root-'))
    const work = mkdtempSync(join(tmpdir(), 'eval-arm-throw-'))
    try {
      const armDir = join(armsRoot, 'leaky')
      mkdirSync(join(armDir, 'template'), { recursive: true })
      writeFileSync(
        join(armDir, 'arm.jsonc'),
        JSON.stringify({
          id: 'leaky',
          scaffold: 'vanilla',
          context: { sailArtifacts: false },
          grading: { behavior: true, beauty: true },
        }),
      )
      writeFileSync(join(armDir, 'template', 'oops.sail'), '<column/>\n')
      const arm = loadArm('leaky', armsRoot)
      expect(() => materializeArm(work, arm)).toThrow(/leaked sail artifacts/)
    } finally {
      rmSync(armsRoot, { recursive: true, force: true })
      rmSync(work, { recursive: true, force: true })
    }
  })
})

describe('runPaced requires model (no silent default)', () => {
  it('throws when model empty', async () => {
    await expect(
      runPaced({
        model: '   ',
        arms: ['sail'],
        tasks: ['login-form'],
        dryRun: true,
        writeReport: false,
      }),
    ).rejects.toThrow(/pace requires --model/)
  })

  it('dryRun does not dispatch', async () => {
    let calls = 0
    const mockAdapter = {
      mode: 'omp' as const,
      async run() {
        calls++
        throw new Error('should not run')
      },
    }
    const summary = await runPaced({
      model: 'dry-model',
      arms: ['sail'],
      tasks: ['login-form'],
      dryRun: true,
      writeReport: false,
      adapter: mockAdapter,
      intervalMs: 0,
      noSleep: true,
    })
    expect(calls).toBe(0)
    expect(summary.paced?.executed).toBe(0)
  })
})

describe('RESULTS arm aggregation', () => {
  it('formats by-arm table for multi-arm summary', () => {
    const mk = (task: string, arm: string, ok: boolean): RunResult => ({
      task,
      arm,
      lane: 'frontier',
      dispatch: 'omp',
      model: 'fuzzy',
      workdir: '<workdir>',
      runDir: '<runDir>',
      trajectory: {
        task,
        model: 'fuzzy',
        dispatch: 'omp',
        turns: [],
        turnCount: 2,
        startedAt: '',
        finishedAt: '',
        filesWritten: [],
      },
      grades: { ok },
      startedAt: '',
      finishedAt: '',
      wallMs: 12_000,
    })
    const summary: CorpusSummary = {
      lane: 'frontier',
      dispatch: 'omp',
      startedAt: 't0',
      finishedAt: 't1',
      runs: [
        mk('login-form', 'sail', true),
        mk('login-form', 'react-tw', false),
        mk('login-form', 'vanilla', true),
      ],
      passCount: 2,
      failCount: 1,
      skipCount: 0,
      arms: ['sail', 'react-tw', 'vanilla'],
      model: 'fuzzy',
      paced: { intervalMs: 600_000, resumed: 0, executed: 3 },
    }
    const md = formatResultsMd(summary)
    expect(md).toContain('## By arm')
    expect(md).toContain('| sail |')
    expect(md).toContain('| react-tw |')
    expect(md).toContain('| vanilla |')
    expect(md).toContain('login-form')
    expect(md).toMatch(/static source-text fallback/i)
    expect(md).toMatch(/operator-owned/i)
    expect(md).not.toMatch(/\/home\//)
  })

  it('by-arm table does not count soft-skips as pass (classifyRun)', () => {
    const softSkip: RunResult = {
      task: 'login-form',
      arm: 'sail',
      lane: 'cheap',
      dispatch: 'omp',
      model: 'fuzzy',
      workdir: '<workdir>',
      runDir: '<runDir>',
      trajectory: {
        task: 'login-form',
        model: 'fuzzy',
        dispatch: 'omp',
        turns: [],
        turnCount: 1,
        startedAt: '',
        finishedAt: '',
        filesWritten: [],
      },
      grades: {
        // grades.ok true but compile soft-skipped → classifyRun = skip
        ok: true,
        compile: {
          ok: true,
          errorCount: 0,
          warningCount: 0,
          diagnostics: [],
          repairCount: 0,
          iterationsToGreen: null,
          skipped: true,
        },
        vocab_share: {
          ok: true,
          l1Count: 0,
          primitiveCount: 0,
          share: 0,
          minShare: 0.25,
          skipped: true,
        },
        gates: {
          ok: true,
          lintErrors: 0,
          layoutFindings: 0,
          composeFindings: 0,
          repairCount: 0,
          skipped: true,
        },
      },
      startedAt: '',
      finishedAt: '',
    }
    const summary: CorpusSummary = {
      lane: 'cheap',
      dispatch: 'omp',
      startedAt: 't0',
      finishedAt: 't1',
      runs: [softSkip],
      passCount: 0,
      failCount: 0,
      skipCount: 1,
      arms: ['sail'],
      model: 'fuzzy',
      paced: { intervalMs: 0, resumed: 0, executed: 1 },
    }
    const md = formatResultsMd(summary)
    // By arm: Pass=0 Fail=0 Skip=1 (not Pass=1)
    expect(md).toMatch(/\|\s*sail\s*\|\s*1\s*\|\s*0\s*\|\s*0\s*\|\s*1\s*\|/)
  })

  it('groupByArm uses unknown for missing arm (not sail)', () => {
    const summary: CorpusSummary = {
      lane: 'cheap',
      dispatch: 'omp',
      startedAt: 't0',
      finishedAt: 't1',
      runs: [
        {
          task: 'login-form',
          // arm omitted
          lane: 'cheap',
          dispatch: 'omp',
          model: 'm',
          workdir: '<w>',
          runDir: '<r>',
          trajectory: {
            task: 'login-form',
            model: 'm',
            dispatch: 'omp',
            turns: [],
            turnCount: 0,
            startedAt: '',
            finishedAt: '',
            filesWritten: [],
          },
          grades: { ok: false },
          startedAt: '',
          finishedAt: '',
        },
      ],
      passCount: 0,
      failCount: 1,
      skipCount: 0,
      arms: ['unknown'],
    }
    const md = formatResultsMd(summary)
    expect(md).toContain('| unknown |')
    expect(md).not.toMatch(/\|\s*sail\s*\|\s*1\s*\|/)
  })
})

describe('loadArms selection', () => {
  it('loads subset', () => {
    const arms = loadArms(['sail', 'vanilla'])
    expect(arms.map((a) => a.id)).toEqual(['sail', 'vanilla'])
  })
})

describe('react-tw vite scaffold', () => {
  it('writes vite + tailwind toolchain files', () => {
    const work = mkdtempSync(join(tmpdir(), 'eval-vite-scaffold-'))
    try {
      const files = scaffoldWorkdir(work, 'react-tw')
      expect(files).toEqual(
        expect.arrayContaining([
          'index.html',
          'vite.config.js',
          'postcss.config.js',
          'tailwind.config.js',
          'src/main.jsx',
          'src/App.jsx',
          'src/index.css',
          'package.json',
        ]),
      )
      const pkg = JSON.parse(readFileSync(join(work, 'package.json'), 'utf8')) as {
        dependencies: Record<string, string>
        devDependencies: Record<string, string>
        scripts: Record<string, string>
      }
      expect(pkg.dependencies.react).toBeTruthy()
      expect(pkg.devDependencies.vite).toBeTruthy()
      expect(pkg.devDependencies.tailwindcss).toBeTruthy()
      expect(pkg.scripts.dev).toContain('vite')
      expect(readFileSync(join(work, 'src', 'index.css'), 'utf8')).toMatch(/@tailwind/)
      expect(existsSync(join(work, 'routes', 'page.sail'))).toBe(false)
      expect(findSailArtifactLeaks(work)).toEqual([])
    } finally {
      rmSync(work, { recursive: true, force: true })
    }
  })
})

describe('paced runner smoke (1 task × 1 arm, mock omp)', () => {
  const host = {
    sailBin: null as string | null,
    async check() {
      return { ok: true, diagnostics: [], autoFixed: [] as string[] }
    },
    async inspectTree() {
      return {
        ok: true,
        nodes: [{ tag: 'column', layer: 'l1-keyword' as const }],
      }
    },
    async layout() {
      return { ok: true, findings: [] as unknown[] }
    },
  }

  it('records trajectory, grades, RESULTS; second call resumes', async () => {
    const runsRoot = mkdtempSync(join(tmpdir(), 'eval-pace-smoke-'))
    const mockAdapter = {
      mode: 'omp' as const,
      async run(req: {
        prompt: string
        workdir: string
        model?: string
        task: string
        scaffold?: string
        sailArtifacts?: boolean
      }) {
        expect(req.scaffold).toBe('blank')
        expect(req.sailArtifacts).toBe(true)
        writeFileSync(
          join(req.workdir, 'routes', 'page.sail'),
          '<column gap="4"><h1>Sign in</h1></column>\n',
        )
        return {
          task: req.task,
          model: req.model ?? 'smoke',
          dispatch: 'omp' as const,
          turns: [
            { role: 'user' as const, content: req.prompt },
            { role: 'assistant' as const, content: 'done' },
          ],
          turnCount: 1,
          startedAt: new Date().toISOString(),
          finishedAt: new Date().toISOString(),
          filesWritten: ['routes/page.sail'],
        }
      },
    }
    try {
      const first = await runPaced({
        model: 'smoke-model',
        intervalMs: 0,
        noSleep: true,
        arms: ['sail'],
        tasks: ['login-form'],
        lane: 'cheap',
        host,
        adapter: mockAdapter,
        runsRoot,
        writeReport: false,
        keepWorkdir: true,
        allowSkip: true,
        allowSkipBeauty: true,
      })
      expect(first.paced?.executed).toBe(1)
      expect(first.paced?.resumed).toBe(0)
      expect(first.runs).toHaveLength(1)
      expect(first.runs[0]!.arm).toBe('sail')
      expect(first.runs[0]!.trajectory.turnCount).toBe(1)
      expect(first.runs[0]!.model).toContain('smoke')
      expect(first.runs[0]!.wallMs).toBeTypeOf('number')
      expect(first.runs[0]!.runDir).not.toBe('<runDir>')
      expect(existsSync(join(first.runs[0]!.runDir, 'grades.json'))).toBe(true)
      expect(existsSync(join(first.runs[0]!.runDir, 'result.json'))).toBe(true)
      expect(existsSync(join(first.runs[0]!.runDir, 'trajectory.jsonl'))).toBe(true)
      expect(existsSync(join(first.runs[0]!.runDir, 'screenshots', 'README.md'))).toBe(
        true,
      )
      // Path includes model slug (base + content hash)
      expect(first.runs[0]!.runDir.replace(/\\/g, '/')).toMatch(
        /\/smoke-model-[a-f0-9]{8}$/,
      )
      expect(first.runs[0]!.runDir).toBe(
        tripleRunDir(
          runsRoot,
          new Date().toISOString().slice(0, 10),
          'login-form',
          'sail',
          'smoke-model',
        ),
      )

      const second = await runPaced({
        model: 'smoke-model',
        intervalMs: 0,
        noSleep: true,
        arms: ['sail'],
        tasks: ['login-form'],
        lane: 'cheap',
        host,
        adapter: mockAdapter,
        runsRoot,
        writeReport: false,
        allowSkip: true,
        allowSkipBeauty: true,
      })
      expect(second.paced?.executed).toBe(0)
      expect(second.paced?.resumed).toBe(1)
      expect(second.runs).toHaveLength(1)
    } finally {
      rmSync(runsRoot, { recursive: true, force: true })
    }
  })

  it('retries wipe dirty workdir (fresh materialize)', async () => {
    const runsRoot = mkdtempSync(join(tmpdir(), 'eval-pace-wipe-'))
    let attempt = 0
    const mockAdapter = {
      mode: 'omp' as const,
      async run(req: {
        workdir: string
        model?: string
        task: string
      }) {
        attempt++
        if (attempt === 1) {
          // Contaminate workdir as if agent wrote junk + fail the run via throw path:
          // write a poison file then succeed so grades land; we force error by writing
          // result.error on first pass via throwing after writing.
          writeFileSync(join(req.workdir, 'poison.sail'), '<column/>\n')
          writeFileSync(join(req.workdir, 'leftover.txt'), 'stale agent output\n')
          throw new Error('simulated mid-run failure')
        }
        // Second attempt: leftover must be gone (fresh workdir)
        expect(existsSync(join(req.workdir, 'leftover.txt'))).toBe(false)
        expect(existsSync(join(req.workdir, 'poison.sail'))).toBe(false)
        writeFileSync(
          join(req.workdir, 'routes', 'page.sail'),
          '<column><h1>ok</h1></column>\n',
        )
        return {
          task: req.task,
          model: req.model ?? 'wipe',
          dispatch: 'omp' as const,
          turns: [{ role: 'assistant' as const, content: 'ok' }],
          turnCount: 1,
          startedAt: new Date().toISOString(),
          finishedAt: new Date().toISOString(),
          filesWritten: ['routes/page.sail'],
        }
      },
    }
    try {
      const first = await runPaced({
        model: 'wipe-model',
        intervalMs: 0,
        noSleep: true,
        arms: ['sail'],
        tasks: ['login-form'],
        lane: 'cheap',
        host,
        adapter: mockAdapter,
        runsRoot,
        writeReport: false,
        allowSkip: true,
        allowSkipBeauty: true,
      })
      expect(first.runs[0]!.error).toMatch(/simulated mid-run failure/)

      const second = await runPaced({
        model: 'wipe-model',
        intervalMs: 0,
        noSleep: true,
        arms: ['sail'],
        tasks: ['login-form'],
        lane: 'cheap',
        host,
        adapter: mockAdapter,
        runsRoot,
        writeReport: false,
        keepWorkdir: true,
        allowSkip: true,
        allowSkipBeauty: true,
      })
      expect(second.paced?.executed).toBe(1)
      expect(second.runs[0]!.error).toBeUndefined()
      expect(attempt).toBe(2)
    } finally {
      rmSync(runsRoot, { recursive: true, force: true })
    }
  })

  it('control arm react-tw: no sail grades, framing flags set, no sail leaks in workdir', async () => {
    const runsRoot = mkdtempSync(join(tmpdir(), 'eval-pace-rtw-'))
    let sawReq: {
      scaffold?: string
      sailArtifacts?: boolean
    } | null = null
    const mockAdapter = {
      mode: 'omp' as const,
      async run(req: {
        prompt: string
        workdir: string
        model?: string
        task: string
        scaffold?: string
        sailArtifacts?: boolean
      }) {
        sawReq = {
          scaffold: req.scaffold,
          sailArtifacts: req.sailArtifacts,
        }
        // Full login-form chrome in App.jsx (Vite shell index.html stays empty #root)
        writeFileSync(
          join(req.workdir, 'src', 'App.jsx'),
          `export default function App() {
  return (
    <div>
      <h1>Sign in</h1>
      <label htmlFor="email">Email</label>
      <input id="email" name="email" />
      <label htmlFor="password">Password</label>
      <input id="password" name="password" type="password" />
      <button type="submit">Sign in</button>
      <a href="/forgot">Forgot password?</a>
    </div>
  )
}
`,
        )
        return {
          task: req.task,
          model: req.model ?? 'smoke',
          dispatch: 'omp' as const,
          turns: [
            { role: 'user' as const, content: req.prompt },
            { role: 'assistant' as const, content: 'done' },
          ],
          turnCount: 2,
          startedAt: new Date().toISOString(),
          finishedAt: new Date().toISOString(),
          filesWritten: ['src/App.jsx'],
        }
      },
    }
    const noSailHost = {
      sailBin: null as string | null,
      async check() {
        return { ok: false, diagnostics: [], autoFixed: [] as string[], error: 'no sail' }
      },
      async inspectTree() {
        return { ok: false, nodes: [], error: 'no sail' }
      },
      async layout() {
        return { ok: true, findings: [] as unknown[] }
      },
    }
    try {
      const summary = await runPaced({
        model: 'smoke-rtw',
        intervalMs: 0,
        noSleep: true,
        arms: ['react-tw'],
        tasks: ['login-form'],
        lane: 'frontier',
        host: noSailHost,
        adapter: mockAdapter,
        runsRoot,
        writeReport: false,
        keepWorkdir: true,
        allowSkip: true,
        allowSkipBeauty: true,
      })
      expect(summary.paced?.executed).toBe(1)
      expect(sawReq?.scaffold).toBe('react-tw')
      expect(sawReq?.sailArtifacts).toBe(false)
      const run = summary.runs[0]!
      expect(run.arm).toBe('react-tw')
      expect(run.grades.compile).toBeUndefined()
      expect(run.grades.vocab_share).toBeUndefined()
      expect(run.grades.gates).toBeUndefined()
      // Always resolve a real workdir — no soft-skip of leak assertion
      expect(run.workdir).not.toBe('<workdir>')
      expect(run.workdir).toBeTruthy()
      expect(existsSync(run.workdir)).toBe(true)
      expect(findSailArtifactLeaks(run.workdir)).toEqual([])
      // Vite scaffold present after materialize
      expect(existsSync(join(run.workdir, 'vite.config.js'))).toBe(true)
      expect(existsSync(join(run.workdir, 'tailwind.config.js'))).toBe(true)
      // Static behavior must grade App.jsx (not empty index.html shell)
      expect(run.grades.behavior).toBeTruthy()
      expect(run.grades.behavior?.skipped).not.toBe(true)
      expect(run.grades.behavior?.ok).toBe(true)
      expect(run.grades.behavior?.detail ?? '').toMatch(/static source-text fallback/i)
    } finally {
      rmSync(runsRoot, { recursive: true, force: true })
    }
  })

  it('control arm vanilla: no sail grades, no leaks', async () => {
    const runsRoot = mkdtempSync(join(tmpdir(), 'eval-pace-van-'))
    const mockAdapter = {
      mode: 'omp' as const,
      async run(req: {
        workdir: string
        model?: string
        task: string
        scaffold?: string
        sailArtifacts?: boolean
      }) {
        expect(req.scaffold).toBe('vanilla')
        expect(req.sailArtifacts).toBe(false)
        writeFileSync(
          join(req.workdir, 'index.html'),
          '<!doctype html><html><body><h1>Sign in</h1></body></html>\n',
        )
        return {
          task: req.task,
          model: req.model ?? 'smoke',
          dispatch: 'omp' as const,
          turns: [{ role: 'assistant' as const, content: 'ok' }],
          turnCount: 1,
          startedAt: new Date().toISOString(),
          finishedAt: new Date().toISOString(),
          filesWritten: ['index.html'],
        }
      },
    }
    try {
      const summary = await runPaced({
        model: 'smoke-van',
        intervalMs: 0,
        noSleep: true,
        arms: ['vanilla'],
        tasks: ['login-form'],
        lane: 'frontier',
        host,
        adapter: mockAdapter,
        runsRoot,
        writeReport: false,
        keepWorkdir: true,
        allowSkip: true,
        allowSkipBeauty: true,
      })
      const run = summary.runs[0]!
      expect(run.arm).toBe('vanilla')
      expect(run.grades.compile).toBeUndefined()
      expect(run.workdir).not.toBe('<workdir>')
      expect(findSailArtifactLeaks(run.workdir)).toEqual([])
    } finally {
      rmSync(runsRoot, { recursive: true, force: true })
    }
  })
})
