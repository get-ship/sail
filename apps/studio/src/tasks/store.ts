import {
  clearBatch,
  getBatchItems,
  removeBatchItem,
} from '@ship/sail-studio-shell'
import { parseListItem, parseThread } from './parse'
import { __resetThumbsForTests } from './thumbs'
import type {
  ActorFilter,
  DriftFilter,
  OpenFilter,
  TasksOpsClient,
  TasksSurfaceState,
} from './types'
import { userError } from '../user-error'

type Listener = (s: TasksSurfaceState) => void

let client: TasksOpsClient | null = null
let state: TasksSurfaceState = emptyState()
const listeners = new Set<Listener>()

function emptyState(): TasksSurfaceState {
  return {
    projectRoot: '',
    threads: [],
    selectedId: null,
    selected: null,
    openFilter: 'open',
    actorFilter: 'all',
    driftFilter: 'all',
    replyDraft: '',
    loading: false,
    error: null,
    commitPath: null,
    lastBatchCount: 0,
  }
}

function emit(): void {
  const snap = { ...state }
  for (const fn of listeners) fn(snap)
}

export function setTasksOpsClient(c: TasksOpsClient | null): void {
  client = c
}

export function getTasksOpsClient(): TasksOpsClient | null {
  return client
}

export function setTasksProjectRoot(root: string): void {
  state = { ...state, projectRoot: root }
  emit()
}

export function getTasksState(): TasksSurfaceState {
  return state
}

export function projectRootArg(): string | undefined {
  return state.projectRoot || undefined
}

export function recordBatchCount(n: number): void {
  state = { ...state, lastBatchCount: n }
  emit()
}

export function subscribeTasks(fn: Listener): () => void {
  listeners.add(fn)
  fn(state)
  return () => {
    listeners.delete(fn)
  }
}

export function __resetTasksForTests(): void {
  client = null
  state = emptyState()
  listeners.clear()
  __resetThumbsForTests()
}

function projectArg(): string | undefined {
  return projectRootArg()
}

function humanActor() {
  return { kind: 'human', name: 'user' }
}

export async function loadThreads(): Promise<void> {
  if (!client) {
    // No ops client yet — empty threads until chrome boots the client.
    state = { ...state, error: null, loading: false, threads: [] }
    emit()
    return
  }
  state = { ...state, loading: true, error: null }
  emit()
  try {
    const args: {
      project?: string
      actor?: string
      drift?: string
      open?: boolean
    } = { project: projectArg() }
    if (state.actorFilter !== 'all') args.actor = state.actorFilter
    if (state.driftFilter !== 'all') args.drift = state.driftFilter
    if (state.openFilter === 'open') args.open = true

    const listFn = client.sailTaskThreadList ?? client.sailTaskList
    if (!listFn) throw new Error('Could not load tasks — try again.')
    const ret = await listFn(args)
    const threads = (ret.tasks ?? [])
      .map(parseListItem)
      .filter((t): t is NonNullable<typeof t> => t != null)
    state = { ...state, threads, loading: false }
    if (state.selectedId && !threads.some((t) => t.id === state.selectedId)) {
      state = { ...state, selectedId: null, selected: null }
    }
    emit()
    if (state.selectedId) await selectThread(state.selectedId)
  } catch (e) {
    state = {
      ...state,
      loading: false,
      error: userError(e, 'Could not load tasks — try again.'),
    }
    emit()
  }
}

export async function selectThread(id: string | null): Promise<void> {
  if (!id) {
    state = { ...state, selectedId: null, selected: null, replyDraft: '', commitPath: null }
    emit()
    return
  }
  if (!client?.sailTaskGet && !client) {
    state = { ...state, selectedId: id, selected: null }
    emit()
    return
  }
  state = { ...state, selectedId: id, loading: true, error: null }
  emit()
  try {
    const getFn = client!.sailTaskGet
    if (!getFn) throw new Error('Could not open that thread — try again.')
    const ret = await getFn({ project: projectArg(), id })
    const selected = parseThread(ret.task)
    state = { ...state, selected, loading: false }
    emit()
  } catch (e) {
    state = {
      ...state,
      loading: false,
      error: userError(e, 'Could not open that thread — try again.'),
    }
    emit()
  }
}

export function setOpenFilter(f: OpenFilter): void {
  state = { ...state, openFilter: f }
  emit()
  void loadThreads()
}

export function setActorFilter(f: ActorFilter): void {
  state = { ...state, actorFilter: f }
  emit()
  void loadThreads()
}

export function setDriftFilter(f: DriftFilter): void {
  state = { ...state, driftFilter: f }
  emit()
  void loadThreads()
}

export function setReplyDraft(text: string): void {
  state = { ...state, replyDraft: text }
  emit()
}

/**
 * Human reply. `@agent …` is a handoff *request* attributed to the human
 * (comment body kept intact). Actual agent progress uses AgentNote / ThreadReply
 * with `actor.kind=agent` from MCP or an agent client — never mis-label a human.
 */
export async function sendReply(): Promise<void> {
  const id = state.selectedId
  const body = state.replyDraft.trim()
  if (!id || !body || !client) return
  const replyFn = client.sailTaskThreadReply ?? client.sailTaskComment
  if (!replyFn) return
  try {
    await replyFn({
      project: projectArg(),
      id,
      body,
      actor: humanActor(),
    })
    state = { ...state, replyDraft: '' }
    emit()
    await selectThread(id)
    await loadThreads()
  } catch (e) {
    state = {
      ...state,
      error: userError(e, 'Could not send reply — try again.'),
    }
    emit()
  }
}

/** Agent progress note (kind=agent only). Studio/MCP agents call this, not humans. */
export async function sendAgentNote(body: string): Promise<void> {
  const id = state.selectedId
  const text = body.trim()
  if (!id || !text || !client?.sailTaskAgentNote) return
  try {
    await client.sailTaskAgentNote({
      project: projectArg(),
      id,
      body: text,
      actor: { kind: 'agent', name: 'agent' },
    })
    await selectThread(id)
    await loadThreads()
  } catch (e) {
    state = {
      ...state,
      error: userError(e, 'Could not post that note — try again.'),
    }
    emit()
  }
}

export async function createThread(opts: {
  title: string
  body?: string
  anchor: Record<string, unknown>
}): Promise<string | null> {
  if (!client) return null
  const createFn = client.sailTaskThreadCreate ?? client.sailTaskCreate
  if (!createFn) return null
  try {
    const ret = await createFn({
      project: projectArg(),
      title: opts.title,
      body: opts.body,
      anchor: opts.anchor,
      actor: humanActor(),
    })
    const task = parseThread(ret.task)
    const id = task?.id ?? null
    await loadThreads()
    if (id) await selectThread(id)
    return id
  } catch (e) {
    state = {
      ...state,
      error: userError(e, 'Could not start that task — try again.'),
    }
    emit()
    return null
  }
}

/** Persist a short decision note for the selected thread. */
export async function commitIntent(): Promise<string | null> {
  const id = state.selectedId
  if (!id || !client?.sailTaskCommitIntent) return null
  try {
    const ret = await client.sailTaskCommitIntent({
      project: projectArg(),
      id,
      actor: humanActor(),
    })
    const path = ret.path ? String(ret.path) : null
    state = { ...state, commitPath: path }
    emit()
    await selectThread(id)
    return path
  } catch (e) {
    state = {
      ...state,
      error: userError(e, 'Could not save that decision — try again.'),
    }
    emit()
    return null
  }
}

export async function reanchorSelected(anchor: Record<string, unknown>): Promise<void> {
  const id = state.selectedId
  if (!id || !client?.sailTaskReanchor) return
  try {
    await client.sailTaskReanchor({
      project: projectArg(),
      id,
      anchor,
      actor: humanActor(),
    })
    await selectThread(id)
    await loadThreads()
  } catch (e) {
    state = {
      ...state,
      error: userError(e, 'Could not re-link — try again.'),
    }
    emit()
  }
}

/** Accept resolved live location when drift is `moved` (explicit re-link only). */
export async function acceptLiveNode(): Promise<void> {
  const t = state.selected
  const d = t?.drift
  if (!t || d?.status !== 'moved' || !d.liveNodeId) {
    state = {
      ...state,
      error: 'That linked spot has not moved, or no new location is available.',
    }
    emit()
    return
  }
  const hash =
    d.liveHash ?? d.liveNodeId.split('#').pop() ?? t.anchor?.contentHash ?? 'unknown'
  await reanchorSelected({
    nodeId: d.liveNodeId,
    elementPath: t.anchor?.elementPath ?? '',
    contentHash: hash,
    file: t.anchor?.file,
    span: d.span ?? undefined,
  })
}

/**
 * Explicit re-bind to current shell selection (lost or moved recovery).
 * Never silent — requires a selected nodeId from the caller.
 */
export async function reanchorToNode(opts: {
  nodeId: string
  elementPath?: string
  contentHash?: string
  file?: string
  span?: Record<string, unknown>
}): Promise<void> {
  const nodeId = opts.nodeId?.trim()
  if (!nodeId) {
    state = { ...state, error: 'Select an element in Edit to re-link' }
    emit()
    return
  }
  const contentHash = opts.contentHash ?? nodeId.split('#').pop() ?? 'unknown'
  const file =
    opts.file ??
    (nodeId.includes('.sail') ? nodeId.split('#')[0] : state.selected?.anchor?.file)
  const elementPath =
    opts.elementPath ??
    state.selected?.anchor?.elementPath ??
    (file ? `${file.split('/').pop()} › ${nodeId}` : nodeId)
  await reanchorSelected({
    nodeId,
    elementPath,
    contentHash,
    file,
    span: opts.span,
  })
}

export { getBatchItems, removeBatchItem, clearBatch }
