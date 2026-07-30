import { describe, expect, it } from 'vitest'
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { loadCorpus, loadTask, listTasks } from '../src/task.ts'
import { tasksDir } from '../src/paths.ts'
import { parseJsonc } from '../src/jsonc.ts'

const EXPECTED = [
  'asymmetric-editorial-layout',
  'card-grid',
  'dashboard-shell',
  'dense-data-table',
  'docs-article-page',
  'empty-state-list',
  'login-form',
  'mobile-first-nav-flow',
  'realtime-chat-feed',
  'settings-page',
]

describe('corpus tasks', () => {
  it('lists all 10 v1 tasks', () => {
    const slugs = listTasks()
    expect(slugs.sort()).toEqual([...EXPECTED].sort())
  })

  it('loads each task with valid shape + golden', () => {
    for (const slug of EXPECTED) {
      const t = loadTask(slug)
      expect(t.slug).toBe(slug)
      expect(t.def.prompt.length).toBeGreaterThan(20)
      expect(t.def.lanes).toContain('cheap')
      expect(t.def.grading.compile).toBe(true)
      expect(t.spec?.scenarios?.length).toBeGreaterThan(0)
      expect(t.goldenDir).toBeTruthy()
      expect(existsSync(join(t.goldenDir!, 'routes', 'page.sail'))).toBe(true)
      // scenarios never use CSS selectors as primary queries
      const raw = readFileSync(join(t.dir, 'spec.jsonc'), 'utf8')
      expect(raw).not.toMatch(/querySelector|getElementById\(|#[a-z]+-/)
    }
  })

  it('loadCorpus returns 10', () => {
    expect(loadCorpus()).toHaveLength(10)
  })

  it('rejects unknown task', () => {
    expect(() => loadTask('no-such-task-xyz')).toThrow(/not found/)
  })

  it('task.jsonc is parseable JSONC with comments', () => {
    const raw = readFileSync(join(tasksDir(), 'login-form', 'task.jsonc'), 'utf8')
    expect(raw).toMatch(/\/\//)
    const def = parseJsonc(raw)
    expect(def).toHaveProperty('prompt')
  })

  it('rejects invalid lane values', () => {
    const tmp = mkdtempSync(join(tmpdir(), 'eval-lane-'))
    try {
      mkdirSync(join(tmp, 'bad-lane'), { recursive: true })
      writeFileSync(
        join(tmp, 'bad-lane', 'task.jsonc'),
        JSON.stringify({
          prompt: 'enough characters for a real prompt here',
          scaffold: 'blank',
          lanes: ['Cheap'],
          grading: { compile: true },
        }),
      )
      expect(() => loadTask('bad-lane', tmp)).toThrow(/invalid lane/)
    } finally {
      rmSync(tmp, { recursive: true, force: true })
    }
  })
})
