/**
 * Node-side OpsClient: shell out to `sail op` (no HTTP server required).
 * Used by unit tests only — do not import from browser routes.
 */

import type { OpsClient } from '../../src/edit/types'

export function createCliOpsClient(repoRoot: string): OpsClient {
  const run = async (serviceMethod: string, args: object) => {
    const { spawnSync } = await import('node:child_process')
    const { existsSync } = await import('node:fs')
    const { join } = await import('node:path')
    const candidates = [
      join(repoRoot, 'target/debug/sail'),
      join(repoRoot, 'target/release/sail'),
    ]
    const bin = candidates.find((p) => existsSync(p))
    const r = bin
      ? spawnSync(bin, ['op', serviceMethod, '--json', JSON.stringify(args)], {
          cwd: repoRoot,
          encoding: 'utf8',
          maxBuffer: 32 * 1024 * 1024,
          env: { ...process.env },
        })
      : spawnSync(
          'cargo',
          [
            'run',
            '-q',
            '-p',
            'deck-cli',
            '--bin',
            'sail',
            '--',
            'op',
            serviceMethod,
            '--json',
            JSON.stringify(args),
          ],
          {
            cwd: repoRoot,
            encoding: 'utf8',
            maxBuffer: 32 * 1024 * 1024,
            env: { ...process.env },
          },
        )
    if (r.status !== 0) {
      throw new Error(
        `sail op ${serviceMethod} failed (${r.status}): ${r.stderr || r.stdout}`,
      )
    }
    const out = r.stdout.trim()
    const start = out.indexOf('{')
    if (start < 0) throw new Error(`no JSON from sail op ${serviceMethod}`)
    return JSON.parse(out.slice(start))
  }

  return {
    sailInspectModel: (args) => run('SailInspect/Model', args),
    sailInspectTree: (args) => run('SailInspect/Tree', args),
    sailInspectState: (args) => run('SailInspect/State', args),
    sailInspectTokenUsage: (args) => run('SailInspect/TokenUsage', args),
    sailRenderScreenshot: (args) => run('SailRender/Screenshot', args),
    sailAuthoringEdit: (args) => run('SailAuthoring/Edit', args),
    sailAuthoringCheck: (args) => run('SailAuthoring/Check', args),
    sailAuthoringLayoutCheck: (args) => run('SailAuthoring/LayoutCheck', args),
    sailTaskList: (args) => run('SailTask/List', args),
    sailTaskGet: (args) => run('SailTask/Get', args),
    sailTaskCreate: (args) => run('SailTask/Create', args),
    sailTaskComment: (args) => run('SailTask/Comment', args),
    sailTaskAddArtifact: (args) => run('SailTask/AddArtifact', args),
    sailTaskAgentNote: (args) => run('SailTask/AgentNote', args),
    sailTaskThreadCreate: (args) => run('SailTask/ThreadCreate', args),
    sailTaskThreadReply: (args) => run('SailTask/ThreadReply', args),
    sailTaskThreadList: (args) => run('SailTask/ThreadList', args),
    sailTaskReanchor: (args) => run('SailTask/Reanchor', args),
    sailTaskCommitIntent: (args) => run('SailTask/CommitIntent', args),
    sailThemeList: (args) => run('SailTheme/List', args),
    sailThemeRead: (args) => run('SailTheme/Read', args),
    sailThemeWrite: (args) => run('SailTheme/Write', args),
    sailThemeDelete: (args) => run('SailTheme/Delete', args),
    sailThemeFork: (args) => run('SailTheme/Fork', args),
    sailThemeGenerate: (args) => run('SailTheme/Generate', args),
    sailThemeGraph: (args) => run('SailTheme/Graph', args),
    sailThemeEffectiveIds: (args) => run('SailTheme/EffectiveIds', args),
    readSource: async (path: string) => {
      const { readFileSync } = await import('node:fs')
      const { join } = await import('node:path')
      const abs = path.startsWith('/') ? path : join(repoRoot, path)
      return readFileSync(abs, 'utf8')
    },
    writeSource: async (path: string, content: string) => {
      const { writeFileSync, mkdirSync } = await import('node:fs')
      const { join, dirname } = await import('node:path')
      const abs = path.startsWith('/') ? path : join(repoRoot, path)
      mkdirSync(dirname(abs), { recursive: true })
      writeFileSync(abs, content, 'utf8')
    },
  }
}
