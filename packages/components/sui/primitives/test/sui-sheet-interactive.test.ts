// TESTS-FIRST — sui-sheet INTERACTIVITY slice: the live surface.
//
// Status: RED until the controller wires the on:* handlers + the view grows the
// editor input / fill-handle affordance. These component-level specs drive the
// real custom element via DOM events (click / drag / keydown / dblclick / copy /
// paste) and assert the emitted events (sui-selection-change, sui-cell-activate,
// sui-cell-commit) + the resulting cell state.
//
// Authority: job spec (sui-sheet interactivity lane) — selection, edit/commit,
// keyboard nav, fill-handle series, copy/paste round-trip.

import { afterEach, describe, expect, it } from 'vitest'
import '../src/sui-sheet/sui-sheet.compiled.js'
import { createSheet } from '../src/sui-sheet/engine/create-sheet'
import type { WorkbookDef } from '../src/sui-sheet/engine/contract'
import { createContainerRegistry, flush, mount } from '../src/test-utils'

const reg = createContainerRegistry()
afterEach(() => reg.cleanup())

function book(cells: Record<string, { value?: unknown; formula?: string }> = {}): WorkbookDef {
  return {
    sheets: [{ id: 's1', name: 'Sheet1', rowCount: 100, colCount: 26, cells: cells as never }],
    activeSheetId: 's1',
  }
}

function mountSheet(cells: Record<string, { value?: unknown; formula?: string }> = {}) {
  const { el, shadow, container } = mount(reg, 'sui-sheet', {})
  const sheet = createSheet({ workbook: book(cells) })
  ;(el as unknown as { sheet: unknown }).sheet = sheet
  return { el, shadow, container, sheet }
}

const cellAt = (shadow: ShadowRoot | HTMLElement, a1: string): HTMLElement =>
  shadow.querySelector(`[data-a1="${a1}"]`) as HTMLElement

function pointer(el: Element, type: string, extra: MouseEventInit = {}): void {
  el.dispatchEvent(new MouseEvent(type, { bubbles: true, composed: true, ...extra }))
}
function key(el: Element, k: string, extra: KeyboardEventInit = {}): void {
  el.dispatchEvent(new KeyboardEvent('keydown', { key: k, bubbles: true, composed: true, cancelable: true, ...extra }))
}

describe('<sui-sheet> — pointer selection', () => {
  it('clicking a cell sets it active and emits sui-selection-change', async () => {
    const { el, shadow, sheet } = mountSheet({ A1: { value: 1 }, B2: { value: 2 } })
    await flush()
    let evt: CustomEvent | null = null
    el.addEventListener('sui-selection-change', (e) => { evt = e as CustomEvent })
    pointer(cellAt(shadow!, 'B2'), 'pointerdown')
    await flush()
    // The engine is the source of truth for the active cell + selection.
    expect(sheet.activeCell()).toEqual({ row: 1, col: 1 })
    expect(sheet.getCell('B2')().isActive).toBe(true)
    // The advertised event fires with the active address + its A1.
    expect(evt).not.toBeNull()
    const detail = evt!.detail as { active: { row: number; col: number }; a1: string }
    expect(detail.active).toEqual({ row: 1, col: 1 })
    expect(detail.a1).toBe('B2')
    expect(evt!.bubbles).toBe(true)
    expect(evt!.composed).toBe(true)
  })

  it('shift-click extends the selection range from the active anchor', async () => {
    const { shadow, sheet } = mountSheet()
    await flush()
    pointer(cellAt(shadow!, 'A1'), 'pointerdown')
    pointer(cellAt(shadow!, 'C3'), 'pointerdown', { shiftKey: true })
    await flush()
    expect(sheet.selection().ranges[0]!.toA1()).toBe('A1:C3')
  })

  it('pointer drag (down → move → up) extends a range', async () => {
    const { shadow, sheet } = mountSheet()
    await flush()
    pointer(cellAt(shadow!, 'B2'), 'pointerdown')
    pointer(cellAt(shadow!, 'D4'), 'pointermove')
    pointer(cellAt(shadow!, 'D4'), 'pointerup')
    await flush()
    expect(sheet.selection().ranges[0]!.toA1()).toBe('B2:D4')
  })
})

describe('<sui-sheet> — editing model', () => {
  it('double-click opens the editor on the active cell and emits sui-cell-activate', async () => {
    const { el, shadow, sheet } = mountSheet({ A1: { value: 9 } })
    await flush()
    let activateEvt: CustomEvent | null = null
    el.addEventListener('sui-cell-activate', (e) => { activateEvt = e as CustomEvent })
    pointer(cellAt(shadow!, 'A1'), 'dblclick')
    await flush()
    expect(sheet.editing()).not.toBeNull()
    expect(sheet.editing()!.cell).toEqual({ row: 0, col: 0 })
    expect(activateEvt).not.toBeNull()
    expect((activateEvt!.detail as { a1: string }).a1).toBe('A1')
    expect(activateEvt!.bubbles).toBe(true)
    expect(activateEvt!.composed).toBe(true)
    expect(shadow!.querySelector('[part="cell-editor"]')).not.toBeNull()
  })

  it('typing a printable char on an active cell starts editing seeded with that char', async () => {
    const { shadow, sheet } = mountSheet()
    await flush()
    pointer(cellAt(shadow!, 'A1'), 'pointerdown')
    key(shadow!.querySelector('[part="grid"]')!, '4')
    await flush()
    expect(sheet.editing()).not.toBeNull()
    expect(sheet.editing()!.draft).toBe('4')
  })

  it('Enter commits the draft, emits sui-cell-commit, and advances down', async () => {
    const { el, shadow, sheet } = mountSheet()
    await flush()
    let commitEvt: CustomEvent | null = null
    let committed: { a1: string; after: { value?: unknown } } | null = null
    el.addEventListener('sui-cell-commit', (e) => {
      commitEvt = e as CustomEvent
      committed = (e as CustomEvent).detail
    })
    pointer(cellAt(shadow!, 'A1'), 'pointerdown')
    key(shadow!.querySelector('[part="grid"]')!, '4')
    await flush()
    const editor = shadow!.querySelector('[part="cell-editor"]') as HTMLInputElement
    editor.value = '42'
    editor.dispatchEvent(new Event('input', { bubbles: true, composed: true }))
    key(editor, 'Enter')
    await flush()
    expect(sheet.getCell('A1')().value).toBe(42)
    expect(committed).not.toBeNull()
    expect(committed!.a1).toBe('A1')
    expect(committed!.after.value).toBe(42)
    expect(commitEvt!.bubbles).toBe(true)
    expect(commitEvt!.composed).toBe(true)
    // Enter advances the active cell down.
    expect(sheet.activeCell()).toEqual({ row: 1, col: 0 })
  })

  it('Escape cancels the edit without writing', async () => {
    const { shadow, sheet } = mountSheet({ A1: { value: 'orig' } })
    await flush()
    pointer(cellAt(shadow!, 'A1'), 'dblclick')
    await flush()
    const editor = shadow!.querySelector('[part="cell-editor"]') as HTMLInputElement
    editor.value = 'changed'
    editor.dispatchEvent(new Event('input', { bubbles: true, composed: true }))
    key(editor, 'Escape')
    await flush()
    expect(sheet.editing()).toBeNull()
    expect(sheet.getCell('A1')().value).toBe('orig')
  })
})

describe('<sui-sheet> — keyboard navigation', () => {
  it('arrow keys move the active cell', async () => {
    const { shadow, sheet } = mountSheet()
    await flush()
    pointer(cellAt(shadow!, 'A1'), 'pointerdown')
    const grid = shadow!.querySelector('[part="grid"]')!
    key(grid, 'ArrowDown')
    key(grid, 'ArrowRight')
    await flush()
    expect(sheet.activeCell()).toEqual({ row: 1, col: 1 })
  })

  it('shift+arrow extends the selection', async () => {
    const { shadow, sheet } = mountSheet()
    await flush()
    pointer(cellAt(shadow!, 'B2'), 'pointerdown')
    const grid = shadow!.querySelector('[part="grid"]')!
    key(grid, 'ArrowDown', { shiftKey: true })
    key(grid, 'ArrowRight', { shiftKey: true })
    await flush()
    expect(sheet.selection().ranges[0]!.toA1()).toBe('B2:C3')
  })

  it('Tab moves the active cell right', async () => {
    const { shadow, sheet } = mountSheet()
    await flush()
    pointer(cellAt(shadow!, 'A1'), 'pointerdown')
    key(shadow!.querySelector('[part="grid"]')!, 'Tab')
    await flush()
    expect(sheet.activeCell()).toEqual({ row: 0, col: 1 })
  })
})

describe('<sui-sheet> — fill handle', () => {
  it('renders a fill-handle affordance on the active selection', async () => {
    const { shadow } = mountSheet({ A1: { value: 1 } })
    await flush()
    expect(shadow!.querySelector('[part="fill-handle"]')).not.toBeNull()
  })

  it('dragging the fill handle series-fills a numeric run', async () => {
    const { shadow, sheet } = mountSheet({ A1: { value: 1 }, A2: { value: 2 } })
    await flush()
    // Select the A1:A2 source run.
    pointer(cellAt(shadow!, 'A1'), 'pointerdown')
    pointer(cellAt(shadow!, 'A2'), 'pointermove')
    pointer(cellAt(shadow!, 'A2'), 'pointerup')
    await flush()
    // Drag the fill handle down to A4.
    const handle = shadow!.querySelector('[part="fill-handle"]')!
    pointer(handle, 'pointerdown')
    pointer(cellAt(shadow!, 'A4'), 'pointermove')
    pointer(cellAt(shadow!, 'A4'), 'pointerup')
    await flush()
    expect(sheet.getCell('A3')().value).toBe(3)
    expect(sheet.getCell('A4')().value).toBe(4)
  })
})

/** A minimal DataTransfer stub (jsdom ships neither DataTransfer nor a
 *  ClipboardEvent that carries clipboardData). Backs a Map of MIME → string. */
function clipboardStub(): DataTransfer {
  const store = new Map<string, string>()
  return {
    getData: (t: string) => store.get(t) ?? '',
    setData: (t: string, v: string) => void store.set(t, v),
  } as unknown as DataTransfer
}
/** Dispatch a copy/paste Event carrying the stub clipboardData (jsdom's
 *  ClipboardEvent constructor ignores the option). */
function clipboardEvent(type: 'copy' | 'paste', clip: DataTransfer): Event {
  const e = new Event(type, { bubbles: true, composed: true, cancelable: true })
  Object.defineProperty(e, 'clipboardData', { value: clip, configurable: true })
  return e
}

describe('<sui-sheet> — clipboard', () => {
  it('copy → paste round-trips the selected range to a new anchor', async () => {
    const { shadow, sheet } = mountSheet({ A1: { value: 5 }, B1: { value: 6 } })
    await flush()
    // Select A1:B1.
    pointer(cellAt(shadow!, 'A1'), 'pointerdown')
    pointer(cellAt(shadow!, 'B1'), 'pointermove')
    pointer(cellAt(shadow!, 'B1'), 'pointerup')
    await flush()
    const grid = shadow!.querySelector('[part="grid"]') as HTMLElement
    const clip = clipboardStub()
    grid.dispatchEvent(clipboardEvent('copy', clip))
    expect(clip.getData('text/plain')).toBe('5\t6')
    // Move the active cell to A3, then paste.
    pointer(cellAt(shadow!, 'A3'), 'pointerdown')
    await flush()
    grid.dispatchEvent(clipboardEvent('paste', clip))
    await flush()
    expect(sheet.getCell('A3')().value).toBe(5)
    expect(sheet.getCell('B3')().value).toBe(6)
  })
})
