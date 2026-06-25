import { controller, signal, derived, effect, debounce } from '@ship/sui-runtime'
import type { Scope } from './sui-draw.sui'
import { createCanvas } from '../engine/create-canvas'
import { createDrawScene, type DrawElement, type DrawTool } from './engine/scene'
import type { ShapeKind } from './engine/shape'

// SSR fallback = the verified sui-combobox shape: plain JSON.parse, try/caught.
// The live path is prop:* identity onto the scene model.
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

// <sui-draw> controller — an owned SVG-native whiteboard on the canvas
// substrate. The scene model holds elements (world coords); the view maps to
// screen through the substrate transform. Pen drafting + shape drag + select
// + undo/redo run through on:* handlers / mount (Cardinal 3).
export default controller<Scope>({
  scope: ({ value, sketchy, readonly }) => {
    const seed = parseScene(value())
    const scene = createDrawScene({
      elements: seed?.elements ?? [],
      sketchy: sketchy(),
    })
    const canvas = createCanvas({
      getViewportElement: () => null, // <sui-canvas> owns the viewport thunk
    })
    const drawing = signal(false)
    const movingSelection = signal(false)
    const lastPointer = signal<{ x: number; y: number } | null>(null)
    const version = signal(0)
    const changeSrc = debounce(version, 200)
    return { scene, canvas, drawing, movingSelection, lastPointer, version, changeSrc, readonly }
  },

  derived: {
    rendered: ({ scene }) => scene.rendered(),
    selectedSet: ({ scene }) => new Set(scene.selection()),
    draftPath: ({ scene }) => scene.draft()?.path ?? '',
    draftColor: ({ scene }) => scene.draft()?.color ?? 'currentColor',
    selectionRect: ({ scene }) => scene.selectionBounds(),
    statusLabel: ({ scene }) => {
      const n = scene.selection().length
      return n > 0 ? `${n} selected` : `${scene.elements().length} elements`
    },
  },

  on: {
    // Press on empty surface: begin a draft for the active tool (pen/shape),
    // or clear selection in select mode.
    surfaceDown: (s, e: PointerEvent) => {
      if (s.readonly) return
      if (e.button !== 0) return
      const tool = s.scene.tool()
      const world = s.canvas.screenToWorld({ x: e.clientX, y: e.clientY })
      if (tool === 'pen') {
        s.scene.beginStroke(world, 'currentColor')
        s.drawing = true
      } else if (SHAPE_TOOLS.has(tool)) {
        s.scene.beginShape(tool as ShapeKind, world, 'currentColor')
        s.drawing = true
      } else if (tool === 'select') {
        s.scene.clearSelection()
      }
    },
    // Press on an element (select mode): select + begin a move drag.
    elementDown: (s, e: PointerEvent) => {
      if (s.readonly) return
      if (s.scene.tool() !== 'select') return
      const el = e.currentTarget as SVGElement
      const id = el.dataset.elId
      if (!id) return
      if (e.shiftKey) s.scene.toggleSelection(id)
      else if (!s.scene.selection().includes(id)) s.scene.setSelection([id])
      s.movingSelection = true
      s.lastPointer = { x: e.clientX, y: e.clientY }
      e.stopPropagation()
    },
  },

  mount(state, host) {
    const scene = state.scene
    const canvas = state.canvas

    const onMove = (e: PointerEvent): void => {
      if (state.drawing) {
        const world = canvas.screenToWorld({ x: e.clientX, y: e.clientY })
        if (scene.tool() === 'pen') scene.extendStroke({ x: world.x, y: world.y, pressure: 0.5 })
        else scene.extendShape(world)
        return
      }
      if (state.movingSelection && state.lastPointer) {
        const scale = canvas.transform().scale
        const dx = (e.clientX - state.lastPointer.x) / scale
        const dy = (e.clientY - state.lastPointer.y) / scale
        scene.moveSelectionBy(dx, dy)
        state.lastPointer = { x: e.clientX, y: e.clientY }
      }
    }
    const onUp = (): void => {
      if (state.drawing) {
        const el = scene.commitDraft()
        state.drawing = false
        if (el) {
          state.version = state.version + 1
          host.emit!('sui-draw-select', { ids: scene.selection() })
        }
      }
      if (state.movingSelection) {
        state.movingSelection = false
        state.lastPointer = null
        state.version = state.version + 1
      }
    }
    const onKey = (e: KeyboardEvent): void => {
      // Never hijack keys while typing into a field.
      const tgt = e.target as HTMLElement | null
      if (tgt && (tgt.tagName === 'INPUT' || tgt.tagName === 'TEXTAREA' || tgt.isContentEditable)) return
      if (e.key === 'Escape') {
        scene.clearSelection()
        scene.cancelDraft()
        state.drawing = false
        host.emit!('sui-draw-select', { ids: [] })
        return
      }
      const mod = e.metaKey || e.ctrlKey
      if (!state.readonly && mod && e.key.toLowerCase() === 'z') {
        if (e.shiftKey) scene.redo()
        else scene.undo()
        state.version = state.version + 1
        e.preventDefault()
        return
      }
      if (!state.readonly && (e.key === 'Delete' || e.key === 'Backspace') && scene.selection().length > 0) {
        scene.removeElements(scene.selection())
        state.version = state.version + 1
        host.emit!('sui-draw-select', { ids: [] })
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
      const v = state.changeSrc
      if (v === 0) return
      host.emit!('sui-draw-change', { value: scene.serialize(), version: v })
    })

    return () => {
      host.removeEventListener('pointermove', onMove as EventListener)
      host.removeEventListener('pointerup', onUp as EventListener)
      host.removeEventListener('keydown', onKey as EventListener)
      toolObserver.disconnect()
      stopChange()
      canvas.dispose()
    }
  },
})
