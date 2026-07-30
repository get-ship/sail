import { describe, expect, it } from 'vitest'
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { gradeCompile, collectSailFiles } from '../src/graders/compile.ts'
import { gradeGates } from '../src/graders/gates.ts'
import { gradeBeauty } from '../src/graders/beauty.ts'
import { runGradeChain } from '../src/graders/chain.ts'
import type { HostTooling } from '../src/host.ts'
import type { Trajectory } from '../src/types.ts'

// HostTooling re-export used by host-error gates test

function mockHost(opts: {
  ok?: boolean
  diagnostics?: Array<{ severity: string; message: string; rule?: string }>
  autoFixed?: Array<Record<string, unknown>>
  nodes?: Array<{ tag: string; layer?: string }>
  layoutFindings?: Array<Record<string, unknown>>
}): HostTooling {
  return {
    sailBin: '/mock/sail',
    async check() {
      return {
        ok: opts.ok ?? true,
        diagnostics: opts.diagnostics ?? [],
        autoFixed: opts.autoFixed ?? [],
      }
    },
    async inspectTree() {
      return { ok: true, nodes: opts.nodes ?? [{ tag: 'card', layer: 'l1-element' }] }
    },
    async layout() {
      return { ok: true, findings: opts.layoutFindings ?? [] }
    },
  }
}

describe('compile grader', () => {
  it('collectSailFiles finds nested sails and skips node_modules', () => {
    const dir = mkdtempSync(join(tmpdir(), 'eval-sail-'))
    try {
      mkdirSync(join(dir, 'routes'), { recursive: true })
      mkdirSync(join(dir, 'node_modules', 'x'), { recursive: true })
      writeFileSync(join(dir, 'routes', 'page.sail'), '<p>x</p>')
      writeFileSync(join(dir, 'node_modules', 'x', 'bad.sail'), '<p>no</p>')
      const files = collectSailFiles(dir)
      expect(files).toHaveLength(1)
      expect(files[0]).toMatch(/page\.sail$/)
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })

  it('fails when no sail files', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'eval-empty-'))
    try {
      const g = await gradeCompile({
        host: mockHost({}),
        workdir: dir,
        allowSkip: false,
      })
      expect(g.ok).toBe(false)
      expect(g.errorCount).toBe(1)
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })

  it('passes clean check and records iterations-to-green', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'eval-ok-'))
    try {
      mkdirSync(join(dir, 'routes'), { recursive: true })
      writeFileSync(join(dir, 'routes', 'page.sail'), '<column><p>hi</p></column>')
      const traj: Trajectory = {
        task: 't',
        model: 'm',
        dispatch: 'recorded',
        turns: [
          { role: 'user', content: 'x' },
          { role: 'assistant', content: 'y' },
        ],
        turnCount: 2,
        startedAt: '',
        finishedAt: '',
        filesWritten: [],
      }
      const g = await gradeCompile({
        host: mockHost({ ok: true }),
        workdir: dir,
        trajectory: traj,
      })
      expect(g.ok).toBe(true)
      expect(g.iterationsToGreen).toBe(2)
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })

  it('counts repairs from autoFixed', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'eval-fix-'))
    try {
      mkdirSync(join(dir, 'routes'), { recursive: true })
      writeFileSync(join(dir, 'routes', 'page.sail'), '<p>x</p>')
      const g = await gradeCompile({
        host: mockHost({
          ok: true,
          autoFixed: [{ rule: 'sail-compose-rhythm' }, { rule: 'sail-compose-rhythm' }],
        }),
        workdir: dir,
      })
      expect(g.repairCount).toBe(2)
      expect(g.ok).toBe(true)
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })

  it('skips when sail binary null with allowSkip true', async () => {
    const host: HostTooling = {
      sailBin: null,
      async check() {
        return { ok: false, diagnostics: [], autoFixed: [] }
      },
      async inspectTree() {
        return { ok: false, nodes: [] }
      },
      async layout() {
        return { ok: true, findings: [] }
      },
    }
    const g = await gradeCompile({ host, workdir: '/tmp', allowSkip: true })
    expect(g.skipped).toBe(true)
  })

  it('defaults allowSkip false — missing sail fails compile', async () => {
    const host: HostTooling = {
      sailBin: null,
      async check() {
        return { ok: false, diagnostics: [], autoFixed: [] }
      },
      async inspectTree() {
        return { ok: false, nodes: [] }
      },
      async layout() {
        return { ok: true, findings: [] }
      },
    }
    const g = await gradeCompile({ host, workdir: '/tmp' })
    expect(g.ok).toBe(false)
    expect(g.skipped).toBeFalsy()
  })
})

describe('gates grader', () => {
  it('fails on lint errors', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'eval-gates-'))
    try {
      mkdirSync(join(dir, 'routes'), { recursive: true })
      writeFileSync(join(dir, 'routes', 'page.sail'), '<p>x</p>')
      const g = await gradeGates({
        host: mockHost({
          ok: false,
          diagnostics: [{ severity: 'error', message: 'bad', rule: 'x' }],
        }),
        workdir: dir,
      })
      expect(g.ok).toBe(false)
      expect(g.lintErrors).toBe(1)
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })

  it('records layout findings without failing', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'eval-layout-'))
    try {
      mkdirSync(join(dir, 'routes'), { recursive: true })
      writeFileSync(join(dir, 'routes', 'page.sail'), '<p>x</p>')
      const g = await gradeGates({
        host: mockHost({ layoutFindings: [{ rule: 'overflow' }] }),
        workdir: dir,
      })
      expect(g.ok).toBe(true)
      expect(g.layoutFindings).toBe(1)
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })
})

describe('beauty grader', () => {
  it('cheap lane skips without score', async () => {
    const g = await gradeBeauty({ lane: 'cheap' })
    expect(g.skipped).toBe(true)
    expect(g.ok).toBe(true)
  })

  it('frontier with score uses pass threshold', async () => {
    const pass = await gradeBeauty({ lane: 'frontier', score: 8 })
    expect(pass.ok).toBe(true)
    expect(pass.source).toBe('judge')
    const fail = await gradeBeauty({ lane: 'frontier', score: 3 })
    expect(fail.ok).toBe(false)
  })

  it('frontier without score is incomplete (not a green skip)', async () => {
    const g = await gradeBeauty({ lane: 'frontier' })
    expect(g.skipped).toBe(true)
    expect(g.ok).toBe(false)
  })

  it('frontier without score soft-skips when allowSkip', async () => {
    const g = await gradeBeauty({ lane: 'frontier', allowSkip: true })
    expect(g.skipped).toBe(true)
    expect(g.ok).toBe(true)
  })
})

describe('gates host errors', () => {
  it('fails when check returns host error without raw payload', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'eval-gates-host-'))
    try {
      mkdirSync(join(dir, 'routes'), { recursive: true })
      writeFileSync(join(dir, 'routes', 'page.sail'), '<p>x</p>')
      const host: HostTooling = {
        sailBin: '/mock/sail',
        async check() {
          return {
            ok: false,
            diagnostics: [],
            autoFixed: [],
            error: 'sail crashed',
          }
        },
        async inspectTree() {
          return { ok: false, nodes: [] }
        },
        async layout() {
          return { ok: true, findings: [] }
        },
      }
      const g = await gradeGates({ host, workdir: dir, allowSkip: false })
      expect(g.ok).toBe(false)
      expect(g.lintErrors).toBeGreaterThanOrEqual(1)
      expect(g.detail).toMatch(/crashed/)
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })
})

describe('grade chain', () => {
  it('runs compile+vocab+gates for cheap (behavior off by default)', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'eval-chain-'))
    try {
      mkdirSync(join(dir, 'routes'), { recursive: true })
      writeFileSync(
        join(dir, 'routes', 'page.sail'),
        '<column><card><h1>Sign in</h1><button>Sign in</button></card></column>',
      )
      const report = await runGradeChain({
        host: mockHost({
          nodes: [
            { tag: 'column', layer: 'l1-keyword' },
            { tag: 'card', layer: 'l1-element' },
            { tag: 'h1', layer: 'l1-element' },
            { tag: 'button', layer: 'l1-element' },
          ],
        }),
        workdir: dir,
        lane: 'cheap',
        grading: {
          compile: true,
          vocab_share: true,
          gates: true,
          behavior: true,
          beauty: true,
        },
        behaviorHtml: '<h1>Sign in</h1><button>Sign in</button>',
        spec: {
          scenarios: [{ name: 's', steps: [{ see: 'Sign in' }] }],
        },
      })
      expect(report.compile?.ok).toBe(true)
      expect(report.vocab_share?.ok).toBe(true)
      expect(report.gates?.ok).toBe(true)
      // cheap behavior skipped unless enableCheapBehavior
      expect(report.behavior?.skipped).toBe(true)
      expect(report.beauty?.skipped).toBe(true)
      expect(report.ok).toBe(true)
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })

  it('runs cheap behavior only when enableCheapBehavior', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'eval-chain-beh-'))
    try {
      mkdirSync(join(dir, 'routes'), { recursive: true })
      writeFileSync(join(dir, 'routes', 'page.sail'), '<h1>Sign in</h1>')
      const report = await runGradeChain({
        host: mockHost({}),
        workdir: dir,
        lane: 'cheap',
        grading: {
          compile: true,
          vocab_share: false,
          gates: false,
          behavior: true,
        },
        behaviorHtml: '<h1>Sign in</h1>',
        enableCheapBehavior: true,
        spec: {
          scenarios: [{ name: 's', steps: [{ see: 'Sign in' }] }],
        },
      })
      expect(report.behavior?.ok).toBe(true)
      expect(report.behavior?.skipped).toBeFalsy()
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })

  it('marks overall not ok when compile fails', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'eval-chain-fail-'))
    try {
      mkdirSync(join(dir, 'routes'), { recursive: true })
      writeFileSync(join(dir, 'routes', 'page.sail'), '<p>x</p>')
      const report = await runGradeChain({
        host: mockHost({
          ok: false,
          diagnostics: [{ severity: 'error', message: 'boom' }],
        }),
        workdir: dir,
        lane: 'cheap',
        grading: { compile: true, vocab_share: false, gates: false },
        allowSkip: false,
      })
      expect(report.ok).toBe(false)
    } finally {
      rmSync(dir, { recursive: true, force: true })
    }
  })
})
