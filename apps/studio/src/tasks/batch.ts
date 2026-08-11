import type { BatchItem } from '@ship/sail-studio-shell'
import { clearBatch, getBatchItems, removeBatchItem } from '../../../../packages/sail/studio-shell/src/index.ts'
import {
  createThread,
  getTasksOpsClient,
  getTasksState,
  loadThreads,
  projectRootArg,
  recordBatchCount,
  selectThread,
} from './store'
import { rememberArtifactThumb } from './thumbs'

/** Attach batch tray items as render artifacts on a task (or create one). */
export async function newTaskFromBatch(title?: string): Promise<string | null> {
  const items = getBatchItems()
  const client = getTasksOpsClient()
  if (!items.length || !client) return null
  const first = items[0]
  const node = first.nodes[0]
  const anchor = {
    nodeId: node?.nodeId ?? `batch-${first.id}`,
    elementPath: node?.label ? String(node.label) : `Selection · ${first.kind}`,
    contentHash: (node?.nodeId ?? first.id).split('#').pop() ?? first.id,
    file: node?.nodeId?.includes('.sail') ? node.nodeId.split('#')[0] : undefined,
  }
  const id = await createThread({
    title: title?.trim() || `Collected · ${items.length} items`,
    body: `Collected ${items.length} item${items.length === 1 ? '' : 's'} to share with a task.`,
    anchor,
  })
  if (!id) return null
  await attachBatchItems(id, items)
  clearBatch()
  recordBatchCount(items.length)
  await selectThread(id)
  await loadThreads()
  return id
}

export async function sendBatchToExisting(taskId: string): Promise<void> {
  const items = getBatchItems()
  const client = getTasksOpsClient()
  if (!items.length || !client || !taskId) return
  await attachBatchItems(taskId, items)
  clearBatch()
  recordBatchCount(items.length)
  await selectThread(taskId)
  await loadThreads()
}

/**
 * Attach the newest batch-tray crop to the selected (or given) thread.
 * One gesture: tray → thread. Leaves remaining tray items in place.
 */
export async function attachLatestCrop(taskId?: string): Promise<boolean> {
  const items = getBatchItems()
  const client = getTasksOpsClient()
  const id = taskId ?? getTasksState().selectedId
  if (!items.length || !client || !id) return false
  const latest = items[items.length - 1]!
  await attachBatchItems(id, [latest])
  removeBatchItem(latest.id)
  recordBatchCount(1)
  await selectThread(id)
  await loadThreads()
  return true
}

async function attachBatchItems(taskId: string, items: BatchItem[]): Promise<void> {
  const client = getTasksOpsClient()
  if (!client?.sailTaskAddArtifact) return
  for (let i = 0; i < items.length; i++) {
    const it = items[i]
    const png = it.imagePng ?? ''
    // Minimal PNG-less payload so offline tests still attach files.
    // Prefer real crop bytes when the tray has them (evidence thumbs).
    const content =
      png ||
      (typeof btoa !== 'undefined'
        ? btoa(`batch-${it.id}`)
        : Buffer.from(`batch-${it.id}`).toString('base64'))
    // sailTaskAddArtifact expects raw base64, not a data: URL.
    const contentB64 = content.startsWith('data:')
      ? content.replace(/^data:[^;]+;base64,/, '')
      : content
    const node = it.nodes[0]
    const ret = await client.sailTaskAddArtifact({
      project: projectRootArg(),
      id: taskId,
      filename: `${it.kind}-${i + 1}.png`,
      content_base64: contentB64,
      mime: 'image/png',
      kind: 'render',
      label:
        it.kind === 'screenshot'
          ? `Screenshot · ${it.rect.w | 0}px`
          : node?.label ?? `Selection · ${it.id}`,
      region: {
        nodeId: node?.nodeId ?? null,
        label: node?.label ?? null,
        rect: it.rect,
        kind: it.kind,
      },
      actor: { kind: 'human', name: 'user' },
    })
    // Cache evidence thumb for <img> — same bytes just written.
    const artId = ret.artifact_id ? String(ret.artifact_id) : ''
    const artPath = ret.path ? String(ret.path) : ''
    if (artId) rememberArtifactThumb(artId, 'image/png', contentB64)
    if (artPath) rememberArtifactThumb(artPath, 'image/png', contentB64)
  }
}
