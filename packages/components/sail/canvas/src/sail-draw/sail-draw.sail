<!--
  <sail-draw> — SVG whiteboard on <sail-canvas> (pen/shapes/select/undo).
  Scene model in engine/; optional roughjs sketchy. Events: sail-draw-change, sail-draw-select.
-->

<scope>
  props: {
    value = ''
    readonly = false
    theme: 'light' | 'dark' = 'light'
    grid: 'none' | 'dots' | 'lines' = 'none'
    tool: 'select' | 'pan' | 'pen' | 'rect' | 'ellipse' | 'arrow' | 'line' | 'text' = 'select'
    sketchy = false
    label = ''
  }
</scope>
<controller>

import type { Scope } from './sail-draw.sail'
import { createCanvas } from '../engine/create-canvas'
import { createDrawScene, type DrawElement, type DrawTool } from './engine/scene'
import type { ShapeKind } from './engine/shape'

function parseScene(raw: string): { elements?: DrawElement[] } | null {
  if (!raw) return null
  try {
    return JSON.parse(raw) as { elements?: DrawElement[] }
  } catch {
    return null
  }
}

const SHAPE_TOOLS = new Set<DrawTool>(['rect', 'ellipse', 'arrow', 'line'])
const TOOL_KEYS: Record<string, DrawTool> = {
  v: 'select',
  h: 'pan',
  p: 'pen',
  r: 'rect',
  o: 'ellipse',
  a: 'arrow',
  l: 'line',
  t: 'text',
}

export default controller(({ value, sketchy, readonly }: Scope) => {
  const seed = parseScene(value())
  const scene = createDrawScene({
    elements: seed?.elements ?? [],
    sketchy: sketchy(),
  })
  const canvas = createCanvas({
    getViewportElement: () => null, // <sail-canvas> owns the viewport thunk
  })
  const drawing = signal(false)
  const movingSelection = signal(false)
  const lastPointer = signal<{ x: number; y: number } | null>(null)
  const version = signal(0)
  const changeSrc = debounce(version, 200)
  // Derived memos are plain statements in the setup body now — no `derived:`
  // section to fold them into.
  const rendered = derived(() => scene.rendered())
  const selectedSet = derived(() => new Set(scene.selection()))
  const draftPath = derived(() => scene.draft()?.path ?? '')
  const draftColor = derived(() => scene.draft()?.color ?? 'currentColor')
  const selectionRect = derived(() => scene.selectionBounds())
  const statusLabel = derived(() => {
    const n = scene.selection().length
    return n > 0 ? `${n} selected` : `${scene.elements().length} elements`
  })

  // Press on empty surface: begin a draft for the active tool (pen/shape),
  // or clear selection in select mode.
  const surfaceDown = (e: DomEv): void => {
    if (readonly()) return
    if (e.button !== 0) return
    const tool = scene.tool()
    const world = canvas.screenToWorld({ x: e.clientX, y: e.clientY })
    if (tool === 'pen') {
      scene.beginStroke(world, 'currentColor')
      drawing.set(true)
    } else if (SHAPE_TOOLS.has(tool)) {
      scene.beginShape(tool as ShapeKind, world, 'currentColor')
      drawing.set(true)
    } else if (tool === 'select') {
      scene.clearSelection()
    }
  }
  // Press on an element (select mode): select + begin a move drag.
  const elementDown = (e: DomEv): void => {
    if (readonly()) return
    if (scene.tool() !== 'select') return
    const el = e.currentTarget as HostBag
    const id = el.dataset.elId
    if (!id) return
    if (e.shiftKey) scene.toggleSelection(id)
    else if (!scene.selection().includes(id)) scene.setSelection([id])
    movingSelection.set(true)
    lastPointer.set({ x: e.clientX, y: e.clientY })
    e.stopPropagation()
  }

  mount((host) => {
    const onMove = (e: DomEv): void => {
      if (drawing()) {
        const world = canvas.screenToWorld({ x: e.clientX, y: e.clientY })
        if (scene.tool() === 'pen') scene.extendStroke({ x: world.x, y: world.y, pressure: 0.5 })
        else scene.extendShape(world)
        return
      }
      const last = lastPointer()
      if (movingSelection() && last) {
        const scale = canvas.transform().scale
        const dx = (e.clientX - last.x) / scale
        const dy = (e.clientY - last.y) / scale
        scene.moveSelectionBy(dx, dy)
        lastPointer.set({ x: e.clientX, y: e.clientY })
      }
    }
    const onUp = (): void => {
      if (drawing()) {
        const el = scene.commitDraft()
        drawing.set(false)
        if (el) {
          version.set(version() + 1)
          host.emit!('sail-draw-select', { ids: scene.selection() })
        }
      }
      if (movingSelection()) {
        movingSelection.set(false)
        lastPointer.set(null)
        version.set(version() + 1)
      }
    }
    const onKey = (e: DomEv): void => {
      // Never hijack keys while typing into a field.
      const tgt = e.target as HostBag | null
      if (tgt && (tgt.tagName === 'INPUT' || tgt.tagName === 'TEXTAREA' || tgt.isContentEditable)) return
      if (e.key === 'Escape') {
        scene.clearSelection()
        scene.cancelDraft()
        drawing.set(false)
        host.emit!('sail-draw-select', { ids: [] })
        return
      }
      const mod = e.metaKey || e.ctrlKey
      if (!readonly() && mod && e.key.toLowerCase() === 'z') {
        if (e.shiftKey) scene.redo()
        else scene.undo()
        version.set(version() + 1)
        e.preventDefault()
        return
      }
      if (!readonly() && (e.key === 'Delete' || e.key === 'Backspace') && scene.selection().length > 0) {
        scene.removeElements(scene.selection())
        version.set(version() + 1)
        host.emit!('sail-draw-select', { ids: [] })
        e.preventDefault()
        return
      }
      // tool shortcuts (only when not typing into a field)
      const t = TOOL_KEYS[e.key.toLowerCase()]
      if (t && !e.metaKey && !e.ctrlKey) {
        scene.setTool(t)
        hostSetAttr(host, 'tool', t)
      }
    }
    host.addEventListener('pointermove', onMove)
    host.addEventListener('pointerup', onUp)
    host.addEventListener('keydown', onKey)

    // Reflect the host `tool` attribute (set declaratively or via a toolbar)
    // onto the scene tool slice.
    const syncTool = (): void => {
      const t = (hostGetAttr(host, 'tool') ?? 'select') as DrawTool
      if (t !== scene.tool()) scene.setTool(t)
    }
    syncTool()
    const stop_toolObserver = observeHostAttrs(host, ['tool'], syncTool)

    // Debounced scene-change event (the KEEP'd contract: value + version).
    const stopChange = effect(() => {
      const v = changeSrc()
      if (v === 0) return
      host.emit!('sail-draw-change', { value: scene.serialize(), version: v })
    })

    return () => {
      host.removeEventListener('pointermove', onMove)
      host.removeEventListener('pointerup', onUp)
      host.removeEventListener('keydown', onKey)
      stop_toolObserver()
      stopChange()
      canvas.dispose()
    }
  })

  return {
    scene, canvas, drawing, movingSelection, lastPointer, version, changeSrc,
    rendered, selectedSet, draftPath, draftColor, selectionRect, statusLabel,
    surfaceDown, elementDown,
  }
})

</controller>

<!-- `.toolbar` was a flex container; compose a primitive instead — flex
     layout is the floor's boundary, and `<row>` IS the row container. -->
<row role="toolbar" aria-label="Drawing tools" gap="1" align="center" style({ pad({ x: 2, y: 1 }).border({ edge: b, color: border, width: 1 }).bg(surface-1) })>
  <slot name="toolbar"></slot>
</row>

<sail-canvas
  role="application"
  aria-label={label()}
  canvas={canvas}
  readonly={readonly()}
  on:pointerdown={surfaceDown} style({ display(block).position(relative).size({ w: full, h: full }).fg(foreground) })>
  <svg role="group" aria-label={statusLabel()} style({ position({ scheme: absolute, t: 0, l: 0 }).overflow({ all: visible }).pointerEvents(none) })>
    <for {rendered() as el} key={el.id}>
      <if {el.kind === 'text'}>
        <text
          x={el.at.x}
          y={el.at.y}
          fill={selectedSet().has(el.id) ? 'var(--primary)' : el.color}
          stroke="none"
          data-el-id={el.id}
          aria-selected={selectedSet().has(el.id) ? 'true' : 'false'}
          on:pointerdown={elementDown}
          pointer-events="fill" style({ type({ family: body, size: base }) })>{el.text}</text>
      {else => <path
          d={el.path}
          data-el-id={el.id}
          data-state={el.kind === 'pen' ? 'pen' : null}
          aria-selected={selectedSet().has(el.id) ? 'true' : 'false'}
          use:swatch={el.color}
          on:pointerdown={elementDown}
          vector-effect="non-scaling-stroke"
          pointer-events={el.kind === 'pen' ? 'fill' : 'stroke'}
          fill={el.kind === 'pen' ? 'currentColor' : 'none'}
          stroke={el.kind === 'pen' ? 'none' : selectedSet().has(el.id) ? 'var(--primary)' : null}
          stroke-width="2" style({ cursor(pointer) })></path>}</if>
    </for>
    <if {draftPath()}>
      <path
        d={draftPath()}
        use:swatch={draftColor()}
        vector-effect="non-scaling-stroke"
        pointer-events="none"
        fill="none"
        stroke-width="2" style({ filter({ opacity: 85 }) })></path>
    </if>

    <if {selectionRect()}>
      <rect
        x={selectionRect().x}
        y={selectionRect().y}
        width={selectionRect().w}
        height={selectionRect().h}

        fill="none"
        stroke="currentColor"
        stroke-width="1"
        stroke-dasharray="4 3" style({ fg(primary).pointerEvents(none) })></rect>
    </if>
  </svg>
</sail-canvas>

<div role="status" aria-live="polite" style({ position({ scheme: absolute, b: 1, l: 2 }).fg(muted-foreground).pointerEvents(none) })>  <span style({ display(contents).type({ size: xs }) })>    {statusLabel()}
  </span>
</div>
