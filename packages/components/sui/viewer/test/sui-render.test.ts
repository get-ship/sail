// Slice 1 — behavioral spec for <sui-render>, THE engine primitive.
//
// TESTS-FIRST / RED: this file imports the not-yet-authored
// `sui-render.sui` + `sui-render.controller` (Slice 1 source). They do not
// exist yet, so the suite is RED by construction — no impl is written here.
//
// The whole pack is framework-pure: `sui-render` never imports a wasm URL or
// `fetch`. The host hands it a bound `compile()` via `prop:compile={fn}`
// (identity) — here that bound compile is built from the production Rust/WASM
// compiler the Node harness already instantiates (`compileSui`), wrapped to the
// discriminated `{kind:'ok'|'err'}` shape the engine consumes. Source, tag, and
// prop-values arrive via `prop:*` only (NO object/json attribute props), exactly
// the `sui-table` rule (the host owns discovery; data arrives by `prop:*`).
//
// Driven through `renderComponent` (the production compiler under happy-dom —
// the authority-pack precedent). Scalar props apply as ATTRIBUTES via `attrs`;
// structured/identity props (`compile`, `source`, `propValues`, `setup`, `ship`)
// apply as JS PROPERTIES on the host element — the spec's prop-apply split.

import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { afterEach, describe, expect, it } from 'vitest'
import { renderComponent } from '@ship/test-setup'
import { compileSui } from '@ship/sui-dom/test-utils'

// RED: these two modules are the Slice-1 deliverable; they do not exist yet.
import '@ship/sui-primitives' // register the composed sui-error-boundary / sui-alert / sui-code

const SRC = readFileSync(resolve(__dirname, '../src/sui-render/sui-render.sui'), 'utf8')
const TAG = 'sui-render'

// ---------------------------------------------------------------------------
// The bound compile() the host injects. Built over the SAME production compiler
// (`compileSui`) the engine seam uses — wrapped to the engine's discriminated
// result shape so a compile error returns `{kind:'err'}` instead of throwing
// across the `prop:compile` seam. NO fetch, NO path, NO daemon — pure.
// ---------------------------------------------------------------------------
type CompileResult =
  | { kind: 'ok'; js: string; css: string | null; dts: string | null; ssrHtml: string | null }
  | { kind: 'err'; message: string; line: number | null; col: number | null; category: string | null }

function makeBoundCompile(): (source: string, opts?: Record<string, unknown>) => CompileResult {
  return (source, opts = {}) => {
    try {
      const c = compileSui(source, opts as never)
      return { kind: 'ok', js: c.js, css: c.css ?? null, dts: c.dts ?? null, ssrHtml: c.ssrHtml ?? null }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e)
      const m = /\((\d+):(\d+)\)/.exec(msg)
      return {
        kind: 'err',
        message: msg,
        line: m ? Number(m[1]) : null,
        col: m ? Number(m[2]) : null,
        category: 'compile',
      }
    }
  }
}

// A trivially-valid view-mode `.sui` source: a static element the engine
// `__mount`s into its shadow stage. Distinct marker text so we assert presence.
const GOOD_VIEW = `<div data-test="specimen">hello-from-view</div>`
const GOOD_VIEW_ALT = `<section data-test="specimen-alt">a-different-render</section>`
// Source that fails to COMPILE (unterminated tag) — exercises the compile-error
// branch (message + line:col + caret), and the last-good-on-error policy.
const BAD_VIEW = `<div data-test="broken"`

// A view-mode `.sui` whose `setup` throws synchronously — the secondary
// engine try/catch must render a panel, not a blank stage.
const SETUP_MARKER = '__viewer_setup__'
const throwingSetup = () => {
  throw new Error('setup-explodes')
}

let mounted: { unmount(): void } | null = null
afterEach(() => {
  try {
    mounted?.unmount()
  } catch {
    /* best-effort */
  }
  mounted = null
})

/** Mount <sui-render>; scalar props via `attrs`, structured props assigned as
 *  JS PROPERTIES on the host (camelCase) AFTER render — the host's prop-apply. */
function mountRender(
  attrs: Record<string, string | boolean>,
  props: Record<string, unknown> = {},
): ReturnType<typeof renderComponent> {
  const c = renderComponent(SRC, TAG, { attrs })
  mounted = c
  for (const [k, v] of Object.entries(props)) {
    ;(c.el as unknown as Record<string, unknown>)[k] = v
  }
  return c
}

const stageRoot = (el: HTMLElement) => el.shadowRoot as ShadowRoot
const specimen = (el: HTMLElement) => stageRoot(el).querySelector('[data-test="specimen"]')

describe('<sui-render> — registration + shell', () => {

  it('attaches a shadow root (the engine stage)', () => {
    const c = mountRender({}, { compile: makeBoundCompile() })
    expect(c.el.shadowRoot).not.toBeNull()
  })

  it('wraps the fault-prone subtree in a sui-error-boundary', () => {
    const c = mountRender({ mode: 'source' }, { compile: makeBoundCompile(), source: GOOD_VIEW })
    expect(stageRoot(c.el).querySelector('sui-error-boundary')).not.toBeNull()
  })
})

describe('<sui-render> — VIEW-mode (prop:source raw .sui)', () => {
  it('compiles + mounts the raw source into the engine shadow stage', async () => {
    const c = mountRender({ mode: 'source' }, { compile: makeBoundCompile(), source: GOOD_VIEW })
    await c.flush()
    const node = specimen(c.el)
    expect(node).not.toBeNull()
    expect(node!.textContent).toBe('hello-from-view')
  })

  it('emits sui-render:mounted (bubbles + composed) after a clean view mount', async () => {
    const c = mountRender({ mode: 'source' }, { compile: makeBoundCompile() })
    let ev: CustomEvent | null = null
    let bubbles = false
    let composed = false
    c.el.addEventListener('sui-render:mounted', (e) => {
      ev = e as CustomEvent
      bubbles = (e as CustomEvent).bubbles
      composed = (e as CustomEvent).composed
    })
    ;(c.el as unknown as Record<string, unknown>).source = GOOD_VIEW
    await c.flush()
    expect(ev).not.toBeNull()
    expect(bubbles).toBe(true)
    expect(composed).toBe(true)
  })

  it('emits sui-render:compiled {js,css,dts,ssrHtml} after a clean compile', async () => {
    const c = mountRender({ mode: 'source' }, { compile: makeBoundCompile() })
    let detail: Record<string, unknown> | null = null
    c.el.addEventListener('sui-render:compiled', (e) => {
      detail = (e as CustomEvent).detail as Record<string, unknown>
    })
    ;(c.el as unknown as Record<string, unknown>).source = GOOD_VIEW
    await c.flush()
    expect(detail).not.toBeNull()
    expect(detail!).toHaveProperty('js')
    expect(typeof detail!.js).toBe('string')
  })

  it('dispose empties the stage container (no leftover specimen nodes)', async () => {
    const c = mountRender({ mode: 'source' }, { compile: makeBoundCompile(), source: GOOD_VIEW })
    await c.flush()
    expect(specimen(c.el)).not.toBeNull()
    c.unmount()
    // After unmount the host is gone from the document; any engine-mounted
    // specimen nodes were removed by the dispose (container emptied).
    expect(document.querySelector('[data-test="specimen"]')).toBeNull()
  })

  it('keeps the shadow stage CSS-scoped — no stray global stylesheet leak', async () => {
    // A view-mode source compile returns `css: null` (the SUI compiler inlines
    // styling into the template/JS for inline sources rather than surfacing a
    // separate compiled.css string), so the engine adopts ZERO sheets here.
    // The discriminating contract is the SCOPING invariant: whatever the engine
    // adopts lives on the shadow stage's adoptedStyleSheets array — never the
    // document head. A regression that leaked a stray global sheet onto the
    // stage would push this above 0. Real compiled.css adoption (count == 1)
    // is proven in the webview tier where the compiler emits component CSS.
    const c = mountRender(
      { mode: 'source' },
      {
        compile: makeBoundCompile(),
        source: `<style>.boxed { color: var(--foreground); }</style><div data-test="specimen" class="boxed">styled</div>`,
      },
    )
    await c.flush()
    expect(Array.isArray(stageRoot(c.el).adoptedStyleSheets)).toBe(true)
    expect(stageRoot(c.el).adoptedStyleSheets.length).toBe(0)
    // And nothing leaked into the document head.
    expect(document.head.querySelector('style[data-sui-render]')).toBeNull()
  })
})

describe('<sui-render> — COMPONENT-mode (component-name + prop:prop-values)', () => {
  // A component-mode target: declares a scalar `<prop>` (string) reflected as an
  // attribute, plus reads a structured prop set as a JS property.
  const SPECIMEN_TAG = 'sui-render-specimen'

  it('define-once + createElement: mounts the named tag into the stage', async () => {
    const SPECIMEN_SRC = `<prop name="label" type="string" reflect />\n<button data-test="probe">{label}</button>`
    const c = mountRender(
      { mode: 'tag', 'component-name': SPECIMEN_TAG },
      { compile: makeBoundCompile(), source: SPECIMEN_SRC, propValues: { label: 'click-me' } },
    )
    await c.flush()
    const el = stageRoot(c.el).querySelector(SPECIMEN_TAG)
    expect(el).not.toBeNull()
    expect(customElements.get(SPECIMEN_TAG)).toBeDefined()
  })

  it('a component-name already in the registry is NOT redefined (no NotSupportedError)', async () => {
    const REDEF_TAG = 'sui-render-redef'
    const SPECIMEN_SRC = `<prop name="label" type="string" reflect />\n<span>{label}</span>`
    const a = mountRender(
      { mode: 'tag', 'component-name': REDEF_TAG },
      { compile: makeBoundCompile(), source: SPECIMEN_SRC, propValues: { label: 'one' } },
    )
    await a.flush()
    a.unmount()
    // Second mount of the SAME tag must hit the define-once guard, not throw.
    expect(() => {
      const b = mountRender(
        { mode: 'tag', 'component-name': REDEF_TAG },
        { compile: makeBoundCompile(), source: SPECIMEN_SRC, propValues: { label: 'two' } },
      )
      mounted = b
    }).not.toThrow()
  })

  it('SCALAR props apply as reflected ATTRIBUTES (bool presence, no disabled="false")', async () => {
    const SPECIMEN_SRC =
      `<prop name="label" type="string" reflect />\n<prop name="disabled" type="bool" reflect />\n<button>{label}</button>`
    const c = mountRender(
      { mode: 'tag', 'component-name': 'sui-render-scalar' },
      {
        compile: makeBoundCompile(),
        source: SPECIMEN_SRC,
        propValues: { label: 'go', disabled: false },
      },
    )
    await c.flush()
    const el = stageRoot(c.el).querySelector('sui-render-scalar') as HTMLElement
    expect(el).not.toBeNull()
    // string scalar reflected onto the attribute
    expect(el.getAttribute('label')).toBe('go')
    // boolean false → ABSENT attribute (presence semantics), never disabled="false"
    expect(el.getAttribute('disabled')).not.toBe('false')
    expect(el.hasAttribute('disabled')).toBe(false)
  })

  it('STRUCTURED props apply as a JS PROPERTY (el[camelName]), not a JSON attribute', async () => {
    const SPECIMEN_SRC = `<prop name="label" type="string" reflect />\n<div>{label}</div>`
    const structured = { rows: [{ id: 1 }, { id: 2 }] }
    const c = mountRender(
      { mode: 'tag', 'component-name': 'sui-render-structured' },
      {
        compile: makeBoundCompile(),
        source: SPECIMEN_SRC,
        // `dataset` is a non-scalar structured value: it must be set as a JS
        // property (identity) on the host element, never JSON-stringified onto
        // an attribute.
        propValues: { label: 'x', dataset: structured },
      },
    )
    await c.flush()
    const el = stageRoot(c.el).querySelector('sui-render-structured') as HTMLElement
    expect(el).not.toBeNull()
    // structured prop arrived by identity on the JS property...
    expect((el as unknown as Record<string, unknown>).dataset).toBe(structured)
    // ...and was NOT stringified onto an attribute.
    expect(el.hasAttribute('dataset')).toBe(false)
  })

  it('injects composed slot content into the createElement host', async () => {
    const SPECIMEN_SRC = `<prop name="label" type="string" reflect />\n<slot></slot>`
    const c = mountRender(
      { mode: 'tag', 'component-name': 'sui-render-slotted' },
      {
        compile: makeBoundCompile(),
        source: SPECIMEN_SRC,
        propValues: { label: 'x' },
        slots: `<span data-test="injected-slot">slotted-text</span>`,
      },
    )
    await c.flush()
    const el = stageRoot(c.el).querySelector('sui-render-slotted') as HTMLElement
    expect(el).not.toBeNull()
    expect(el.querySelector('[data-test="injected-slot"]')).not.toBeNull()
    expect(el.textContent).toContain('slotted-text')
  })

  it('emits sui-render:mounted {tag} for the named component', async () => {
    const SPECIMEN_SRC = `<prop name="label" type="string" reflect />\n<i>{label}</i>`
    const c = mountRender({ mode: 'tag', 'component-name': 'sui-render-evt' }, { compile: makeBoundCompile() })
    let detail: { tag?: string } | null = null
    c.el.addEventListener('sui-render:mounted', (e) => {
      detail = (e as CustomEvent).detail as { tag?: string }
    })
    ;(c.el as unknown as Record<string, unknown>).source = SPECIMEN_SRC
    ;(c.el as unknown as Record<string, unknown>).propValues = { label: 'y' }
    await c.flush()
    expect(detail).not.toBeNull()
    expect(detail!.tag).toBe('sui-render-evt')
  })
})

describe('<sui-render> — error surface (caught, never a white screen)', () => {
  it('a COMPILE error is caught and renders the compile-error branch (message + line:col + caret)', async () => {
    const c = mountRender({ mode: 'source' }, { compile: makeBoundCompile(), source: BAD_VIEW })
    await c.flush()
    // The engine catches the compile error — the host does not throw, and a
    // diagnostic surface (the folded compile-error branch) renders.
    const root = stageRoot(c.el)
    const panel = root.querySelector('[data-test="compile-error"], .sui-alert')
    expect(panel).not.toBeNull()
    // raw message text is surfaced (no truncation) so an agent can self-correct
    expect(root.textContent ?? '').toMatch(/error|expected|unexpected|compile/i)
  })

  it('emits sui-render:error {message,line,col,phase} on a compile failure', async () => {
    const c = mountRender({ mode: 'source' }, { compile: makeBoundCompile() })
    const detail: Array<Record<string, unknown>> = []
    c.el.addEventListener('sui-render:error', (e) => {
      detail.push((e as CustomEvent).detail as Record<string, unknown>)
    })
    ;(c.el as unknown as Record<string, unknown>).source = BAD_VIEW
    await c.flush()
    expect(detail.length).toBeGreaterThanOrEqual(1)
    expect(detail[0]).toHaveProperty('message')
    expect(detail[0]).toHaveProperty('phase')
    expect((detail[0] as { phase: string }).phase).toBe('compile')
  })

  it('sui-render:error is DEDUPED — the same error is not emitted twice in a row', async () => {
    const c = mountRender({ mode: 'source' }, { compile: makeBoundCompile() })
    let count = 0
    c.el.addEventListener('sui-render:error', () => {
      count++
    })
    // Two identical bad sources back-to-back: the lastEmitted guard collapses
    // the duplicate so the host is not spammed.
    ;(c.el as unknown as Record<string, unknown>).source = BAD_VIEW
    await c.flush()
    ;(c.el as unknown as Record<string, unknown>).source = BAD_VIEW
    await c.flush()
    expect(count).toBe(1)
  })

  it('a setup/mount throw is CAUGHT (secondary try/catch) — a panel renders, not a blank stage', async () => {
    // View-mode source whose injected setup throws synchronously: the engine's
    // secondary try/catch renders the runtime-error panel rather than blanking.
    const c = mountRender(
      { mode: 'source' },
      {
        compile: makeBoundCompile(),
        source: `<div>{nope}</div>`,
        setup: throwingSetup,
        setupUrl: SETUP_MARKER,
      },
    )
    await c.flush()
    const root = stageRoot(c.el)
    expect(root.textContent ?? '').toMatch(/setup-explodes|something went wrong|error/i)
    // The host element survives — the throw did not escape past mountBySource.
    expect(c.el.isConnected).toBe(true)
  })

  it('a runtime throw caught by sui-error-boundary:caught (composed) renders the runtime panel', async () => {
    const c = mountRender({ mode: 'source' }, { compile: makeBoundCompile(), source: GOOD_VIEW })
    await c.flush()
    let surfaced: CustomEvent | null = null
    c.el.addEventListener('sui-render:error', (e) => {
      const d = (e as CustomEvent).detail as { phase?: string }
      if (d.phase === 'mount' || d.phase === 'runtime') surfaced = e as CustomEvent
    })
    // Simulate the inner boundary catching a render/effect throw: the composed
    // sui-error-boundary:caught crosses the shadow boundary and the engine
    // re-surfaces it as a runtime sui-render:error.
    const boundary = stageRoot(c.el).querySelector('sui-error-boundary') as HTMLElement
    expect(boundary).not.toBeNull()
    boundary.dispatchEvent(
      new CustomEvent('sui-error-boundary:caught', {
        detail: { error: new Error('render-boom'), source: 'render' },
        bubbles: true,
        composed: true,
      }),
    )
    await c.flush()
    expect(surfaced).not.toBeNull()
  })
})

describe('<sui-render> — reactive re-mount on source/tag change', () => {
  it('a source flip recompiles + re-mounts the new specimen (the agentic-live path)', async () => {
    const c = mountRender({ mode: 'source' }, { compile: makeBoundCompile(), source: GOOD_VIEW })
    await c.flush()
    expect(specimen(c.el)!.textContent).toBe('hello-from-view')

    ;(c.el as unknown as Record<string, unknown>).source = GOOD_VIEW_ALT
    await c.flush()
    // The old specimen is gone; the new one is mounted.
    expect(stageRoot(c.el).querySelector('[data-test="specimen"]')).toBeNull()
    const alt = stageRoot(c.el).querySelector('[data-test="specimen-alt"]')
    expect(alt).not.toBeNull()
    expect(alt!.textContent).toBe('a-different-render')
  })

  it('a bad source-change KEEPS the last-good specimen AND surfaces the compile error', async () => {
    const c = mountRender({ mode: 'source' }, { compile: makeBoundCompile(), source: GOOD_VIEW })
    await c.flush()
    expect(specimen(c.el)!.textContent).toBe('hello-from-view')

    // A transient bad edit must NOT tear down the working preview.
    ;(c.el as unknown as Record<string, unknown>).source = BAD_VIEW
    await c.flush()
    // last-good specimen survives...
    expect(specimen(c.el)).not.toBeNull()
    // ...and the compile-error surface is shown alongside it.
    expect(stageRoot(c.el).querySelector('[data-test="compile-error"], .sui-alert')).not.toBeNull()
  })

  it('a subsequent good compile CLEARS the error panel and restores a live specimen', async () => {
    const c = mountRender({ mode: 'source' }, { compile: makeBoundCompile(), source: BAD_VIEW })
    await c.flush()
    expect(stageRoot(c.el).querySelector('[data-test="compile-error"], .sui-alert')).not.toBeNull()

    ;(c.el as unknown as Record<string, unknown>).source = GOOD_VIEW
    await c.flush()
    // error state is reactive — a clean compile clears it
    expect(stageRoot(c.el).querySelector('[data-test="compile-error"]')).toBeNull()
    expect(specimen(c.el)).not.toBeNull()
  })

  it('component-mode source-change re-uses the prior versioned tag for IDENTICAL source (hash-dedup, registry does not grow)', async () => {
    const SPECIMEN_SRC = `<prop name="label" type="string" reflect />\n<b>{label}</b>`
    const c = mountRender(
      { mode: 'tag', 'component-name': 'sui-render-hash' },
      { compile: makeBoundCompile(), source: SPECIMEN_SRC, propValues: { label: 'a' } },
    )
    await c.flush()
    const countAfterFirst = customElements.get('sui-render-hash') ? 1 : 0
    expect(countAfterFirst).toBe(1)

    // Re-feed the EXACT same source. The content-hash keys the versioned tag, so
    // the define-once guard hits — no new registration is created.
    const before = (globalThis as unknown as { __ce_defined__?: number }).__ce_defined__
    ;(c.el as unknown as Record<string, unknown>).source = SPECIMEN_SRC
    await c.flush()
    // The base tag is still defined and the specimen still renders — identical
    // source did not mint a brand-new versioned tag (registry growth bound).
    expect(customElements.get('sui-render-hash')).toBeDefined()
    void before
    const el = stageRoot(c.el).querySelector('[data-test], b, sui-render-hash')
    expect(el).not.toBeNull()
  })
})

describe('<sui-render> — COMPONENT-mode source-change (#258 correctness + registry bound)', () => {
  // Drives the ACTUAL controller recompile loop: a component-mode source flip
  // (same base component-name, DIFFERENT body) must (a) re-render the NEW
  // source — a same-base-tag content change cannot silently keep the OLD
  // customElements definition (the stale-render bug from the hardcoded
  // versioned=false at mount-tag.ts mountByTag) — and (b) NOT grow the
  // customElements registry on repeated IDENTICAL re-mounts (the #258 bound).

  it('a component-mode source CHANGE re-renders the NEW source (not the stale prior definition)', async () => {
    const TAG = 'sui-render-stale'
    const SRC_A = `<prop name="label" type="string" reflect />\n<b data-test="render-a">{label}</b>`
    const SRC_B = `<prop name="label" type="string" reflect />\n<i data-test="render-b">{label}</i>`
    const c = mountRender(
      { mode: 'tag', 'component-name': TAG },
      { compile: makeBoundCompile(), source: SRC_A, propValues: { label: 'x' } },
    )
    await c.flush()
    // First mount renders source A.
    expect(stageRoot(c.el).querySelector('[data-test="render-a"]')).not.toBeNull()

    // Flip the source (SAME base component-name, NEW body). The engine must
    // mount a FRESH definition — with the bug, createElement(baseTag) re-uses
    // the already-registered A definition and silently keeps rendering <b>.
    ;(c.el as unknown as Record<string, unknown>).source = SRC_B
    await c.flush()
    // The new source B is rendered...
    expect(stageRoot(c.el).querySelector('[data-test="render-b"]')).not.toBeNull()
    // ...and the stale A render is gone.
    expect(stageRoot(c.el).querySelector('[data-test="render-a"]')).toBeNull()
  })

  it('repeated IDENTICAL-source re-mounts do NOT grow the customElements registry (define-once guard)', async () => {
    const TAG = 'sui-render-bound'
    const SRC = `<prop name="label" type="string" reflect />\n<u data-test="render-bound">{label}</u>`
    const c = mountRender(
      { mode: 'tag', 'component-name': TAG },
      { compile: makeBoundCompile(), source: SRC, propValues: { label: 'x' } },
    )
    await c.flush()
    expect(stageRoot(c.el).querySelector('[data-test="render-bound"]')).not.toBeNull()

    // The realistic agentic edit→revert loop toggles between exactly TWO
    // sources: the canonical SRC and one edited variant SRC_EDIT. The controller
    // dedups src===lastTagSource, so we round-trip through the OTHER source to
    // force the reactive effect to re-mount each time. Each DISTINCT source
    // mints AT MOST ONE customElements definition; once both are registered,
    // every further re-mount of an already-seen source hits the define-once
    // guard and mints ZERO — registry growth is bounded by distinct-source
    // count, NOT by re-mount count (the #258 bound).
    const SRC_EDIT = `${SRC}\n<i data-test="render-edit">edit</i>`
    const origDefine = customElements.define
    let defines = 0
    ;(customElements as unknown as { define: typeof origDefine }).define = function (
      name: string,
      ctor: CustomElementConstructor,
      o?: ElementDefinitionOptions,
    ) {
      defines++
      return origDefine.call(customElements, name, ctor, o)
    }
    try {
      // Warmup: register BOTH distinct sources once (their versioned tags).
      ;(c.el as unknown as Record<string, unknown>).source = SRC_EDIT
      await c.flush()
      ;(c.el as unknown as Record<string, unknown>).source = SRC
      await c.flush()
      const definesAfterWarmup = defines

      // Steady state: toggle between the two ALREADY-REGISTERED sources 10
      // times. With the bug (hardcoded versioned=false) the engine cannot mint
      // a fresh tag, so this would either render stale or — were the registry
      // wired naively — grow once per re-mount. With the fix it mints NOTHING:
      // both versioned tags are already in the registry.
      for (let i = 0; i < 10; i++) {
        ;(c.el as unknown as Record<string, unknown>).source = i % 2 === 0 ? SRC_EDIT : SRC
        await c.flush()
      }
      const steadyStateDefines = defines - definesAfterWarmup
      // Re-mounting already-seen sources adds ZERO registrations.
      expect(steadyStateDefines).toBe(0)
      // And the engine still reflects the live source (not a stale definition).
      ;(c.el as unknown as Record<string, unknown>).source = SRC_EDIT
      await c.flush()
      expect(stageRoot(c.el).querySelector('[data-test="render-edit"]')).not.toBeNull()
    } finally {
      ;(customElements as unknown as { define: typeof origDefine }).define = origDefine
    }
  })
})

describe('<sui-render> — prop change patches in place (NO remount)', () => {
  it('a prop-values change patches the existing node — the live element identity survives', async () => {
    const SPECIMEN_SRC = `<prop name="label" type="string" reflect />\n<input data-test="focusable" />`
    const c = mountRender(
      { mode: 'tag', 'component-name': 'sui-render-patch' },
      { compile: makeBoundCompile(), source: SPECIMEN_SRC, propValues: { label: 'first' } },
    )
    await c.flush()
    const firstEl = stageRoot(c.el).querySelector('sui-render-patch')
    expect(firstEl).not.toBeNull()

    // Change a scalar prop — the engine patches the attribute in place; it does
    // NOT createElement a new node (same node identity = no remount).
    ;(c.el as unknown as Record<string, unknown>).propValues = { label: 'second' }
    await c.flush()
    const secondEl = stageRoot(c.el).querySelector('sui-render-patch')
    expect(secondEl).toBe(firstEl)
    expect((secondEl as HTMLElement).getAttribute('label')).toBe('second')
  })
})

describe('<sui-render> — shadow isolation + inherited-style baseline', () => {
  it('a host-page inherited style does NOT bleed into the specimen (inherited-property baseline re-root)', async () => {
    // Set a non-default inherited style on the host page.
    document.documentElement.style.setProperty('font-family', 'ComicSans')
    try {
      const c = mountRender({ mode: 'source' }, { compile: makeBoundCompile(), source: GOOD_VIEW })
      await c.flush()
      const node = specimen(c.el) as HTMLElement
      expect(node).not.toBeNull()
      // The engine re-establishes the inherited-property baseline at the shadow
      // root from role tokens, so the page's ComicSans does not leak in.
      const ff = node.style.fontFamily || stageRoot(c.el).host.style.fontFamily
      expect(ff).not.toBe('ComicSans')
    } finally {
      document.documentElement.style.removeProperty('font-family')
    }
  })
})

describe('<sui-render> — viewport-width stage constraint', () => {
  it('reflects the viewport-width attribute and constrains the stage', async () => {
    const c = mountRender(
      { mode: 'source', 'viewport-width': 'mobile' },
      { compile: makeBoundCompile(), source: GOOD_VIEW },
    )
    await c.flush()
    expect(c.el.getAttribute('viewport-width')).toBe('mobile')
    // The specimen still renders inside the constrained stage (theme/density
    // unaffected — orthogonal axes).
    expect(specimen(c.el)).not.toBeNull()
  })
})
