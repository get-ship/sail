/**
 * Recorded-lane replay: no live agent.
 * Copies golden/fixture files into the workdir and returns a synthetic trajectory.
 * CI-suitable cheap-lane path.
 */

import { existsSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { copyTreeInto } from '../scaffold.ts'
import type { Trajectory, TrajectoryTurn } from '../types.ts'
import type { DispatchAdapter, DispatchRequest } from './types.ts'

export class RecordedDispatch implements DispatchAdapter {
  readonly mode = 'recorded' as const

  async run(req: DispatchRequest): Promise<Trajectory> {
    const startedAt = new Date().toISOString()
    const fixtureDir = req.fixtureDir
    if (!fixtureDir || !existsSync(fixtureDir)) {
      throw new Error(
        `recorded dispatch: no fixture/golden for task "${req.task}" (looked at ${fixtureDir ?? 'null'})`,
      )
    }

    let turns: TrajectoryTurn[] = []
    const trajPath = join(fixtureDir, 'trajectory.jsonl')
    if (existsSync(trajPath)) {
      turns = readTrajectoryJsonl(readFileSync(trajPath, 'utf8'))
    } else {
      turns = [
        { role: 'user', content: req.prompt, ts: startedAt },
        {
          role: 'assistant',
          content: `[recorded] replaying golden/fixture for ${req.task}`,
          ts: startedAt,
        },
      ]
    }

    // Copy solution files (skip trajectory metadata files)
    const filesWritten = copyTreeInto(fixtureDir, req.workdir).filter(
      (p) => p !== 'trajectory.jsonl' && !p.endsWith('.meta.json'),
    )

    const finishedAt = new Date().toISOString()
    return {
      task: req.task,
      model: req.model || 'recorded',
      dispatch: 'recorded',
      turns,
      turnCount: countAssistantTurns(turns),
      startedAt,
      finishedAt,
      filesWritten,
      meta: { fixtureDir },
    }
  }
}

export function readTrajectoryJsonl(text: string): TrajectoryTurn[] {
  const turns: TrajectoryTurn[] = []
  for (const line of text.split('\n')) {
    const t = line.trim()
    if (!t || t.startsWith('//')) continue
    try {
      const obj = JSON.parse(t) as TrajectoryTurn
      if (obj && typeof obj.role === 'string' && typeof obj.content === 'string') {
        turns.push(obj)
      }
    } catch {
      // skip malformed lines; recorded fixtures should be clean
    }
  }
  return turns
}

function countAssistantTurns(turns: TrajectoryTurn[]): number {
  const n = turns.filter((t) => t.role === 'assistant').length
  return Math.max(n, 1)
}
