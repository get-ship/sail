import { describe, expect, it } from 'vitest'
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync, existsSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import {
  RecordedDispatch,
  readTrajectoryJsonl,
  createDispatch,
  buildOmpArgs,
  resolveModelLabel,
} from '../src/dispatch/index.ts'
import { OmpDispatch } from '../src/dispatch/omp.ts'
import { scaffoldWorkdir } from '../src/scaffold.ts'

describe('recorded dispatch', () => {
  it('copies golden files and builds trajectory', async () => {
    const fixture = mkdtempSync(join(tmpdir(), 'eval-fix-'))
    const work = mkdtempSync(join(tmpdir(), 'eval-work-'))
    try {
      mkdirSync(join(fixture, 'routes'), { recursive: true })
      writeFileSync(join(fixture, 'routes', 'page.sail'), '<h1>Sign in</h1>\n')
      writeFileSync(
        join(fixture, 'trajectory.jsonl'),
        JSON.stringify({ role: 'user', content: 'do it' }) +
          '\n' +
          JSON.stringify({ role: 'assistant', content: 'done' }) +
          '\n',
      )
      const d = new RecordedDispatch()
      const traj = await d.run({
        prompt: 'do it',
        workdir: work,
        model: 'recorded',
        task: 'login-form',
        fixtureDir: fixture,
      })
      expect(traj.dispatch).toBe('recorded')
      expect(traj.turnCount).toBe(1)
      expect(existsSync(join(work, 'routes', 'page.sail'))).toBe(true)
      expect(readFileSync(join(work, 'routes', 'page.sail'), 'utf8')).toContain('Sign in')
      expect(traj.filesWritten).toContain('routes/page.sail')
    } finally {
      rmSync(fixture, { recursive: true, force: true })
      rmSync(work, { recursive: true, force: true })
    }
  })

  it('throws when fixture missing', async () => {
    const work = mkdtempSync(join(tmpdir(), 'eval-work-'))
    try {
      const d = new RecordedDispatch()
      await expect(
        d.run({
          prompt: 'x',
          workdir: work,
          model: 'recorded',
          task: 'missing-task',
          fixtureDir: join(work, 'nope'),
        }),
      ).rejects.toThrow(/no fixture/)
    } finally {
      rmSync(work, { recursive: true, force: true })
    }
  })

  it('readTrajectoryJsonl skips bad lines', () => {
    const turns = readTrajectoryJsonl(
      'not-json\n{"role":"assistant","content":"ok"}\n// comment\n',
    )
    expect(turns).toHaveLength(1)
    expect(turns[0]!.content).toBe('ok')
  })
})

describe('createDispatch', () => {
  it('returns recorded and omp adapters', () => {
    expect(createDispatch('recorded').mode).toBe('recorded')
    expect(createDispatch('omp').mode).toBe('omp')
  })
})

describe('buildOmpArgs / resolveModelLabel', () => {
  it('never injects a default --model', () => {
    const args = buildOmpArgs({
      workdir: '/tmp/w',
      prompt: 'do the task',
    })
    expect(args).toEqual([
      '-p',
      '--cwd',
      '/tmp/w',
      '--no-session',
      '--mode',
      'json',
      'do the task',
    ])
    expect(args).not.toContain('--model')
  })

  it('passes --model and --api-key only when provided', () => {
    const args = buildOmpArgs({
      workdir: '/tmp/w',
      prompt: 'p',
      model: 'opus',
      apiKey: 'sk-test',
    })
    expect(args).toContain('--model')
    expect(args).toContain('opus')
    expect(args).toContain('--api-key')
    expect(args).toContain('sk-test')
  })

  it('resolveModelLabel prefers requested model; never invents one', () => {
    expect(resolveModelLabel('opus', '')).toBe('opus')
    expect(resolveModelLabel(undefined, '')).toBe('')
    expect(resolveModelLabel('', '{"model":"provider/x"}')).toBe('provider/x')
  })
})

describe('omp dispatch', () => {
  it('runs a mock binary and captures output', async () => {
    const work = mkdtempSync(join(tmpdir(), 'eval-omp-'))
    try {
      scaffoldWorkdir(work, 'blank')
      // Fake "omp" that writes a file and prints JSON
      const bin = join(work, 'fake-omp.sh')
      writeFileSync(
        bin,
        `#!/bin/sh
echo '{"model":"mock/resolved"}'
echo "extra" > routes/note.txt
`,
      )
      const { chmodSync } = await import('node:fs')
      chmodSync(bin, 0o755)
      const d = new OmpDispatch({ bin, extraArgs: [] })
      const traj = await d.run({
        prompt: 'build login',
        workdir: work,
        model: 'fuzzy-test',
        task: 'login-form',
        timeoutMs: 5000,
      })
      expect(traj.dispatch).toBe('omp')
      expect(traj.model).toBe('fuzzy-test')
      expect(traj.turns[1]?.content).toContain('mock/resolved')
      expect(traj.filesWritten.some((f) => f.includes('note.txt'))).toBe(true)
    } finally {
      rmSync(work, { recursive: true, force: true })
    }
  })

  it('rejects when binary missing', async () => {
    const work = mkdtempSync(join(tmpdir(), 'eval-omp-miss-'))
    try {
      const d = new OmpDispatch({ bin: join(work, 'does-not-exist-omp') })
      await expect(
        d.run({
          prompt: 'x',
          workdir: work,
          task: 't',
          timeoutMs: 1000,
        }),
      ).rejects.toThrow()
    } finally {
      rmSync(work, { recursive: true, force: true })
    }
  })
})

describe('react-tw control-arm scaffold', () => {
  it('writes App.jsx shell without sail routes', () => {
    const work = mkdtempSync(join(tmpdir(), 'eval-react-tw-'))
    try {
      const files = scaffoldWorkdir(work, 'react-tw')
      expect(files).toContain('src/App.jsx')
      expect(existsSync(join(work, 'src', 'App.jsx'))).toBe(true)
      expect(existsSync(join(work, 'routes', 'page.sail'))).toBe(false)
      expect(readFileSync(join(work, 'src', 'App.jsx'), 'utf8')).toContain('react-tw')
    } finally {
      rmSync(work, { recursive: true, force: true })
    }
  })
})
