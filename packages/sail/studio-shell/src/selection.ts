/**
 * Shared Studio selection — one node id, everywhere.
 *
 * Surfaces (explorer, canvas, checks, threads) read/write this store.
 * Bridge selection pings land here via `selectFromBridge`.
 */

export type Selection = {
  /** Span-keyed Sail node id (T-I3). Bridge may pass a CSS selector until inspect resolves. */
  nodeId: string | null
  /** Optional locate-live selector from the bridge (not assessed). */
  selector?: string | null
  source: 'surface' | 'bridge' | 'clear'
}

type Listener = (sel: Selection) => void

let current: Selection = { nodeId: null, selector: null, source: 'clear' }
const listeners = new Set<Listener>()

export function getSelection(): Selection {
  return current
}

export function subscribeSelection(fn: Listener): () => void {
  listeners.add(fn)
  return () => {
    listeners.delete(fn)
  }
}

function emit(): void {
  for (const fn of listeners) fn(current)
}

/** Select from any Studio surface (explorer, checks, …). */
export function selectNode(nodeId: string | null, source: 'surface' | 'clear' = 'surface'): void {
  current = { nodeId, selector: null, source: nodeId ? source : 'clear' }
  emit()
}

/** Bridge locate-live ping — node id may be a provisional selector until ops resolve. */
export function selectFromBridge(nodeId: string | null, selector?: string | null): void {
  current = {
    nodeId,
    selector: selector ?? null,
    source: nodeId ? 'bridge' : 'clear',
  }
  emit()
}

export function clearSelection(): void {
  selectNode(null, 'clear')
}

/** Test helper. */
export function __resetSelectionForTests(): void {
  current = { nodeId: null, selector: null, source: 'clear' }
  listeners.clear()
}
