// Slice 2 — behavioral spec for <sui-prop-knobs>, the CEM-driven playground
// control set (build-spec §The viewing components #2 + behavioral specs #6/#7).
//
// TESTS-FIRST / RED: this file imports the not-yet-authored
// `sui-prop-knobs.sui` + `sui-prop-knobs.controller` (Slice 2 source). They do
// not exist yet, so the suite is RED by construction — no impl is written here.
//
// The knobs are a PURE TRANSFORM over CEM `attributes[]` → a FOUR-way
// classification (enum→segmented / boolean→switch / number→slider|number-input
// / string→text). The CEM array, and the controlled `propValues` snapshot,
// arrive via `prop:*` IDENTITY (set as JS properties on the host) — NEVER a
// json/object attribute (Cardinal: `<prop type=json>` does not exist). Driven
// through `renderComponent` (the production Rust/WASM compiler under happy-dom —
// the authority/charts precedent), with `@ship/sui-primitives` imported to
// register the composed sui-toggle-group / sui-switch / sui-slider / sui-input.

import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { afterEach, describe, expect, it } from 'vitest'
import { renderComponent } from '@ship/test-setup'

// RED: these two modules are the Slice-2 deliverable; they do not exist yet.
import '@ship/sui-primitives' // register the composed enum/bool/number/string controls
import controller from '../src/sui-prop-knobs/sui-prop-knobs.controller'

const SRC = readFileSync(resolve(__dirname, '../src/sui-prop-knobs/sui-prop-knobs.sui'), 'utf8')
const TAG = 'sui-prop-knobs'

// ---------------------------------------------------------------------------
// The CEM `attributes[]` shape the knobs consume (verified `cem.rs` `attr_entry`):
// `{ name, type: { text }, default?, values? }`. `type.text` is 'boolean' /
// 'number' / 'string', or (for a union) the `'a' | 'b'` joined string with a
// separate `values: string[]`. This is the canonical knob source.
// ---------------------------------------------------------------------------
interface CemAttr {
  name: string
  type: { text: string }
  default?: string
  values?: string[]
}

const ATTR_ENUM_VARIANT: CemAttr = {
  name: 'variant',
  type: { text: "'outline' | 'ghost' | 'segmented'" },
  default: 'outline',
  values: ['outline', 'ghost', 'segmented'],
}
const ATTR_ENUM_SIZE: CemAttr = {
  name: 'size',
  type: { text: "'sm' | 'md' | 'lg'" },
  default: 'sm',
  values: ['sm', 'md', 'lg'],
}
const ATTR_BOOL_DISABLED: CemAttr = { name: 'disabled', type: { text: 'boolean' }, default: 'false' }
const ATTR_NUMBER_MAX: CemAttr = { name: 'max', type: { text: 'number' }, default: '100' }
const ATTR_STRING_LABEL: CemAttr = { name: 'label', type: { text: 'string' }, default: '' }

const ALL_KINDS: CemAttr[] = [
  ATTR_ENUM_VARIANT,
  ATTR_BOOL_DISABLED,
  ATTR_NUMBER_MAX,
  ATTR_STRING_LABEL,
]

let mounted: { unmount(): void } | null = null
afterEach(() => {
  try {
    mounted?.unmount()
  } catch {
    /* best-effort */
  }
  mounted = null
})

/** Mount <sui-prop-knobs>; scalar props via `attrs`, structured props
 *  (`props` = the CEM array, `value` = the controlled propValues) assigned as
 *  JS PROPERTIES on the host (identity) AFTER render — the prop:* identity path. */
function mountKnobs(
  props: { props?: CemAttr[]; value?: Record<string, unknown> } = {},
  attrs: Record<string, string | boolean> = {},
): ReturnType<typeof renderComponent> {
  const c = renderComponent(SRC, TAG, { controller, attrs })
  mounted = c
  for (const [k, v] of Object.entries(props)) {
    ;(c.el as unknown as Record<string, unknown>)[k] = v
  }
  return c
}

// The shipped component shadow-encapsulates (deck compiles `shadow: true`), but
// the `renderComponent` harness compiles `shadow: false`, so the template
// renders into the host's LIGHT DOM. Query the rendered subtree at the
// host-or-shadow root, whichever this build target uses.
const root = (el: HTMLElement): ParentNode => el.shadowRoot ?? el
const qa = (el: HTMLElement, sel: string) => Array.from(root(el).querySelectorAll(sel))

// Find the control element rendered for a given prop `name`. The template tags
// each knob row with `data-knob="<name>"` so a test (and a host) can target one.
const knobRow = (el: HTMLElement, name: string) => root(el).querySelector(`[data-knob="${name}"]`)

describe('<sui-prop-knobs> — registration + shell', () => {

  it('renders its knob panel subtree (the sui-stack part)', async () => {
    const c = mountKnobs({ props: [] })
    await c.flush()
    expect(root(c.el).querySelector('[part="knobs"]')).not.toBeNull()
  })

  it('renders one knob row per CEM attribute', async () => {
    const c = mountKnobs({ props: ALL_KINDS })
    await c.flush()
    expect(qa(c.el, '[data-knob]').length).toBe(ALL_KINDS.length)
  })
})

describe('<sui-prop-knobs> — FOUR-way classification', () => {
  it('values[] non-empty → enum control is a segmented sui-toggle-group', async () => {
    const c = mountKnobs({ props: [ATTR_ENUM_VARIANT] })
    await c.flush()
    const row = knobRow(c.el, 'variant') as HTMLElement
    expect(row).not.toBeNull()
    const group = row.querySelector('sui-toggle-group') as HTMLElement
    expect(group).not.toBeNull()
    expect(group.getAttribute('variant')).toBe('segmented')
    // one sui-toggle per enum member
    expect(group.querySelectorAll('sui-toggle').length).toBe(ATTR_ENUM_VARIANT.values!.length)
  })

  it("type 'boolean' → bool control is a sui-switch (or sui-checkbox)", async () => {
    const c = mountKnobs({ props: [ATTR_BOOL_DISABLED] })
    await c.flush()
    const row = knobRow(c.el, 'disabled') as HTMLElement
    expect(row).not.toBeNull()
    expect(row.querySelector('sui-switch, sui-checkbox')).not.toBeNull()
  })

  it("type 'number' (no values) → numeric control (sui-slider or sui-input type=number)", async () => {
    const c = mountKnobs({ props: [ATTR_NUMBER_MAX] })
    await c.flush()
    const row = knobRow(c.el, 'max') as HTMLElement
    expect(row).not.toBeNull()
    const numeric = row.querySelector('sui-slider, sui-input[type="number"]')
    expect(numeric).not.toBeNull()
  })

  it("else → string control is a text sui-input", async () => {
    const c = mountKnobs({ props: [ATTR_STRING_LABEL] })
    await c.flush()
    const row = knobRow(c.el, 'label') as HTMLElement
    expect(row).not.toBeNull()
    const input = row.querySelector('sui-input') as HTMLElement
    expect(input).not.toBeNull()
    // string knob is NOT a number input
    expect(input.getAttribute('type')).not.toBe('number')
  })

  it('renders a label per knob (sui-label)', async () => {
    const c = mountKnobs({ props: [ATTR_STRING_LABEL] })
    await c.flush()
    const row = knobRow(c.el, 'label') as HTMLElement
    expect(row.querySelector('sui-label')).not.toBeNull()
  })
})

describe('<sui-prop-knobs> — default seeding', () => {
  it('seeds defaults from prop.default and emits the seeded snapshot on mount (sui-prop-reset)', async () => {
    let resetDetail: { values?: Record<string, unknown> } | null = null
    const c = renderComponent(SRC, TAG, { controller })
    mounted = c
    c.el.addEventListener('sui-prop-reset', (e) => {
      resetDetail = (e as CustomEvent).detail as { values?: Record<string, unknown> }
    })
    ;(c.el as unknown as Record<string, unknown>).props = [ATTR_STRING_LABEL, ATTR_NUMBER_MAX]
    await c.flush()
    expect(resetDetail).not.toBeNull()
    // numeric default is COERCED to a number in the seeded snapshot
    expect(resetDetail!.values!.max).toBe(100)
    expect(typeof resetDetail!.values!.max).toBe('number')
  })

  it('prefers the largest size-enum variant when seeding a size knob default', async () => {
    // An enum named `size` with no explicit default seeds the largest legible
    // variant (studio legibility rule): `lg` over `sm`/`md`.
    const sizeNoDefault: CemAttr = {
      name: 'size',
      type: { text: "'sm' | 'md' | 'lg'" },
      values: ['sm', 'md', 'lg'],
    }
    let resetDetail: { values?: Record<string, unknown> } | null = null
    const c = renderComponent(SRC, TAG, { controller })
    mounted = c
    c.el.addEventListener('sui-prop-reset', (e) => {
      resetDetail = (e as CustomEvent).detail as { values?: Record<string, unknown> }
    })
    ;(c.el as unknown as Record<string, unknown>).props = [sizeNoDefault]
    await c.flush()
    expect(resetDetail).not.toBeNull()
    expect(resetDetail!.values!.size).toBe('lg')
  })
})

describe('<sui-prop-knobs> — sui-prop-change emission', () => {
  it('emits sui-prop-change {name,value,values} (bubbles + composed) on an enum edit', async () => {
    const c = mountKnobs({ props: [ATTR_ENUM_VARIANT], value: { variant: 'outline' } })
    await c.flush()
    let detail: { name?: string; value?: unknown; values?: Record<string, unknown> } | null = null
    let bubbles = false
    let composed = false
    c.el.addEventListener('sui-prop-change', (e) => {
      detail = (e as CustomEvent).detail as typeof detail
      bubbles = (e as CustomEvent).bubbles
      composed = (e as CustomEvent).composed
    })
    const group = (knobRow(c.el, 'variant') as HTMLElement).querySelector('sui-toggle-group') as HTMLElement
    // The composed sui-toggle-group emits `sui-change {value}`; the knob
    // controller maps it to `sui-prop-change {name,value,values}`.
    group.dispatchEvent(
      new CustomEvent('sui-change', { detail: { value: 'ghost' }, bubbles: true, composed: true }),
    )
    await c.flush()
    expect(detail).not.toBeNull()
    expect(detail!.name).toBe('variant')
    expect(detail!.value).toBe('ghost')
    // the full snapshot rides along so a consumer wires it straight into render
    expect(detail!.values!.variant).toBe('ghost')
    expect(bubbles).toBe(true)
    expect(composed).toBe(true)
  })

  it('coerces a number knob edit to a NUMBER before emitting (never a string)', async () => {
    const c = mountKnobs({ props: [ATTR_NUMBER_MAX], value: { max: 100 } })
    await c.flush()
    let detail: { value?: unknown; values?: Record<string, unknown> } | null = null
    c.el.addEventListener('sui-prop-change', (e) => {
      detail = (e as CustomEvent).detail as typeof detail
    })
    const numeric = (knobRow(c.el, 'max') as HTMLElement).querySelector(
      'sui-slider, sui-input[type="number"]',
    ) as HTMLElement
    expect(numeric).not.toBeNull()
    // sui-slider emits `sui-change {value:number}`; sui-input emits `sui-input
    // {value:string}` — either way the knob coerces to a number.
    numeric.dispatchEvent(
      new CustomEvent('sui-change', { detail: { value: 42 }, bubbles: true, composed: true }),
    )
    numeric.dispatchEvent(
      new CustomEvent('sui-input', { detail: { value: '42' }, bubbles: true, composed: true }),
    )
    await c.flush()
    expect(detail).not.toBeNull()
    expect(detail!.value).toBe(42)
    expect(typeof detail!.value).toBe('number')
    expect(typeof detail!.values!.max).toBe('number')
  })

  it('emits a boolean (not a string) on a bool-knob edit', async () => {
    const c = mountKnobs({ props: [ATTR_BOOL_DISABLED], value: { disabled: false } })
    await c.flush()
    let detail: { value?: unknown } | null = null
    c.el.addEventListener('sui-prop-change', (e) => {
      detail = (e as CustomEvent).detail as typeof detail
    })
    const toggle = (knobRow(c.el, 'disabled') as HTMLElement).querySelector(
      'sui-switch, sui-checkbox',
    ) as HTMLElement
    toggle.dispatchEvent(
      new CustomEvent('sui-change', { detail: { checked: true }, bubbles: true, composed: true }),
    )
    await c.flush()
    expect(detail).not.toBeNull()
    expect(detail!.value).toBe(true)
    expect(typeof detail!.value).toBe('boolean')
  })

  it('DEDUPES: the same (name,value) edit is not emitted twice in a row (lastEmitted guard)', async () => {
    const c = mountKnobs({ props: [ATTR_ENUM_VARIANT], value: { variant: 'outline' } })
    await c.flush()
    let count = 0
    c.el.addEventListener('sui-prop-change', () => {
      count++
    })
    const group = (knobRow(c.el, 'variant') as HTMLElement).querySelector('sui-toggle-group') as HTMLElement
    group.dispatchEvent(
      new CustomEvent('sui-change', { detail: { value: 'ghost' }, bubbles: true, composed: true }),
    )
    await c.flush()
    // identical edit again — the lastEmitted guard collapses the duplicate
    group.dispatchEvent(
      new CustomEvent('sui-change', { detail: { value: 'ghost' }, bubbles: true, composed: true }),
    )
    await c.flush()
    expect(count).toBe(1)
  })
})

describe('<sui-prop-knobs> — controlled prop:value round-trip', () => {
  it('reflects a controlled prop:value into the enum control (the deep-link seam drives it)', async () => {
    const c = mountKnobs({ props: [ATTR_ENUM_VARIANT], value: { variant: 'outline' } })
    await c.flush()
    let group = (knobRow(c.el, 'variant') as HTMLElement).querySelector('sui-toggle-group') as HTMLElement
    expect(group.getAttribute('value')).toBe('outline')

    // Host drives a new controlled value (URL/deep-link mirror) — the control
    // reflects it without an internal edit.
    ;(c.el as unknown as Record<string, unknown>).value = { variant: 'segmented' }
    await c.flush()
    group = (knobRow(c.el, 'variant') as HTMLElement).querySelector('sui-toggle-group') as HTMLElement
    expect(group.getAttribute('value')).toBe('segmented')
  })
})

describe('<sui-prop-knobs> — standalone null-fallback (no provider)', () => {
  it('with NO sui-component-viewer ancestor, useShared returns null and the tag operates off prop:value/prop:props', async () => {
    // Mounted bare (no provider in ancestry) — the tag must still classify +
    // render + emit purely from its controlled inputs, never erroring on the
    // null shared context.
    const c = mountKnobs({ props: ALL_KINDS, value: { variant: 'ghost', disabled: true, max: 50, label: 'hi' } })
    await c.flush()
    // it rendered every knob from prop:props alone...
    expect(qa(c.el, '[data-knob]').length).toBe(ALL_KINDS.length)
    // ...and an edit still emits (operating off the controlled inputs)
    let fired = false
    c.el.addEventListener('sui-prop-change', () => {
      fired = true
    })
    const input = (knobRow(c.el, 'label') as HTMLElement).querySelector('sui-input') as HTMLElement
    input.dispatchEvent(
      new CustomEvent('sui-input', { detail: { value: 'edited' }, bubbles: true, composed: true }),
    )
    await c.flush()
    expect(fired).toBe(true)
  })
})

describe('<sui-prop-knobs> — empty CEM', () => {
  it('renders no knob rows when props is empty', async () => {
    const c = mountKnobs({ props: [] })
    await c.flush()
    expect(qa(c.el, '[data-knob]').length).toBe(0)
  })
})

describe('<sui-prop-knobs> — controller purity (grep)', () => {
  it('the controller never constructs view markup (no innerHTML/outerHTML/insertAdjacentHTML)', () => {
    const ctrl = readFileSync(
      resolve(__dirname, '../src/sui-prop-knobs/sui-prop-knobs.controller.ts'),
      'utf8',
    )
    expect(ctrl).not.toMatch(/\.innerHTML\b/)
    expect(ctrl).not.toMatch(/\.outerHTML\b/)
    expect(ctrl).not.toMatch(/insertAdjacentHTML/)
  })
})
