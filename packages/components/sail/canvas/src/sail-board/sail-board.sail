<!--
  <sail-board> — SVG node/edge/port diagram on <sail-canvas> substrate.
  Board model owns geometry; node faces are slotted. Events: sail-board-change,
  sail-node-select, sail-edge-create.
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

import type { Scope } from './sail-board.sail'
import { createCanvas } from '../engine/create-canvas'
import { createBoardModel, type Node, type Edge, type PortRef } from './engine/board-model'
import { applyBoardKey } from './engine/board-keys'
import type { LayoutKind } from './engine/layout-adapter'

// SSR JSON fallback; live path is identity onto the board model.
function parseBoard(raw: string): { nodes?: Node[]; edges?: Edge[] } | null {
  if (!raw) return null
  try {
    return JSON.parse(raw) as { nodes?: Node[]; edges?: Edge[] }
  } catch {
    return null
  }
}

export default controller(({ value, layout }: Scope) => {
  const seed = parseBoard(value())
  const board = createBoardModel({
    nodes: seed?.nodes ?? [],
    edges: seed?.edges ?? [],
  })
  const canvas = createCanvas({
    getViewportElement: () => null,
  })
  const draggingNode = signal<string | null>(null)
  const lastPointer = signal<{ x: number; y: number } | null>(null)
  const version = signal(0)
  const changeSrc = debounce(version, 200)
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

  const nodePointerDown = (e: DomEv): void => {
    const el = e.currentTarget as HostBag
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
    ;(el).setPointerCapture?.(e.pointerId)
    e.stopPropagation()
  }
  const portPointerDown = (e: DomEv): void => {
    const el = e.currentTarget as HostBag
    const portId = el.dataset.portId
    const nodeEl = el.closest('[data-node-id]') as HostBag | null
    const nodeId = nodeEl?.dataset.nodeId
    if (!portId || !nodeId) return
    board.beginEdge({ node: nodeId, port: portId })
    e.stopPropagation()
  }

  mount((host) => {
    // Node-drag pump: move the dragged (or whole-selection) node by the
    // world-space delta. Listens on the host so capture survives.
    const onMove = (e: DomEv): void => {
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
    const onUp = (e: DomEv): void => {
      if (draggingNode()) {
        draggingNode.set(null)
        lastPointer.set(null)
      }
      const pe = board.pendingEdge()
      if (pe) {
        const tEl = (e.target)?.closest?.('[data-port-id]') as HostBag | null
        const nodeEl = tEl?.closest('[data-node-id]') as HostBag | null
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
    const onKey = (e: DomEv): void => {
      const el = (e.target as HostBag | null)?.closest?.('[data-node-id]') as HostBag | null
      const focusedId = el?.dataset.nodeId ?? null
      const before = board.serialize()
      const acted = applyBoardKey(board, e, focusedId, canvas.transform().scale)
      if (!acted) return
      if (board.serialize() !== before) version.set(version() + 1)
      e.preventDefault()
    }
    host.addEventListener('pointermove', onMove)
    host.addEventListener('pointerup', onUp)
    host.addEventListener('keydown', onKey)

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
      host.removeEventListener('pointermove', onMove)
      host.removeEventListener('pointerup', onUp)
      host.removeEventListener('keydown', onKey)
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
  canvas={canvas}
  readonly={readonly()} style({ display(block).position(relative).size({ w: 100%, h: 100% }) })>
  <svg aria-hidden="true" style({ position({ scheme: absolute, t: 0, l: 0 }).overflow({ all: visible }).pointerEvents(none) })>
    <for {routes() as r} key={r.id}>
      <path d={r.path} data-edge-id={r.id} data-state={incidentSet().has(r.id) ? 'incident' : null} fill="none" stroke="currentColor" stroke-width="2" style({ fg(border), when: { stateIncident: fg(primary) } })></path>
    </for>
    <if {pendingPath()}>
      <path d={pendingPath()} fill="none" stroke="currentColor" stroke-width="2" stroke-dasharray="6 4" style({ fg(primary) })></path>
    </if>
  </svg>

  <for {nodes() as n} key={n.id}>
    <div
      role="group"
      tabindex="0"
      aria-label={n.id}
      aria-selected={selectedSet().has(n.id) ? 'true' : 'false'}
      data-node-id={n.id}
      use:rect={{top: n.rect.y, left: n.rect.x, width: n.rect.w, height: n.rect.h}}
      on:pointerdown={nodePointerDown} style({ pad(2).boxSizing(border).bg(surface-1).fg(foreground).border({ color: border, width: 1 }).radius(md).cursor(grab).select(none).border({ kind: outline, style: none }), when: { focus: border({ kind: outline, color: ring, width: 2, style: solid }), selected: border({ color: primary }).border({ kind: outline, color: primary }) } })>
      <slot name={n.id}></slot>
      <for {n.ports as p} key={p.id}>
        <span
          data-port-id={p.id}
          data-state={p.side}
          use:portEdge={p.side}
          on:pointerdown={portPointerDown} style({ position({ scheme: absolute, t: bind(portTop), b: bind(portBottom), l: bind(portLeft), r: bind(portRight) }).size({ w: 3, h: 3 }).radius(full).bg(background).border({ color: primary, width: 1 }).cursor(crosshair) })></span>
      </for>
    </div>
  </for>
</sail-canvas>
