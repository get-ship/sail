import type { ClientOptions } from '@ship/sail-studio-client'
import { sailAuthoringEdit, sailAuthoringCheck, sailAuthoringLayoutCheck, sailInspectModel, sailInspectState, sailInspectTree, sailInspectTokenUsage, sailRenderScreenshot, sailTaskList, sailTaskGet, sailTaskCreate, sailTaskComment, sailTaskAddArtifact, sailTaskAgentNote, sailTaskThreadCreate, sailTaskThreadReply, sailTaskThreadList, sailTaskReanchor, sailTaskCommitIntent, sailThemeFork, sailThemeList, sailThemeRead, sailThemeWrite, sailThemeDelete, sailThemeGenerate, sailThemeGraph, sailThemeEffectiveIds, sailThemeFontAdd, sailThemeFontSearch } from '../../../../packages/sail/studio-client/src/index.ts'
import type { OpsClient } from './types'

async function writeSourceHttp(
  path: string,
  content: string,
  opts?: ClientOptions,
): Promise<void> {
  const base = opts?.baseUrl ?? ''
  // Prefer injected fetch; fall back to host global (not bare ambient `fetch` id).
  const fetchFn = opts?.fetch ?? globalThis.fetch.bind(globalThis)
  const res = await fetchFn(`${base}/sail.ops.v1.SailAuthoring/Edit`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({
      path,
      source: content,
      ops: [],
      writeSource: true,
      actor: 'human:studio',
    }),
  })
  if (!res.ok) {
    const text = await res.text().catch(() => '')
    throw new Error(
      `writeSource HTTP ${res.status} for ${path}: ${text || res.statusText}`,
    )
  }
  const body = (await res.json()) as {
    applied?: boolean
    message?: string
    error?: string
  }
  if (body.applied === false) {
    throw new Error(
      `writeSource not applied for ${path}: ${body.message || body.error || 'rejected'}`,
    )
  }
}

async function listFilesHttp(
  args: { project?: string },
  opts?: ClientOptions,
): Promise<{ ok: boolean; project: string; files: string[] }> {
  const base = opts?.baseUrl ?? ''
  const fetchFn = opts?.fetch ?? globalThis.fetch.bind(globalThis)
  const res = await fetchFn(`${base}/sail.ops.v1.SailAuthoring/ListFiles`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ project: args.project }),
  })
  if (!res.ok) {
    const text = await res.text().catch(() => '')
    throw new Error(`ListFiles HTTP ${res.status}: ${text || res.statusText}`)
  }
  return (await res.json()) as { ok: boolean; project: string; files: string[] }
}

export function createOpsClient(opts?: ClientOptions): OpsClient {
  return {
    sailInspectModel: (args) => sailInspectModel(args, opts),
    sailInspectTree: (args) => sailInspectTree(args, opts),
    sailInspectState: (args) => sailInspectState(args, opts),
    sailInspectTokenUsage: (args) => sailInspectTokenUsage(args, opts),
    sailRenderScreenshot: (args) => sailRenderScreenshot(args, opts),
    sailAuthoringEdit: (args) => sailAuthoringEdit(args, opts),
    sailAuthoringCheck: (args) => sailAuthoringCheck(args, opts),
    sailAuthoringLayoutCheck: (args) => sailAuthoringLayoutCheck(args, opts),
    sailAuthoringListFiles: (args) => listFilesHttp(args, opts),
    sailTaskList: (args) => sailTaskList(args, opts),
    sailTaskGet: (args) => sailTaskGet(args, opts),
    sailTaskCreate: (args) => sailTaskCreate(args, opts),
    sailTaskComment: (args) => sailTaskComment(args, opts),
    sailTaskAddArtifact: (args) => sailTaskAddArtifact(args, opts),
    sailTaskAgentNote: (args) => sailTaskAgentNote(args, opts),
    sailTaskThreadCreate: (args) => sailTaskThreadCreate(args, opts),
    sailTaskThreadReply: (args) => sailTaskThreadReply(args, opts),
    sailTaskThreadList: (args) => sailTaskThreadList(args, opts),
    sailTaskReanchor: (args) => sailTaskReanchor(args, opts),
    sailTaskCommitIntent: (args) => sailTaskCommitIntent(args, opts),
    sailThemeList: (args) => sailThemeList(args, opts),
    sailThemeRead: (args) => sailThemeRead(args, opts),
    sailThemeWrite: (args) => sailThemeWrite(args, opts),
    sailThemeDelete: (args) => sailThemeDelete(args, opts),
    sailThemeFork: (args) => sailThemeFork(args, opts),
    sailThemeGenerate: (args) => sailThemeGenerate(args, opts),
    sailThemeGraph: (args) => sailThemeGraph(args, opts),
    sailThemeEffectiveIds: (args) => sailThemeEffectiveIds(args, opts),
    sailThemeFontAdd: (args) => sailThemeFontAdd(args, opts),
    sailThemeFontSearch: (args) => sailThemeFontSearch(args, opts),
    readSource: async (path: string) => {
      throw new Error(
        `readSource unavailable in browser for ${path} — open via SailInspect/Model.source`,
      )
    },
    writeSource: (path, content) => writeSourceHttp(path, content, opts),
  }
}
