import { describe, expect, it, vi } from 'vitest'
import { signal, effect } from '../src/signal'
import { applyPatch, applyPatches } from '../src/patch'


describe('applyPatch — set', () => {
  it('replaces root value', () => {
    const $s = signal({ a: 1 })
    applyPatch($s, { op: 'set', value: { b: 2 } })
    expect($s()).toEqual({ b: 2 })
  })
  it('replaces nested field', () => {
    const $s = signal({ a: { b: 1 } })
    applyPatch($s, { op: 'set', path: 'a.b', value: 99 })
    expect($s()).toEqual({ a: { b: 99 } })
  })
  it('replaces array element', () => {
    const $s = signal({ items: [10, 20, 30] })
    applyPatch($s, { op: 'set', path: 'items[1]', value: 99 })
    expect($s()).toEqual({ items: [10, 99, 30] })
  })
  it('preserves reference of unchanged sibling subtrees', () => {
    const sib = { unchanged: true }
    const $s = signal({ a: 1, sib })
    applyPatch($s, { op: 'set', path: 'a', value: 2 })
    expect($s().sib).toBe(sib) // same reference
  })
})

describe('applyPatch — merge', () => {
  it('shallow merges into object', () => {
    const $s = signal({ a: 1, b: 2 })
    applyPatch($s, { op: 'merge', value: { b: 22, c: 3 } })
    expect($s()).toEqual({ a: 1, b: 22, c: 3 })
  })
  it('merges at nested path', () => {
    const $s = signal({ meta: { dirty: false, name: 'x' } })
    applyPatch($s, { op: 'merge', path: 'meta', value: { dirty: true } })
    expect($s()).toEqual({ meta: { dirty: true, name: 'x' } })
  })
})

describe('applyPatch — array ops', () => {
  it('append', () => {
    const $s = signal({ list: [1, 2] })
    applyPatch($s, { op: 'append', path: 'list', value: 3 })
    expect($s()).toEqual({ list: [1, 2, 3] })
  })
  it('prepend', () => {
    const $s = signal({ list: [2, 3] })
    applyPatch($s, { op: 'prepend', path: 'list', value: 1 })
    expect($s()).toEqual({ list: [1, 2, 3] })
  })
  it('insert at index', () => {
    const $s = signal({ list: [1, 4] })
    applyPatch($s, { op: 'insert', path: 'list', value: 2, index: 1 })
    expect($s()).toEqual({ list: [1, 2, 4] })
  })
  it('remove array element', () => {
    const $s = signal({ list: [1, 2, 3] })
    applyPatch($s, { op: 'remove', path: 'list[1]' })
    expect($s()).toEqual({ list: [1, 3] })
  })
  it('remove object field', () => {
    const $s = signal({ a: 1, b: 2 })
    applyPatch($s, { op: 'remove', path: 'b' })
    expect($s()).toEqual({ a: 1 })
  })
  it('move within array', () => {
    const $s = signal({ list: ['a', 'b', 'c', 'd'] })
    applyPatch($s, { op: 'move', path: 'list', from: 0, to: 2 })
    expect($s()).toEqual({ list: ['b', 'c', 'a', 'd'] })
  })
  it('splice replaces', () => {
    const $s = signal({ list: [1, 2, 3, 4, 5] })
    applyPatch($s, { op: 'splice', path: 'list', index: 1, count: 2, value: [99, 100] })
    expect($s()).toEqual({ list: [1, 99, 100, 4, 5] })
  })
})

describe('applyPatch — inc', () => {
  it('numeric increment with default by=1', () => {
    const $s = signal({ counter: 5 })
    applyPatch($s, { op: 'inc', path: 'counter' })
    expect($s()).toEqual({ counter: 6 })
  })
  it('explicit by', () => {
    const $s = signal({ counter: 5 })
    applyPatch($s, { op: 'inc', path: 'counter', by: 10 })
    expect($s()).toEqual({ counter: 15 })
  })
  it('negative by decrements', () => {
    const $s = signal({ x: 100 })
    applyPatch($s, { op: 'inc', path: 'x', by: -5 })
    expect($s()).toEqual({ x: 95 })
  })
})

describe('applyPatch — reactivity', () => {
  it('subscribers fire on signal change from patch', () => {
    const $s = signal({ x: 0 })
    const fn = vi.fn()
    effect(() => {
      $s()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    applyPatch($s, { op: 'set', path: 'x', value: 1 })
    expect(fn).toHaveBeenCalledTimes(2)
  })
})

describe('applyPatches', () => {
  it('multiple patches notify once via batch', () => {
    const $s = signal({ a: 0, b: 0, c: 0 })
    const fn = vi.fn()
    effect(() => {
      $s()
      fn()
    })
    expect(fn).toHaveBeenCalledTimes(1)
    applyPatches($s, [
      { op: 'set', path: 'a', value: 1 },
      { op: 'set', path: 'b', value: 2 },
      { op: 'set', path: 'c', value: 3 },
    ])
    expect(fn).toHaveBeenCalledTimes(2) // +1 for the batched updates
    expect($s()).toEqual({ a: 1, b: 2, c: 3 })
  })
})

describe('applyPatch — errors', () => {
  it('throws on unknown op', () => {
    const $s = signal({})
    expect(() => applyPatch($s, { op: 'bogus' as 'set' })).toThrow(/unknown patch op/)
  })
  it('throws on append to non-array', () => {
    const $s = signal({ x: 'not an array' })
    expect(() => applyPatch($s, { op: 'append', path: 'x', value: 1 })).toThrow(
      /expected array/,
    )
  })
  it('throws on merge into non-object', () => {
    const $s = signal({ x: 5 })
    expect(() =>
      applyPatch($s, { op: 'merge', path: 'x', value: { a: 1 } }),
    ).toThrow(/merge requires/)
  })
})
