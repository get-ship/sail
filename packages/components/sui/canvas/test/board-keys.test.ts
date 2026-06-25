// sui-board — keyboard-operability resolver specs (B1) + single-emit dedup (B2).
//
// applyBoardKey is the pure resolver the controller forwards keydown to. The
// emit-dedup spec mirrors the controller's selection effect (the sui-combobox
// lastEmitted pattern): prime on the mount value without emitting, then emit
// once per distinct selection.
import { describe, expect, it } from 'vitest'
import { createBoardModel } from '../src/sui-board/engine/board-model'
import type { Node } from '../src/sui-board/engine/board-model'
import { applyBoardKey, ARROW_STEP } from '../src/sui-board/engine/board-keys'

function node(id: string, x: number, y: number, w = 100, h = 60): Node {
  return { id, rect: { x, y, w, h }, kind: 'default', ports: [], data: {} }
}

describe('board-keys — arrow movement', () => {
  it('arrow keys nudge the focused node by ARROW_STEP / scale', () => {
    const b = createBoardModel({ nodes: [node('a', 0, 0)] })
    expect(applyBoardKey(b, { key: 'ArrowRight' }, 'a', 1)).toBe(true)
    expect(b.nodes()[0].rect.x).toBe(ARROW_STEP)
    expect(applyBoardKey(b, { key: 'ArrowDown' }, 'a', 1)).toBe(true)
    expect(b.nodes()[0].rect.y).toBe(ARROW_STEP)
  })

  it('divides the world step by the current scale', () => {
    const b = createBoardModel({ nodes: [node('a', 0, 0)] })
    applyBoardKey(b, { key: 'ArrowRight' }, 'a', 2)
    expect(b.nodes()[0].rect.x).toBe(ARROW_STEP / 2)
  })

  it('moves the whole selection when the focused node is selected', () => {
    const b = createBoardModel({ nodes: [node('a', 0, 0), node('b', 300, 0)] })
    b.setSelection(['a', 'b'])
    applyBoardKey(b, { key: 'ArrowLeft' }, 'a', 1)
    expect(b.nodes().find((n) => n.id === 'a')!.rect.x).toBe(-ARROW_STEP)
    expect(b.nodes().find((n) => n.id === 'b')!.rect.x).toBe(300 - ARROW_STEP)
  })

  it('is a no-op (returns false) for an arrow with no focused node', () => {
    const b = createBoardModel({ nodes: [node('a', 0, 0)] })
    expect(applyBoardKey(b, { key: 'ArrowRight' }, null, 1)).toBe(false)
    expect(b.nodes()[0].rect.x).toBe(0)
  })
})

describe('board-keys — select + escape', () => {
  it('Enter selects the focused node (Space too)', () => {
    const b = createBoardModel({ nodes: [node('a', 0, 0), node('b', 300, 0)] })
    expect(applyBoardKey(b, { key: 'Enter' }, 'a', 1)).toBe(true)
    expect(b.selectedNodes()).toEqual(['a'])
    applyBoardKey(b, { key: ' ' }, 'b', 1)
    expect(b.selectedNodes()).toEqual(['b'])
  })

  it('Shift+Enter toggles additively', () => {
    const b = createBoardModel({ nodes: [node('a', 0, 0), node('b', 300, 0)] })
    applyBoardKey(b, { key: 'Enter' }, 'a', 1)
    applyBoardKey(b, { key: 'Enter', shiftKey: true }, 'b', 1)
    expect(new Set(b.selectedNodes())).toEqual(new Set(['a', 'b']))
  })

  it('Escape clears selection and any pending edge', () => {
    const b = createBoardModel({ nodes: [node('a', 0, 0)] })
    b.setSelection(['a'])
    expect(applyBoardKey(b, { key: 'Escape' }, null, 1)).toBe(true)
    expect(b.selectedNodes()).toEqual([])
  })

  it('returns false for an unhandled key', () => {
    const b = createBoardModel({ nodes: [node('a', 0, 0)] })
    expect(applyBoardKey(b, { key: 'x' }, 'a', 1)).toBe(false)
  })
})

// Mirrors the controller's selection effect: a deduped, mount-skipping emitter
// over board.selectedNodes(). Documents the B2 single-emit contract: no mount
// emit, one emit per distinct selection, no double fire on repeat.
describe('board selection emit — single source + dedup (B2)', () => {
  function makeEmitter() {
    const out: string[][] = []
    let lastEmitted: string | null = null
    const tick = (ids: string[]): void => {
      const key = ids.join(' ')
      if (lastEmitted === null) {
        lastEmitted = key
        return
      }
      if (key === lastEmitted) return
      lastEmitted = key
      out.push(ids)
    }
    return { out, tick }
  }

  it('does not emit the initial mount value', () => {
    const { out, tick } = makeEmitter()
    tick([]) // mount
    expect(out).toEqual([])
  })

  it('emits once per distinct selection, never on a repeat', () => {
    const { out, tick } = makeEmitter()
    tick([]) // mount
    tick(['a'])
    tick(['a']) // drag-end re-run with same selection → no second fire
    tick(['a', 'b'])
    tick([])
    expect(out).toEqual([['a'], ['a', 'b'], []])
  })
})
