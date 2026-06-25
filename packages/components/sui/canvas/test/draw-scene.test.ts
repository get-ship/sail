// sui-draw — scene-model behavioral specs (Slice 3).
//
// Pure-runtime: a flat element list (pen strokes / shapes / text) + tool +
// selection + undo/redo command stack on @ship/sui-runtime signals. No DOM,
// no canvas engine — the scene is SVG-native (perfect-freehand outlines +
// roughjs aesthetic) and layers on the createCanvas() substrate.
import { describe, expect, it } from 'vitest'
import { createDrawScene } from '../src/sui-draw/engine/scene'
import type { DrawElement } from '../src/sui-draw/engine/scene'
import { strokePath, outlineToPath, strokeBounds } from '../src/sui-draw/engine/stroke'
import { shapePath, rectFromPoints } from '../src/sui-draw/engine/shape'
import { roughPath } from '../src/sui-draw/engine/rough-adapter'

function pen(id: string, pts: [number, number][]): DrawElement {
  return { id, kind: 'pen', samples: pts.map(([x, y]) => ({ x, y })), color: '#000' }
}

describe('stroke geometry (perfect-freehand)', () => {
  it('strokePath produces a closed filled path from samples', () => {
    const d = strokePath([{ x: 0, y: 0 }, { x: 10, y: 0 }, { x: 20, y: 10 }])
    expect(d.startsWith('M ')).toBe(true)
    expect(d.endsWith('Z')).toBe(true)
  })

  it('strokePath is empty for no samples', () => {
    expect(strokePath([])).toBe('')
  })

  it('outlineToPath handles a degenerate 1-point outline', () => {
    expect(outlineToPath([[5, 5]])).toBe('M 5 5 Z')
  })

  it('strokeBounds is the AABB of the samples', () => {
    expect(strokeBounds([{ x: 10, y: 5 }, { x: 30, y: 25 }])).toEqual({ x: 10, y: 5, w: 20, h: 20 })
  })
})

describe('shape geometry (clean vector)', () => {
  it('rectFromPoints normalizes drag direction', () => {
    expect(rectFromPoints({ x: 30, y: 40 }, { x: 10, y: 10 })).toEqual({ x: 10, y: 10, w: 20, h: 30 })
  })

  it('rect path is a closed box', () => {
    const d = shapePath('rect', { x: 0, y: 0 }, { x: 20, y: 10 })
    expect(d).toBe('M 0 0 h 20 v 10 h -20 Z')
  })

  it('line path is a single segment', () => {
    expect(shapePath('line', { x: 0, y: 0 }, { x: 5, y: 5 })).toBe('M 0 0 L 5 5')
  })

  it('arrow path includes a head (two extra segments)', () => {
    const d = shapePath('arrow', { x: 0, y: 0 }, { x: 100, y: 0 })
    expect(d).toContain('M 0 0 L 100 0')
    // arrowhead barbs add a second M…L…L run
    expect((d.match(/M /g) || []).length).toBe(2)
  })

  it('ellipse path uses elliptical arcs', () => {
    const d = shapePath('ellipse', { x: 0, y: 0 }, { x: 40, y: 20 })
    expect(d).toContain('a ')
    expect(d.endsWith('Z')).toBe(true)
  })
})

describe('rough-adapter (sketchy aesthetic, sole roughjs importer)', () => {
  it('roughens a rectangle into a non-trivial path', () => {
    const d = roughPath('rect', { x: 0, y: 0 }, { x: 50, y: 30 }, 7)
    expect(d.length).toBeGreaterThan(0)
    expect(d.startsWith('M')).toBe(true)
  })

  it('is deterministic for a fixed seed', () => {
    const a = roughPath('ellipse', { x: 0, y: 0 }, { x: 40, y: 40 }, 42)
    const b = roughPath('ellipse', { x: 0, y: 0 }, { x: 40, y: 40 }, 42)
    expect(a).toBe(b)
  })
})

describe('scene-model — element lifecycle', () => {
  it('seeds elements and exposes them as a derived slice', () => {
    const s = createDrawScene({ elements: [pen('a', [[0, 0], [10, 10]])] })
    expect(s.elements().map((e) => e.id)).toEqual(['a'])
  })

  it('addElement appends; removeElements drops + deselects', () => {
    const s = createDrawScene()
    s.addElement(pen('a', [[0, 0], [1, 1]]))
    s.addElement(pen('b', [[0, 0], [1, 1]]))
    s.setSelection(['a', 'b'])
    s.removeElements(['a'])
    expect(s.elements().map((e) => e.id)).toEqual(['b'])
    expect(s.selection()).toEqual(['b'])
  })

  it('rendered exposes a path per element', () => {
    const s = createDrawScene({ elements: [pen('a', [[0, 0], [10, 0], [20, 10]])] })
    const r = s.rendered()
    expect(r).toHaveLength(1)
    expect(r[0].path.length).toBeGreaterThan(0)
    expect(r[0].kind).toBe('pen')
  })

  it('sketchy mode renders shapes via roughjs (different path than clean)', () => {
    const shape: DrawElement = { id: 's', kind: 'rect', a: { x: 0, y: 0 }, b: { x: 40, y: 20 }, color: '#000', seed: 3 }
    const clean = createDrawScene({ elements: [shape] }).rendered()[0].path
    const rough = createDrawScene({ elements: [shape], sketchy: true }).rendered()[0].path
    expect(clean).not.toBe(rough)
  })
})

describe('scene-model — tool slice', () => {
  it('defaults to select and switches', () => {
    const s = createDrawScene()
    expect(s.tool()).toBe('select')
    s.setTool('pen')
    expect(s.tool()).toBe('pen')
  })
})

describe('scene-model — live drafting', () => {
  it('drafts a pen stroke and commits it as an element', () => {
    const s = createDrawScene()
    s.beginStroke({ x: 0, y: 0 }, '#f00')
    s.extendStroke({ x: 10, y: 5 })
    s.extendStroke({ x: 20, y: 0 })
    expect(s.draft()).not.toBeNull()
    const el = s.commitDraft()
    expect(el).not.toBeNull()
    expect(s.draft()).toBeNull()
    expect(s.elements()).toHaveLength(1)
  })

  it('discards a degenerate pen tap (<2 samples)', () => {
    const s = createDrawScene()
    s.beginStroke({ x: 0, y: 0 }, '#000')
    expect(s.commitDraft()).toBeNull()
    expect(s.elements()).toHaveLength(0)
  })

  it('drafts a shape via begin/extend and commits with extent', () => {
    const s = createDrawScene()
    s.beginShape('rect', { x: 0, y: 0 }, '#000')
    s.extendShape({ x: 50, y: 30 })
    const el = s.commitDraft()
    expect(el).not.toBeNull()
    expect(el!.kind).toBe('rect')
    expect(s.elements()).toHaveLength(1)
  })

  it('discards a zero-extent shape', () => {
    const s = createDrawScene()
    s.beginShape('rect', { x: 5, y: 5 }, '#000')
    s.extendShape({ x: 5, y: 5 })
    expect(s.commitDraft()).toBeNull()
    expect(s.elements()).toHaveLength(0)
  })

  it('cancelDraft drops the in-progress draft', () => {
    const s = createDrawScene()
    s.beginStroke({ x: 0, y: 0 }, '#000')
    s.cancelDraft()
    expect(s.draft()).toBeNull()
    expect(s.elements()).toHaveLength(0)
  })
})

describe('scene-model — selection', () => {
  it('setSelection / toggle / clear', () => {
    const s = createDrawScene({ elements: [pen('a', [[0, 0], [1, 1]]), pen('b', [[0, 0], [1, 1]])] })
    s.setSelection(['a'])
    expect(s.selection()).toEqual(['a'])
    s.toggleSelection('b')
    expect(new Set(s.selection())).toEqual(new Set(['a', 'b']))
    s.toggleSelection('a')
    expect(s.selection()).toEqual(['b'])
    s.clearSelection()
    expect(s.selection()).toEqual([])
  })

  it('selectInRect picks elements whose bounds intersect a world box', () => {
    const s = createDrawScene({
      elements: [pen('a', [[0, 0], [20, 20]]), pen('b', [[200, 0], [220, 20]])],
    })
    s.selectInRect({ x: -5, y: -5, w: 60, h: 60 })
    expect(s.selection()).toEqual(['a'])
  })

  it('selectionBounds is the union AABB of selected elements', () => {
    const s = createDrawScene({
      elements: [pen('a', [[0, 0], [10, 10]]), pen('b', [[40, 40], [50, 50]])],
    })
    s.setSelection(['a', 'b'])
    expect(s.selectionBounds()).toEqual({ x: 0, y: 0, w: 50, h: 50 })
  })

  it('moveSelectionBy translates selected elements only', () => {
    const s = createDrawScene({
      elements: [pen('a', [[0, 0], [10, 10]]), pen('b', [[0, 0], [10, 10]])],
    })
    s.setSelection(['a'])
    s.moveSelectionBy(5, 7)
    const a = s.elements().find((e) => e.id === 'a') as Extract<DrawElement, { kind: 'pen' }>
    const b = s.elements().find((e) => e.id === 'b') as Extract<DrawElement, { kind: 'pen' }>
    expect(a.samples[0]).toMatchObject({ x: 5, y: 7 })
    expect(b.samples[0]).toMatchObject({ x: 0, y: 0 })
  })
})

describe('scene-model — undo / redo', () => {
  it('undo reverts the last mutation; redo replays it', () => {
    const s = createDrawScene()
    expect(s.canUndo()).toBe(false)
    s.addElement(pen('a', [[0, 0], [1, 1]]))
    expect(s.elements()).toHaveLength(1)
    expect(s.canUndo()).toBe(true)
    s.undo()
    expect(s.elements()).toHaveLength(0)
    expect(s.canRedo()).toBe(true)
    s.redo()
    expect(s.elements()).toHaveLength(1)
  })

  it('a new mutation clears the redo stack', () => {
    const s = createDrawScene()
    s.addElement(pen('a', [[0, 0], [1, 1]]))
    s.undo()
    s.addElement(pen('b', [[0, 0], [1, 1]]))
    expect(s.canRedo()).toBe(false)
    expect(s.elements().map((e) => e.id)).toEqual(['b'])
  })

  it('moveSelectionBy is undoable', () => {
    const s = createDrawScene({ elements: [pen('a', [[0, 0], [10, 10]])] })
    s.setSelection(['a'])
    s.moveSelectionBy(10, 10)
    s.undo()
    const a = s.elements()[0] as Extract<DrawElement, { kind: 'pen' }>
    expect(a.samples[0]).toMatchObject({ x: 0, y: 0 })
  })
})

describe('scene-model — content bounds + serialization', () => {
  it('contentBounds is the union of all element AABBs', () => {
    const s = createDrawScene({
      elements: [pen('a', [[0, 0], [10, 10]]), pen('b', [[40, 40], [60, 60]])],
    })
    expect(s.contentBounds()).toEqual({ x: 0, y: 0, w: 60, h: 60 })
  })

  it('contentBounds is null with no elements', () => {
    expect(createDrawScene().contentBounds()).toBeNull()
  })

  it('serialize round-trips through JSON', () => {
    const s = createDrawScene({ elements: [pen('a', [[0, 0], [1, 1]])] })
    const parsed = JSON.parse(s.serialize()) as { elements: DrawElement[] }
    expect(parsed.elements).toHaveLength(1)
    expect(parsed.elements[0].id).toBe('a')
  })
})
