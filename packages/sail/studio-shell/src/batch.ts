
import type { BridgeRect } from './channel'
import type { RegionNode, ResolvedRegion } from './regions'
import { resolveRegion } from './regions'

export type BatchKind = 'annotate' | 'screenshot'

export type BatchItem = {
  id: string
  kind: BatchKind
  rect: BridgeRect
  nodes: RegionNode[]
  fromRender: boolean
  imagePng?: string
  createdAt: number
}

type Listener = (items: BatchItem[]) => void

let items: BatchItem[] = []
const listeners = new Set<Listener>()
let seq = 0

function emit(): void {
  for (const fn of listeners) fn(items.slice())
}

export function getBatchItems(): BatchItem[] {
  return items.slice()
}

export function subscribeBatch(fn: Listener): () => void {
  listeners.add(fn)
  return () => {
    listeners.delete(fn)
  }
}

export function clearBatch(): void {
  items = []
  emit()
}

export function removeBatchItem(id: string): void {
  items = items.filter((i) => i.id !== id)
  emit()
}

export async function addRegionToBatch(
  kind: BatchKind,
  rect: BridgeRect,
  opts?: {
    sailPath?: string | null
    viewport?: { width: number; height: number }
    theme?: string
    mode?: string
  },
): Promise<BatchItem> {
  const resolved: ResolvedRegion = await resolveRegion(rect, opts)
  const item: BatchItem = {
    id: `batch-${++seq}`,
    kind,
    rect: resolved.rect,
    nodes: resolved.nodes,
    fromRender: resolved.fromRender,
    createdAt: Date.now(),
    ...(resolved.imagePng !== undefined ? { imagePng: resolved.imagePng } : {}),
  }
  items = [...items, item]
  emit()
  return item
}

export function __resetBatchForTests(): void {
  items = []
  listeners.clear()
  seq = 0
}
