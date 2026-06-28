import { controller, signal, derived, effect, debounce } from '@ship/sui-runtime'
import type { Scope } from './sui-board.sui'
import { createCanvas } from '../engine/create-canvas'
import { createBoardModel, type Node, type Edge, type PortRef } from './engine/board-model'
import { applyBoardKey } from './engine/board-keys'
import type { LayoutKind } from './engine/layout-adapter'

// SSR fallback = the verified sui-combobox shape: plain JSON.parse, try/caught.
// The live path is prop:* identity onto the board model.
function parseBoard(raw: string): { nodes?: Node[]; edges?: Edge[] } | null {
  if (!raw) return null
  try {
    return JSON.parse(raw) as { nodes?: Node[]; edges?: Edge[] }
  } catch {
    return null
  }
}

// <sui-board> controller — SVG-native nodes/edges/ports on the canvas
// substrate. Geometry lives in the board model (world coords); the view maps
// to screen through the substrate transform. Node visuals are a per-node slot.
export default controller({
  scope: ({ value }: Scope) => {
    const seed = parseBoard(value())
    const board = createBoardModel({
      nodes: seed?.nodes ?? [],
      edges: seed?.edges ?? [],
    })
    const canvas = createCanvas({
      getViewportElement: () => null, // <sui-canvas> owns the viewport thunk
    })
    // Drag bookkeeping — which node is under an active face-drag.
    const draggingNode = signal<string | null>(null)
    const lastPointer = signal<{ x: number; y: number } | null>(null)
    // Outbound debounced change source: a monotonic version bumped on mutation.
    const version = signal(0)
    const changeSrc = debounce(version, 200)
    // Derived memos fold into the scope factory — the canonical form replaces the
    // historical `derived:` section (ScopedControllerDef merges only
    // scope/on/use/mount/unmount, so a top-level `derived:` is silently dropped).
    const nodes = derived(() => board.nodes())
    const routes = derived(() => board.routes())
    const selectedSet = derived(() => new Set(board.selectedNodes()))
    const incidentSet = derived(() => new Set(board.incidentEdges()))
    const pendingPath = derived(() => {
      const pe = board.pendingEdge()
      if (!pe) return ''
      const a = canvas.worldToScreen(pe.start)
      const b = canvas.worldToScreen(pe.end)
      return `M ${a.x} ${a.y} L ${b.x} ${b.y}`
    })
    return {
      board, canvas, draggingNode, lastPointer, version, changeSrc,
      nodes, routes, selectedSet, incidentSet, pendingPath,
    }
  },

  on: {
    nodePointerDown: (s, e: PointerEvent) => {
      const el = e.currentTarget as HTMLElement
      const id = el.dataset.nodeId
      if (!id) return
      // additive select with shift; otherwise replace selection.
      if (e.shiftKey) s.board.toggleSelection(id)
      else if (!s.board.selectedNodes().includes(id)) s.board.setSelection([id])
      s.draggingNode = id
      s.lastPointer = { x: e.clientX, y: e.clientY }
      // Capture on the host — the same element the pointermove/up listeners are
      // bound to (B3). Capturing on the node while listening on the host meant a
      // slotted child's stopPropagation could wedge the drag.
      ;(el.getRootNode() as ShadowRoot).host?.setPointerCapture?.(e.pointerId)
      e.stopPropagation()
    },
    portPointerDown: (s, e: PointerEvent) => {
      const el = e.currentTarget as HTMLElement
      const portId = el.dataset.portId
      const nodeEl = el.closest('[data-node-id]') as HTMLElement | null
      const nodeId = nodeEl?.dataset.nodeId
      if (!portId || !nodeId) return
      s.board.beginEdge({ node: nodeId, port: portId })
      e.stopPropagation()
    },
  },

  mount(state, host) {
    const board = state.board
    const canvas = state.canvas

    // Node-drag pump: move the dragged (or whole-selection) node by the
    // world-space delta. Listens on the host so capture survives.
    const onMove = (e: PointerEvent): void => {
      const id = state.draggingNode
      const last = state.lastPointer
      if (id && last) {
        const scale = canvas.transform().scale
        const dx = (e.clientX - last.x) / scale
        const dy = (e.clientY - last.y) / scale
        if (board.selectedNodes().includes(id)) board.moveSelectionBy(dx, dy)
        else board.moveNodeBy(id, dx, dy)
        state.lastPointer = { x: e.clientX, y: e.clientY }
        state.version = state.version + 1
        return
      }
      if (board.pendingEdge()) {
        board.updatePendingEnd(canvas.screenToWorld({ x: e.clientX, y: e.clientY }))
      }
    }
    const onUp = (e: PointerEvent): void => {
      if (state.draggingNode) {
        state.draggingNode = null
        state.lastPointer = null
      }
      const pe = board.pendingEdge()
      if (pe) {
        const tEl = (e.target as HTMLElement)?.closest?.('[data-port-id]') as HTMLElement | null
        const nodeEl = tEl?.closest('[data-node-id]') as HTMLElement | null
        const target: PortRef | null =
          tEl && nodeEl ? { node: nodeEl.dataset.nodeId!, port: tEl.dataset.portId! } : null
        const edge = target ? board.commitEdge(target) : (board.cancelEdge(), null)
        if (edge) {
          state.version = state.version + 1
          host.emit!('sui-edge-create', { source: edge.source, target: edge.target })
        }
      }
    }
    // Keyboard operability (B1) — board nodes are tabindex=0 role=group, so they
    // must be operable, not just focusable. The focused node is resolved from the
    // event target; arrow keys nudge it (or the whole selection if it is part of
    // one) in world space, Enter/Space select it, Escape clears selection. The
    // pure resolver lives in engine/board-keys; all mutations route through the
    // board model so the selection effect stays the single emit path (B2).
    const onKey = (e: KeyboardEvent): void => {
      const el = (e.target as HTMLElement | null)?.closest?.('[data-node-id]') as HTMLElement | null
      const focusedId = el?.dataset.nodeId ?? null
      const before = board.serialize()
      const acted = applyBoardKey(board, e, focusedId, canvas.transform().scale)
      if (!acted) return
      if (board.serialize() !== before) state.version = state.version + 1
      e.preventDefault()
    }
    host.addEventListener('pointermove', onMove as EventListener)
    host.addEventListener('pointerup', onUp as EventListener)
    host.addEventListener('keydown', onKey as EventListener)

    // Debounced board-change event (the KEEP'd contract: value + version).
    const stopChange = effect(() => {
      const v = state.changeSrc
      if (v === 0) return
      host.emit!('sui-board-change', { value: board.serialize(), version: v })
    })
    // Announce selection changes on the host (live-region consumers read it).
    // SINGLE emit path (B2): deduped against the last-emitted selection (the
    // sui-combobox lastEmitted pattern) and skipping the initial mount value so
    // there is no spurious empty emit and no double fire on drag-end.
    let lastEmitted: string | null = null
    const stopSel = effect(() => {
      const ids = board.selectedNodes()
      const key = ids.join(' ')
      if (lastEmitted === null) {
        lastEmitted = key // prime on mount; do not emit the initial value
        return
      }
      if (key === lastEmitted) return
      lastEmitted = key
      host.emit!('sui-node-select', { ids })
    })

    // Auto-layout: when the `layout` prop changes to a real algorithm, re-lay
    // out the board through the pure adapter and bump the change version so the
    // new geometry is announced. 'none' is a no-op (manual placement wins).
    let lastLayout: LayoutKind | null = null
    const stopLayout = effect(() => {
      const kind = (state.layout || 'none') as LayoutKind
      if (kind === lastLayout) return
      lastLayout = kind
      if (kind === 'none') return
      board.applyLayout({ layout: kind })
      state.version = state.version + 1
    })

    return () => {
      host.removeEventListener('pointermove', onMove as EventListener)
      host.removeEventListener('pointerup', onUp as EventListener)
      host.removeEventListener('keydown', onKey as EventListener)
      stopChange()
      stopSel()
      stopLayout()
      canvas.dispose()
    }
  },
})
