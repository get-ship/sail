// Element-scope tree — custom elements share scope-homed stores.
//
// The regression this guards: component.ts used to `createScope()` per
// element, making every element its own chain root — three components
// sharing one `defineStore` got three instances (the store-counter /
// store-total docs-demo bug). Element scopes must instead form a tree
// mirroring the DOM: nearest sail ancestor's scope as parent, else the
// shared per-document client root.

import { beforeEach, describe, expect, it } from 'vitest'
import { defineComponent, defineStore, useStore, provideStore, signal } from '../src/index'
import { clientRootScope, elementScopeOf } from '../src/internal'
import { _resetClientRootScope } from '../src/element-scope'

let tagCounter = 0
const nextTag = () => `el-scope-test-${++tagCounter}`
let storeCounter = 0
const nextStoreId = () => `unit.el-scope.${++storeCounter}`

const emptySchema = { observedAttrs: [], props: {} }

beforeEach(() => {
  document.body.innerHTML = ''
  _resetClientRootScope()
})

interface Tally {
  n: ReturnType<typeof signal<number>>
  inc: () => void
}
const makeTallyDef = () =>
  defineStore<Tally>(nextStoreId(), () => {
    const n = signal(0)
    return { n, inc: () => n.set(n() + 1) }
  })

describe('cross-element store sharing', () => {
  it('sibling elements resolve the SAME root-homed instance', () => {
    const def = makeTallyDef()
    const seen: Tally[] = []
    const tag = nextTag()
    defineComponent(tag, emptySchema, () => {
      seen.push(useStore(def))
      return () => {}
    })

    for (let i = 0; i < 3; i++) {
      document.body.appendChild(document.createElement(tag))
    }
    expect(seen).toHaveLength(3)
    expect(seen[1]).toBe(seen[0])
    expect(seen[2]).toBe(seen[0])

    // One writer, every reader moves — the demo's acceptance shape:
    // two counters + a total all tracking one tally.
    seen[0]!.inc()
    seen[0]!.inc()
    seen[1]!.inc()
    for (const inst of seen) expect(inst.n()).toBe(3)
  })

  it('element scopes chain to the per-document client root', () => {
    const tag = nextTag()
    defineComponent(tag, emptySchema, () => () => {})
    const el = document.createElement(tag)
    document.body.appendChild(el)

    let cur = elementScopeOf(el)
    expect(cur).not.toBeNull()
    while (cur!.parent) cur = cur!.parent
    expect(cur).toBe(clientRootScope())
  })

  it('a nested element parents into its ancestor element scope', () => {
    const parentTag = nextTag()
    const childTag = nextTag()
    defineComponent(parentTag, emptySchema, () => () => {})
    defineComponent(childTag, emptySchema, () => () => {})

    const parent = document.createElement(parentTag)
    const child = document.createElement(childTag)
    parent.appendChild(child)
    document.body.appendChild(parent)

    expect(elementScopeOf(child)!.parent).toBe(elementScopeOf(parent))
  })
})

describe('ancestor-element provideStore override', () => {
  it('descendant elements see the override; outside siblings keep the root', () => {
    const def = makeTallyDef()
    const providerTag = nextTag()
    const readerTag = nextTag()

    let provided: Tally | null = null
    defineComponent(providerTag, emptySchema, () => {
      provided = provideStore(def)
      return () => {}
    })
    const seen = new Map<Element, Tally>()
    defineComponent(readerTag, emptySchema, (host) => {
      seen.set(host, useStore(def))
      return () => {}
    })

    // <provider><reader/></provider> alongside a top-level <reader/>.
    const provider = document.createElement(providerTag)
    const innerReader = document.createElement(readerTag)
    provider.appendChild(innerReader)
    const outerReader = document.createElement(readerTag)
    document.body.append(provider, outerReader)

    const inner = seen.get(innerReader)!
    const outer = seen.get(outerReader)!
    expect(inner).toBe(provided)
    expect(outer).not.toBe(provided)

    // Writes stay partitioned: the override subtree and the root
    // instance evolve independently.
    inner.inc()
    expect(inner.n()).toBe(1)
    expect(outer.n()).toBe(0)
  })
})

describe('reconnect re-resolves the parent', () => {
  it('moving an element out of a provider subtree drops the override', () => {
    const def = makeTallyDef()
    const providerTag = nextTag()
    const readerTag = nextTag()

    let provided: Tally | null = null
    defineComponent(providerTag, emptySchema, () => {
      provided = provideStore(def)
      return () => {}
    })
    let latest: Tally | null = null
    defineComponent(readerTag, emptySchema, () => {
      latest = useStore(def)
      return () => {}
    })

    const provider = document.createElement(providerTag)
    const reader = document.createElement(readerTag)
    provider.appendChild(reader)
    document.body.appendChild(provider)
    expect(latest).toBe(provided)

    // Move: disconnect + reconnect outside the provider — the factory
    // re-runs and resolves the root instance instead.
    document.body.appendChild(reader)
    expect(latest).not.toBe(provided)
  })
})
