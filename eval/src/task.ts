import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs'
import { join } from 'node:path'
import { parseJsonc } from './jsonc.ts'
import { assertSafeSlug, tasksDir } from './paths.ts'
import type { BehaviorSpec, LoadedTask, TaskDef } from './types.ts'

export function loadTask(slug: string, root = tasksDir()): LoadedTask {
  const safe = assertSafeSlug(slug)
  const dir = join(root, safe)
  if (!existsSync(dir) || !statSync(dir).isDirectory()) {
    throw new Error(`task not found: ${safe} (looked in ${dir})`)
  }
  const taskPath = join(dir, 'task.jsonc')
  if (!existsSync(taskPath)) {
    throw new Error(`missing task.jsonc for ${safe}`)
  }
  const def = parseJsonc<TaskDef>(readFileSync(taskPath, 'utf8'))
  if (!def.prompt || typeof def.prompt !== 'string') {
    throw new Error(`task ${safe}: prompt is required`)
  }
  if (!def.scaffold || !['blank', 'kitchen-sink', 'react-tw'].includes(def.scaffold)) {
    throw new Error(`task ${safe}: scaffold must be blank|kitchen-sink|react-tw`)
  }
  if (!Array.isArray(def.lanes) || def.lanes.length === 0) {
    throw new Error(`task ${safe}: lanes must be a non-empty array`)
  }
  for (const lane of def.lanes) {
    if (lane !== 'cheap' && lane !== 'frontier') {
      throw new Error(
        `task ${safe}: invalid lane "${String(lane)}" (expected cheap|frontier)`,
      )
    }
  }
  if (!def.grading || typeof def.grading !== 'object') {
    throw new Error(`task ${safe}: grading config required`)
  }

  let spec: BehaviorSpec | null = null
  const specPath = join(dir, 'spec.jsonc')
  if (existsSync(specPath)) {
    spec = parseJsonc<BehaviorSpec>(readFileSync(specPath, 'utf8'))
    if (!Array.isArray(spec.scenarios)) {
      throw new Error(`task ${safe}: spec.jsonc scenarios must be an array`)
    }
  }

  const goldenDir = join(dir, 'golden')
  return {
    slug: safe,
    dir,
    def,
    spec,
    goldenDir: existsSync(goldenDir) ? goldenDir : null,
  }
}

export function listTasks(root = tasksDir()): string[] {
  if (!existsSync(root)) return []
  return readdirSync(root)
    .filter((name) => {
      try {
        assertSafeSlug(name)
      } catch {
        return false
      }
      return (
        statSync(join(root, name)).isDirectory() &&
        existsSync(join(root, name, 'task.jsonc'))
      )
    })
    .sort()
}

export function loadCorpus(root = tasksDir()): LoadedTask[] {
  return listTasks(root).map((slug) => loadTask(slug, root))
}
