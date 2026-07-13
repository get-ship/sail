// Slice 4 — behavioral spec for <sui-source-panel>, the source + paste-able
// snippet primitive (build-spec §The viewing components #5).
//
// TESTS-FIRST / RED: this file imports the not-yet-authored
// `sui-source-panel.sui` + `.controller` + the `snippet.ts` builder (the Slice-4
// deliverable). They do not exist yet, so the suite is RED by construction.
//
// Two test surfaces, both framework-pure:
//   1. `buildSnippet` / `copyText` — the pure default-diff builder + the ONE
//      copy helper, unit-tested directly (no DOM mount). This is the studio
//      `snippet.ts` logic reproduced over the CEM `attributes[]` shape.
//   2. <sui-source-panel> — driven through `renderComponent` (the production
//      Rust/WASM compiler under happy-dom, the authority/charts convention),
//      composing the dissolved code display (bare `<pre>` + sx — sui-code was
//      `@ship/sui-primitives`. Structured data (source, CEM attrs, propValues,
//      slot content) arrives via JS PROPERTIES (the `prop:*` identity rule) —
//      scalar control props (`component-name`, `lang`) via `attrs`.

import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { renderComponent } from '@ship/test-setup'

import controller from '../src/sui-source-panel/sui-source-panel.controller'
import { buildSnippet, copyText, type CemAttr } from '../src/sui-source-panel/snippet'

const SRC = readFileSync(resolve(__dirname, '../src/sui-source-panel/sui-source-panel.sui'), 'utf8')
const TAG = 'sui-source-panel'

// A representative CEM `attributes[]` slice (the shape the host bakes from each
// component's `<tag>.custom-elements.json`): `name`/`type:{text}`/`default`/
// `values`. The diff reads `default` to decide what to omit.
const BUTTON_CEM: CemAttr[] = [
  { name: 'variant', type: { text: "'solid' | 'soft' | 'outline'" }, default: 'default', values: ['solid', 'soft', 'outline', 'default'] },
  { name: 'size', type: { text: "'xs' | 'sm' | 'md' | 'lg'" }, default: 'md', values: ['xs', 'sm', 'md', 'lg'] },
  { name: 'label', type: { text: 'string' }, default: '' },
  { name: 'disabled', type: { text: 'boolean' }, default: 'false' },
  { name: 'loading', type: { text: 'boolean' }, default: 'false' },
]

// ---------------------------------------------------------------------------
// 1. buildSnippet — the pure default-diff builder (studio `snippet.ts` logic).
// ---------------------------------------------------------------------------
describe('buildSnippet — default-diffing', () => {
  it('emits ONLY props that differ from the CEM default', () => {
    const out = buildSnippet('button', { variant: 'solid', size: 'md', label: '' }, BUTTON_CEM, undefined)
    // variant differs from default ('default') → emitted; size matches default
    // ('md') → omitted; empty label → omitted.
    expect(out).toContain('variant="solid"')
    expect(out).not.toContain('size=')
    expect(out).not.toContain('label=')
  })

  it('omits a prop whose value equals the declared default', () => {
    const out = buildSnippet('button', { variant: 'default', size: 'sm' }, BUTTON_CEM, undefined)
    // variant='default' matches default → omitted; size='sm' differs → emitted.
    expect(out).not.toContain('variant=')
    expect(out).toContain('size="sm"')
  })

  it('booleans use bare-attribute presence, omitted when false/default', () => {
    const out = buildSnippet('button', { disabled: true, loading: false }, BUTTON_CEM, undefined)
    // disabled true (default false) → bare attribute, NO ="true"; loading false
    // (matches default) → omitted entirely.
    expect(out).toContain(' disabled')
    expect(out).not.toContain('disabled="true"')
    expect(out).not.toContain('disabled="false"')
    expect(out).not.toContain('loading')
  })

  it('escapes string attribute values', () => {
    const out = buildSnippet('button', { label: 'a "b" & <c>' }, BUTTON_CEM, undefined)
    expect(out).toContain('label="a &quot;b&quot; &amp; <c>"')
    // raw double-quote / ampersand never leak into the attribute value
    expect(out).not.toContain('label="a "b"')
  })

  it('inner = composed slot content when supplied', () => {
    const out = buildSnippet('button', { variant: 'solid' }, BUTTON_CEM, '<sui-icon name="check"></sui-icon>Save')
    expect(out).toBe('<button variant="solid"><sui-icon name="check"></sui-icon>Save</button>')
  })

  it('inner falls back to a humanized slotLabel when no slot content is supplied', () => {
    const out = buildSnippet('button', {}, BUTTON_CEM, undefined)
    // no diffing props, no slots → `<button>Button</button>` (slotLabel)
    expect(out).toBe('<button>Button</button>')
  })

  it('no-CEM degraded mode: emits ALL current prop values (no diff)', () => {
    // CEM absent ⇒ the builder cannot know declared defaults, so it emits every
    // current value (the clearly-degraded mode). Booleans still use presence.
    const out = buildSnippet('button', { variant: 'soft', size: 'md', disabled: true }, undefined, undefined)
    expect(out).toContain('variant="soft"')
    expect(out).toContain('size="md"') // NOT diffed away — no default known
    expect(out).toContain(' disabled')
    expect(out).not.toContain('disabled="true"')
  })

  it('no-CEM degraded mode: false booleans + empty strings are still skipped', () => {
    // Degraded mode emits current values, but a false boolean / empty string
    // carries no information — they are skipped so the snippet stays clean.
    const out = buildSnippet('button', { variant: '', disabled: false, loading: true }, undefined, undefined)
    expect(out).not.toContain('variant')
    expect(out).not.toContain('disabled')
    expect(out).toContain(' loading')
  })
})

// ---------------------------------------------------------------------------
// 2. copyText — the ONE clipboard helper (reused by sui-render's error branch).
// ---------------------------------------------------------------------------
describe('copyText — the single copy affordance', () => {
  it('writes the text to navigator.clipboard and resolves true', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined)
    const ok = await copyText('hello-clip', { clipboard: { writeText } } as unknown as Navigator)
    expect(writeText).toHaveBeenCalledWith('hello-clip')
    expect(ok).toBe(true)
  })

  it('resolves false (no throw) when the clipboard write rejects', async () => {
    const writeText = vi.fn().mockRejectedValue(new Error('denied'))
    const ok = await copyText('x', { clipboard: { writeText } } as unknown as Navigator)
    expect(ok).toBe(false)
  })

  it('resolves false (no throw) when no clipboard API is present', async () => {
    const ok = await copyText('x', {} as unknown as Navigator)
    expect(ok).toBe(false)
  })
})

// ---------------------------------------------------------------------------
// 3. <sui-source-panel> — the mounted tag (compose + copy + bindHtml + events).
// ---------------------------------------------------------------------------
let mounted: { unmount(): void } | null = null
let writeText: ReturnType<typeof vi.fn>

beforeEach(() => {
  // happy-dom does not implement navigator.clipboard — stub a resolving writer
  // so the copy affordance can be exercised. The ONE clipboard path in the pack.
  writeText = vi.fn().mockResolvedValue(undefined)
  Object.defineProperty(globalThis.navigator, 'clipboard', {
    configurable: true,
    value: { writeText },
  })
})

afterEach(() => {
  try {
    mounted?.unmount()
  } catch {
    /* best-effort */
  }
  mounted = null
})

function mountPanel(
  attrs: Record<string, string | boolean>,
  props: Record<string, unknown> = {},
): ReturnType<typeof renderComponent> {
  const c = renderComponent(SRC, TAG, { controller, attrs })
  mounted = c
  for (const [k, v] of Object.entries(props)) {
    ;(c.el as unknown as Record<string, unknown>)[k] = v
  }
  return c
}

// The renderComponent harness mounts a component into LIGHT DOM (scoped via
// `data-component`, not a shadow root) — verified consistent across the pack.
// The rendered subtree therefore lives directly under the host element; query
// it via the host, not `el.shadowRoot`.
const root = (el: HTMLElement) => el

describe('<sui-source-panel> — registration + composition', () => {
  it('composes the dissolved code display (bare pre) + button (copy action)', async () => {
    const c = mountPanel(
      { 'component-name': 'button' },
      { source: '<button>hi</button>', props: BUTTON_CEM, value: {} },
    )
    await c.flush()
    expect(root(c.el).querySelector('.source-code pre')).not.toBeNull()
    expect(root(c.el).querySelector('button')).not.toBeNull()
  })
})

describe('<sui-source-panel> — source view', () => {
  it('renders the raw .sui source verbatim (via bindHtml/text, escaped)', async () => {
    const raw = '<button variant="solid">Save & <go></button>'
    const c = mountPanel(
      { 'component-name': 'button' },
      { source: raw, props: BUTTON_CEM, value: { variant: 'solid' } },
    )
    await c.flush()
    // The source is shown verbatim. Reading textContent recovers the raw
    // characters (so the `&` / `<` were rendered as text, not parsed as markup).
    const text = root(c.el).textContent ?? ''
    expect(text).toContain('<button variant="solid">Save & <go></button>')
  })
})

describe('<sui-source-panel> — recomputed snippet', () => {
  it('renders a snippet emitting only non-default props from current values', async () => {
    const c = mountPanel(
      { 'component-name': 'button' },
      {
        source: '<button></button>',
        props: BUTTON_CEM,
        value: { variant: 'solid', size: 'md' },
        slotContent: 'Save',
      },
    )
    await c.flush()
    const text = root(c.el).textContent ?? ''
    expect(text).toContain('<button variant="solid">Save</button>')
    // size='md' equals the declared default → diffed away.
    expect(text).not.toContain('size=')
  })

  it('recomputes the snippet when prop values change (reactive)', async () => {
    const c = mountPanel(
      { 'component-name': 'button' },
      { source: '<button></button>', props: BUTTON_CEM, value: { variant: 'solid' } },
    )
    await c.flush()
    expect(root(c.el).textContent ?? '').toContain('variant="solid"')

    ;(c.el as unknown as Record<string, unknown>).value = { variant: 'soft' }
    await c.flush()
    const text = root(c.el).textContent ?? ''
    expect(text).toContain('variant="soft"')
    expect(text).not.toContain('variant="solid"')
  })

  it('no-CEM degraded mode: emits all current props (no diff) when props is absent', async () => {
    const c = mountPanel(
      { 'component-name': 'button' },
      // props (CEM) deliberately omitted — degraded mode.
      { source: '<button></button>', value: { variant: 'soft', size: 'md' } },
    )
    await c.flush()
    const text = root(c.el).textContent ?? ''
    // both emitted — no defaults known to diff against.
    expect(text).toContain('variant="soft"')
    expect(text).toContain('size="md"')
  })
})

describe('<sui-source-panel> — copy affordance (the ONE clipboard path)', () => {
  it('Copy writes the recomputed snippet to navigator.clipboard', async () => {
    const c = mountPanel(
      { 'component-name': 'button' },
      { source: '<button></button>', props: BUTTON_CEM, value: { variant: 'solid' }, slotContent: 'Save' },
    )
    await c.flush()
    const btn = root(c.el).querySelector('button') as HTMLElement
    expect(btn).not.toBeNull()
    btn.click()
    await c.flush()
    expect(writeText).toHaveBeenCalledTimes(1)
    expect(writeText.mock.calls[0][0]).toContain('<button variant="solid">Save</button>')
  })

  it('shows transient "Copied" feedback after a successful copy', async () => {
    const c = mountPanel(
      { 'component-name': 'button' },
      { source: '<button></button>', props: BUTTON_CEM, value: { variant: 'solid' } },
    )
    await c.flush()
    const btn = root(c.el).querySelector('button') as HTMLElement
    btn.click()
    await c.flush()
    expect(root(c.el).textContent ?? '').toMatch(/copied/i)
  })

  it('emits sui-source-copy {text} (bubbles + composed) on copy', async () => {
    const c = mountPanel(
      { 'component-name': 'button' },
      { source: '<button></button>', props: BUTTON_CEM, value: { variant: 'solid' }, slotContent: 'Save' },
    )
    await c.flush()
    let detail: { text?: string } | null = null
    let bubbles = false
    let composed = false
    c.el.addEventListener('sui-source-copy', (e) => {
      detail = (e as CustomEvent).detail as { text?: string }
      bubbles = (e as CustomEvent).bubbles
      composed = (e as CustomEvent).composed
    })
    const btn = root(c.el).querySelector('button') as HTMLElement
    btn.click()
    await c.flush()
    expect(detail).not.toBeNull()
    expect(detail!.text).toContain('<button variant="solid">Save</button>')
    expect(bubbles).toBe(true)
    expect(composed).toBe(true)
  })

  it('sui-source-copy is DEDUPED per click — one event per press, not two', async () => {
    const c = mountPanel(
      { 'component-name': 'button' },
      { source: '<button></button>', props: BUTTON_CEM, value: { variant: 'solid' } },
    )
    await c.flush()
    let count = 0
    c.el.addEventListener('sui-source-copy', () => {
      count++
    })
    const btn = root(c.el).querySelector('button') as HTMLElement
    btn.click()
    await c.flush()
    expect(count).toBe(1)
  })
})

describe('<sui-source-panel> — controller purity', () => {
  it('the controller never names innerHTML/outerHTML/insertAdjacentHTML', () => {
    const ctrl = readFileSync(
      resolve(__dirname, '../src/sui-source-panel/sui-source-panel.controller.ts'),
      'utf8',
    )
    // The rule targets DOM-construction CALLS / assignments, not prose: match
    // the property-access / call forms (`x.innerHTML =`, `el.insertAdjacentHTML(`).
    expect(ctrl).not.toMatch(/\.innerHTML\s*=/)
    expect(ctrl).not.toMatch(/\.outerHTML\s*=/)
    expect(ctrl).not.toMatch(/\.insertAdjacentHTML\s*\(/)
  })
})
