<!--
  <sail-draw> — an Excalidraw-class whiteboard, OWNED on the canvas substrate.

  Builds createCanvas() + an owned scene model in the controller. Strokes /
  shapes / arrows / text live as DOM/SVG paths (selectable, accessible,
  token-themeable) — NO canvas engine. Pen outlines come from vendored
  perfect-freehand; the optional hand-drawn aesthetic from vendored roughjs
  (the rough-adapter is the sole importer). The whiteboard composes
  <sail-canvas> as its substrate via canvas identity and renders its scene
  into the world layer; selection/transform are an SVG overlay.

  This REPLACES the deleted React + Excalidraw bridge. v1 ships owned
  SVG-native strokes + shapes + selection + move + undo/redo. Resize/rotate
  transform handles are not implemented — the scene is intentionally
  SVG-native (no canvas scene-graph engine), so when they land they will be
  SVG-overlay handles driven by the scene model, not a vendored canvas lib.

  Props
  ─────────────────────────────────────────────────────────────────────
  value     (string, '')                       SSR scene-JSON FALLBACK
                                                  (JSON.parse). * wins.
  readonly  (bool, false reflect)               Pan/zoom only; no edits.
  theme     ('light'|'dark' reflect)            Forwarded to the substrate.
  grid      ('none'|'dots'|'lines')             Forwarded background grid hint.
  tool      ('select'|'pan'|'pen'|'rect'|       Active tool (reflect).
             'ellipse'|'arrow'|'line'|'text')
  sketchy   (bool, false reflect)               Hand-drawn aesthetic (roughjs).
  label     (string, '')                        → aria-label on the region.
  ─────────────────────────────────────────────────────────────────────
  Events (host.emit!):
    sail-draw-change { value, version } — debounced 200ms scene mutation.
    sail-draw-select { ids }            — selection changed.
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
import { controller, signal, derived, effect, debounce } from '@ship/sail-runtime'
import type { Scope } from './sail-draw.sail'
import { createCanvas } from '../engine/create-canvas'
import { createDrawScene, type DrawElement, type DrawTool } from './engine/scene'
import type { ShapeKind } from './engine/shape'

// SSR fallback = the verified sail-combobox shape: plain JSON.parse, try/caught.
// The live path is * identity onto the scene model.
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

// <sail-draw> controller — an owned SVG-native whiteboard on the canvas
// substrate. The scene model holds elements (world coords); the view maps to
// screen through the substrate transform. Pen drafting + shape drag + select
// + undo/redo run through returned handlers / mount (Cardinal 3).
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
  const surfaceDown = (e: PointerEvent): void => {
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
  const elementDown = (e: PointerEvent): void => {
    if (readonly()) return
    if (scene.tool() !== 'select') return
    const el = e.currentTarget as SVGElement
    const id = el.dataset.elId
    if (!id) return
    if (e.shiftKey) scene.toggleSelection(id)
    else if (!scene.selection().includes(id)) scene.setSelection([id])
    movingSelection.set(true)
    lastPointer.set({ x: e.clientX, y: e.clientY })
    e.stopPropagation()
  }

  mount((host) => {
    const onMove = (e: PointerEvent): void => {
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
    const onKey = (e: KeyboardEvent): void => {
      // Never hijack keys while typing into a field.
      const tgt = e.target as HTMLElement | null
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
        host.setAttribute('tool', t)
      }
    }
    host.addEventListener('pointermove', onMove as EventListener)
    host.addEventListener('pointerup', onUp as EventListener)
    host.addEventListener('keydown', onKey as EventListener)

    // Reflect the host `tool` attribute (set declaratively or via a toolbar)
    // onto the scene tool slice.
    const syncTool = (): void => {
      const t = (host.getAttribute('tool') ?? 'select') as DrawTool
      if (t !== scene.tool()) scene.setTool(t)
    }
    syncTool()
    const toolObserver = new MutationObserver(syncTool)
    toolObserver.observe(host, { attributes: true, attributeFilter: ['tool'] })

    // Debounced scene-change event (the KEEP'd contract: value + version).
    const stopChange = effect(() => {
      const v = changeSrc()
      if (v === 0) return
      host.emit!('sail-draw-change', { value: scene.serialize(), version: v })
    })

    return () => {
      host.removeEventListener('pointermove', onMove as EventListener)
      host.removeEventListener('pointerup', onUp as EventListener)
      host.removeEventListener('keydown', onKey as EventListener)
      toolObserver.disconnect()
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
<row role="toolbar" aria-label="Drawing tools" gap="1" align="center" sx="py-1 px-2 border-b border-b-border bg-card">
  <slot name="toolbar"></slot>
</row>

<sail-canvas
  role="application"
  aria-label={label()}
  canvas={canvas}
  readonly={readonly()}
  on:pointerdown={surfaceDown}
  sx="block relative w-full h-full text-foreground"
>
  <svg role="group" aria-label={statusLabel()} sx="absolute top-0 left-0 overflow-visible pointer-events-none">
    {#for el of rendered() key={el.id}}
      {#if el.kind === 'text'}
        <text
          x={el.at.x}
          y={el.at.y}
          fill={el.color}
          data-el-id={el.id}
          aria-selected={selectedSet().has(el.id) ? 'true' : 'false'}
          on:pointerdown={elementDown}
          pointer-events="fill"
          sx="font-body text-base stroke-none selected:fill-primary"
        >{el.text}</text>
      {:else}
        <path
          d={el.path}
          data-el-id={el.id}
          data-state={el.kind === 'pen' ? 'pen' : null}
          aria-selected={selectedSet().has(el.id) ? 'true' : 'false'}
          use:swatch={el.color}
          on:pointerdown={elementDown}
          vector-effect="non-scaling-stroke"
          pointer-events={el.kind === 'pen' ? 'fill' : 'stroke'}
          stroke-width="2"
          sx="fill-none cursor-pointer selected:stroke-primary state-pen:fill-currentColor state-pen:stroke-none"
        ></path>
      {/if}
    {/for}
    {#if draftPath()}
      <path
        d={draftPath()}
        use:swatch={draftColor()}
        vector-effect="non-scaling-stroke"
        pointer-events="none"
        stroke-width="2"
        sx="fill-none opacity-85"
      ></path>
    {/if}

    {#if selectionRect()}
      <rect
        x={selectionRect().x}
        y={selectionRect().y}
        width={selectionRect().w}
        height={selectionRect().h}
        sx="fill-none stroke-primary pointer-events-none"
        stroke-width="1"
        stroke-dasharray="4 3"
      ></rect>
    {/if}
  </svg>
</sail-canvas>

<div role="status" aria-live="polite" sx="absolute bottom-1 left-2 text-xs text-muted-foreground pointer-events-none">{statusLabel()}</div>
