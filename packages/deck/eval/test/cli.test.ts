import { describe, expect, it } from 'vitest'
import { resolve } from 'node:path'
import { listTasks, loadTask, parseArgv } from '../src/cli'

const TASKS_DIR = resolve(__dirname, '..', 'tasks')

describe('parseArgv', () => {
  it('reads the command and string flags', () => {
    const { command, flags } = parseArgv(['run', '--task', 'x', '--model', 'y'])
    expect(command).toBe('run')
    expect(flags['task']).toBe('x')
    expect(flags['model']).toBe('y')
  })

  it('reads boolean flags', () => {
    const { flags } = parseArgv(['run', '--all'])
    expect(flags['all']).toBe(true)
  })
})

describe('loadTask', () => {
  it('loads a known easy task', async () => {
    const t = await loadTask(TASKS_DIR, 'static-info-card')
    expect(t.id).toBe('static-info-card')
    expect(t.difficulty).toBe('easy')
    expect(t.behavioralTests).toContain('data-test')
  })

  it('throws on an unknown task', async () => {
    await expect(loadTask(TASKS_DIR, 'no-such-task')).rejects.toThrow(/not found/)
  })
})

describe('listTasks', () => {
  it('returns at least 6 tasks across all tiers', async () => {
    const all = await listTasks(TASKS_DIR)
    expect(all.length).toBeGreaterThanOrEqual(6)
    const tiers = new Set(all.map((t) => t.difficulty))
    expect(tiers.has('easy')).toBe(true)
    expect(tiers.has('medium')).toBe(true)
    expect(tiers.has('hard')).toBe(true)
  })

  it('filters by tier', async () => {
    const easy = await listTasks(TASKS_DIR, 'easy')
    expect(easy.length).toBeGreaterThanOrEqual(2)
    expect(easy.every((t) => t.difficulty === 'easy')).toBe(true)
  })
})
