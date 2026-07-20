// @vitest-environment jsdom

// Regression coverage for the SSR property-channel hydrate path.
//
// Plain `name={expr}` on a CE/IDL property lowers to bindProp. The
// compiler emits TWO halves of the same contract (see
// crates/sail/compiler/src/emit/template_emit/dom_emit.rs):
//
//   1. SSR HTML carries a marker attribute `prop:NAME="bN"` (wire name;
//      not an author directive) on the element.
//   2. The compiled route's `hydrate(root, ctx)` runs
//        const __m = collectElementMarkers(root);
//        if (__m[bN]) __cleanups.push(hydrateProp(__m[bN], "name", …));
//
// `collectElementMarkers` is the bridge between the two: it maps each
// SSR marker's binding key (`bN`) → the element carrying it. If it does
// not recognise `prop:*` wire markers, `__m[bN]` is `undefined`, the
// `hydrateProp` dispatch is skipped, and the value is silently dropped
// on hydration.

import { describe, expect, it } from 'vitest'
import { signal } from '@ship/sail-runtime'
import { collectElementMarkers } from '../../src/hydrate/markers'
import { hydrateProp } from '../../src/hydrate/prop'

describe('hydrate/markers — collectElementMarkers', () => {
  it('collects a prop:* wire marker so the prop hydrator can find its element', () => {
    // Mirrors the SSR HTML the compiler emits for plain
    //   <my-grid items={items()}></my-grid>
    // lowered to the property channel (marker name remains `prop:items`).
    const root = document.createElement('div')
    root.innerHTML = '<my-grid prop:items="b0"></my-grid>'

    const markers = collectElementMarkers(root)

    // The binding key `b0` must resolve to the <my-grid> element so the
    // compiler-emitted `if (__m["b0"]) hydrateProp(__m["b0"], …)` fires.
    expect(markers.b0).toBe(root.querySelector('my-grid'))
  })

  it('still collects the other marker kinds (positive control)', () => {
    const root = document.createElement('div')
    root.innerHTML =
      '<a attr:href="b0"></a><span prop:items="b1"></span>' +
      '<input bind:value="b2" /><button on:click="b3"></button>'

    const markers = collectElementMarkers(root)
    expect(markers.b0).toBe(root.querySelector('a'))
    expect(markers.b1).toBe(root.querySelector('span'))
    expect(markers.b2).toBe(root.querySelector('input'))
    expect(markers.b3).toBe(root.querySelector('button'))
  })
})

describe('hydrate — prop survives SSR → hydrate', () => {
  it('assigns the SSR-bound prop onto the child element during hydration', () => {
    // Reconstruct the compiler-emitted hydrate path end-to-end:
    //   1. SSR HTML with the `prop:items="b0"` wire marker.
    //   2. collectElementMarkers → key map.
    //   3. the compiled `if (__m["b0"]) hydrateProp(__m["b0"], "items", …)`.
    const items = signal<string[]>(['a', 'b'])
    const root = document.createElement('div')
    root.innerHTML = '<my-grid prop:items="b0"></my-grid>'

    const child = root.querySelector('my-grid') as HTMLElement & {
      items?: string[]
    }
    // Before hydration the prop is not on the JS object — SSR only wrote
    // the marker attribute, not the live value.
    expect(child.items).toBeUndefined()

    // The compiler-emitted dispatch.
    const m = collectElementMarkers(root)
    const cleanups: Array<() => void> = []
    if (m.b0) cleanups.push(hydrateProp(m.b0, 'items', () => items()))

    // The child component must now actually hold the prop value, and
    // track subsequent signal writes.
    expect(child.items).toEqual(['a', 'b'])
    items.set(['x', 'y', 'z'])
    expect(child.items).toEqual(['x', 'y', 'z'])

    for (const c of cleanups) c()
  })
})
