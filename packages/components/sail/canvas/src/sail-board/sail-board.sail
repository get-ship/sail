<!--
  <sail-board> — SVG-native node/edge/port diagram on the canvas substrate.

  Builds createCanvas() + a board model in the controller. Nodes/edges/ports/
  selection are DOM/SVG (selectable, accessible, token-themeable) — NO canvas
  engine. The board composes <sail-canvas> as its substrate via prop:canvas
  identity and renders its domain layer into the world slot. Edges live in an
  SVG overlay; nodes are absolutely-positioned <div>s carrying a default slot
  for consumer-owned visuals (the board owns geometry, not content).

  Props
  ─────────────────────────────────────────────────────────────────────
  value     (string, '')              SSR/declarative board-JSON FALLBACK
                                        (parsed via JSON.parse). prop:* wins.
  readonly  (bool, false reflect)      Pan/zoom only; no node edits.
  theme     ('light'|'dark' reflect)   Forwarded to the substrate.
  grid      ('none'|'dots'|'lines')    Forwarded background grid hint.
  layout    ('none'|'layered'|'force') Auto-layout algorithm (Slice 7).
  label     (string, '')               → aria-label on the application region.
  ─────────────────────────────────────────────────────────────────────
  Events (host.emit!):
    sail-board-change { value, version } — debounced board mutation.
    sail-node-select  { ids }            — selection changed.
    sail-edge-create  { source, target } — drag-from-port created an edge.
-->

<scope>
  props: {
    value = ''
    readonly = false
    theme: 'light' | 'dark' = 'light'
    grid: 'none' | 'dots' | 'lines' = 'none'
    layout: 'none' | 'layered' | 'force' = 'none'
    label = ''
  }
</scope>
<controller>
import { controller, signal, derived, effect, debounce } from '@ship/sail-runtime'
import type { Scope } from './sail-board.sail'
import { createCanvas } from '../engine/create-canvas'
import { createBoardModel, type Node, type Edge, type PortRef } from './engine/board-model'
import { applyBoardKey } from './engine/board-keys'
import type { LayoutKind } from './engine/layout-adapter'

// SSR fallback = the verified sail-combobox shape: plain JSON.parse, try/caught.
// The live path is prop:* identity onto the board model.
function parseBoard(raw: string): { nodes?: Node[]; edges?: Edge[] } | null {
  if (!raw) return null
  try {
    return JSON.parse(raw) as { nodes?: Node[]; edges?: Edge[] }
  } catch {
    return null
  }
}

// <sail-board> controller — SVG-native nodes/edges/ports on the canvas
// substrate. Geometry lives in the board model (world coords); the view maps
// to screen through the substrate transform. Node visuals are a per-node slot.
export default controller(({ value, layout }: Scope) => {
  const seed = parseBoard(value())
  const board = createBoardModel({
    nodes: seed?.nodes ?? [],
    edges: seed?.edges ?? [],
  })
  const canvas = createCanvas({
    getViewportElement: () => null, // <sail-canvas> owns the viewport thunk
  })
  // Drag bookkeeping — which node is under an active face-drag.
  const draggingNode = signal<string | null>(null)
  const lastPointer = signal<{ x: number; y: number } | null>(null)
  // Outbound debounced change source: a monotonic version bumped on mutation.
  const version = signal(0)
  const changeSrc = debounce(version, 200)
  // Derived memos are plain statements in the setup body now — no `derived:`
  // section to fold them into.
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

  const nodePointerDown = (e: PointerEvent): void => {
    const el = e.currentTarget as HTMLElement
    const id = el.dataset.nodeId
    if (!id) return
    // additive select with shift; otherwise replace selection.
    if (e.shiftKey) board.toggleSelection(id)
    else if (!board.selectedNodes().includes(id)) board.setSelection([id])
    draggingNode.set(id)
    lastPointer.set({ x: e.clientX, y: e.clientY })
    // Capture on the host — the same element the pointermove/up listeners are
    // bound to (B3). Capturing on the node while listening on the host meant a
    // slotted child's stopPropagation could wedge the drag.
    ;(el.getRootNode() as ShadowRoot).host?.setPointerCapture?.(e.pointerId)
    e.stopPropagation()
  }
  const portPointerDown = (e: PointerEvent): void => {
    const el = e.currentTarget as HTMLElement
    const portId = el.dataset.portId
    const nodeEl = el.closest('[data-node-id]') as HTMLElement | null
    const nodeId = nodeEl?.dataset.nodeId
    if (!portId || !nodeId) return
    board.beginEdge({ node: nodeId, port: portId })
    e.stopPropagation()
  }

  mount((host) => {
    // Node-drag pump: move the dragged (or whole-selection) node by the
    // world-space delta. Listens on the host so capture survives.
    const onMove = (e: PointerEvent): void => {
      const id = draggingNode()
      const last = lastPointer()
      if (id && last) {
        const scale = canvas.transform().scale
        const dx = (e.clientX - last.x) / scale
        const dy = (e.clientY - last.y) / scale
        if (board.selectedNodes().includes(id)) board.moveSelectionBy(dx, dy)
        else board.moveNodeBy(id, dx, dy)
        lastPointer.set({ x: e.clientX, y: e.clientY })
        version.set(version() + 1)
        return
      }
      if (board.pendingEdge()) {
        board.updatePendingEnd(canvas.screenToWorld({ x: e.clientX, y: e.clientY }))
      }
    }
    const onUp = (e: PointerEvent): void => {
      if (draggingNode()) {
        draggingNode.set(null)
        lastPointer.set(null)
      }
      const pe = board.pendingEdge()
      if (pe) {
        const tEl = (e.target as HTMLElement)?.closest?.('[data-port-id]') as HTMLElement | null
        const nodeEl = tEl?.closest('[data-node-id]') as HTMLElement | null
        const target: PortRef | null =
          tEl && nodeEl ? { node: nodeEl.dataset.nodeId!, port: tEl.dataset.portId! } : null
        const edge = target ? board.commitEdge(target) : (board.cancelEdge(), null)
        if (edge) {
          version.set(version() + 1)
          host.emit!('sail-edge-create', { source: edge.source, target: edge.target })
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
      if (board.serialize() !== before) version.set(version() + 1)
      e.preventDefault()
    }
    host.addEventListener('pointermove', onMove as EventListener)
    host.addEventListener('pointerup', onUp as EventListener)
    host.addEventListener('keydown', onKey as EventListener)

    // Debounced board-change event (the KEEP'd contract: value + version).
    const stopChange = effect(() => {
      const v = changeSrc()
      if (v === 0) return
      host.emit!('sail-board-change', { value: board.serialize(), version: v })
    })
    // Announce selection changes on the host (live-region consumers read it).
    // SINGLE emit path (B2): deduped against the last-emitted selection (the
    // sail-combobox lastEmitted pattern) and skipping the initial mount value so
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
      host.emit!('sail-node-select', { ids })
    })

    // Auto-layout: when the `layout` prop changes to a real algorithm, re-lay
    // out the board through the pure adapter and bump the change version so the
    // new geometry is announced. 'none' is a no-op (manual placement wins).
    let lastLayout: LayoutKind | null = null
    const stopLayout = effect(() => {
      const kind = (layout() || 'none') as LayoutKind
      if (kind === lastLayout) return
      lastLayout = kind
      if (kind === 'none') return
      board.applyLayout({ layout: kind })
      version.set(version() + 1)
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
  })

  return {
    board, canvas, draggingNode, lastPointer, version, changeSrc,
    nodes, routes, selectedSet, incidentSet, pendingPath,
    nodePointerDown, portPointerDown,
  }
})

</controller>


<sail-canvas
  role="application"
  aria-label={label()}
  prop:canvas={canvas}
  prop:readonly={readonly()}
  sx="block relative w-full h-full"
>
  <svg aria-hidden="true" sx="absolute top-0 left-0 overflow-visible pointer-events-none">
    {#for r of routes() key={r.id}}
      <path d={r.path} data-edge-id={r.id} data-state={incidentSet().has(r.id) ? 'incident' : null} sx="fill-none stroke-border state-incident:stroke-primary" stroke-width="2"></path>
    {/for}
    {#if pendingPath()}
      <path d={pendingPath()} sx="fill-none stroke-primary" stroke-width="2" stroke-dasharray="6 4"></path>
    {/if}
  </svg>

  {#for n of nodes() key={n.id}}
    <div
      role="group"
      tabindex="0"
      aria-label={n.id}
      aria-selected={selectedSet().has(n.id) ? 'true' : 'false'}
      data-node-id={n.id}
      use:rect={{top: n.rect.y, left: n.rect.x, width: n.rect.w, height: n.rect.h}}
      on:pointerdown={nodePointerDown}
      sx="box-border bg-card text-card-foreground border border-border rounded-md p-2 cursor-grab select-none outline-none focus:outline-solid focus:outline-2 focus:outline-ring focus:outline-offset-2 selected:border-primary selected:ring-primary"
    >
      <slot name={n.id}></slot>
      {#for p of n.ports key={p.id}}
        <span
          data-port-id={p.id}
          data-state={p.side}
          use:portEdge={p.side}
          on:pointerdown={portPointerDown}
          sx="absolute w-3 h-3 rounded-full bg-background border border-primary cursor-crosshair state-left:left-(--port-left) state-left:top-(--port-top) state-right:right-(--port-right) state-right:top-(--port-top) state-top:top-(--port-top) state-top:left-(--port-left) state-bottom:bottom-(--port-bottom) state-bottom:left-(--port-left)"
        ></span>
      {/for}
    </div>
  {/for}
</sail-canvas>
