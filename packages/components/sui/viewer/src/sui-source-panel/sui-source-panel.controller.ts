// <sui-source-panel> controller — source + recomputed snippet + the one copy
// affordance (build-spec §The viewing components #5).
//
// Controller-purity: this file does NO view-markup DOM construction. It owns
// LIFECYCLE only — wiring host `prop:*` into signals, deriving the snippet,
// rendering source/snippet text via `bindHtml` (the runtime's morph-backed
// HTML binding — never a raw markup assignment), and the copy interaction.

import { controller, signal, derived, bindHtml, ref } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-source-panel.sui'
import { buildSnippet, copyText, type CemAttr } from './snippet'

interface EmitHost extends Element {
  emit?: <T>(name: string, detail?: T) => void
  componentName?: string
}

/** Escape raw source text so `bindHtml` renders it verbatim as code text (the
 *  `<` / `&` show literally, not parsed as markup). */
function escapeText(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const sourceEl = ref<HTMLElement>(null)
    const snippetEl = ref<HTMLElement>(null)
    const copied = signal(false)

    // --- structured host props, as signals (identity/snapshot; never a
    //     stringified attr). Wired to the host element in `wireHost`, below. ---
    const source = signal<string>('')
    const cem = signal<CemAttr[] | null>(null)
    const values = signal<Record<string, unknown>>({})
    const slotContent = signal<string | null>(null)
    // The host element. The runtime hands it to `mount()`, NOT the scope factory
    // ("SUI names no host" — manifest-only), so `componentName` and the copy
    // emit read it through this signal, set once in `wireHost`.
    const hostEl = signal<EmitHost | null>(null)

    // --- the recomputed snippet (default-diff; degraded when CEM absent) ---
    const snippet = derived(() => {
      const tag = hostEl()?.componentName || 'sui-component'
      return buildSnippet(tag, values(), cem() ?? undefined, slotContent() ?? undefined)
    })

    // --- copy: the ONE clipboard path, one event per press (the click handler
    //     fires once per user press — a single call site is the dedup) ---
    let copiedTimer: ReturnType<typeof setTimeout> | null = null
    const copy = (): void => {
      const text = snippet()
      void copyText(text).then((ok) => {
        if (!ok) return
        // Transient "Copied" feedback. A repeated press resets the timer rather
        // than stacking — one feedback window per click.
        copied.set(true)
        if (copiedTimer) clearTimeout(copiedTimer)
        copiedTimer = setTimeout(() => {
          copied.set(false)
          copiedTimer = null
        }, 1400)
      })
      hostEl()?.emit?.('sui-source-copy', { text })
    }
    const clearCopyTimer = (): void => {
      if (copiedTimer) clearTimeout(copiedTimer)
      copiedTimer = null
    }

    // --- DEFERRED host wiring. The host element only exists at mount(); this
    //     closure carries the signal wiring there. Installs `prop:*` accessors
    //     (set `el.source`/`props`/`value`/`slotContent` → signals) + hands the
    //     element to `hostEl`. ---
    const wireHost = (host: EmitHost): void => {
      hostEl.set(() => host)
      const defineHostProp = <T>(
        name: string,
        sig: { (): T; set: (u: (p: T) => T) => void },
        coerce: (v: unknown) => T,
      ): void => {
        const existing = (host as unknown as Record<string, unknown>)[name]
        if (existing !== undefined) sig.set(() => coerce(existing))
        Object.defineProperty(host, name, {
          configurable: true,
          get: () => sig(),
          set: (v: unknown) => sig.set(() => coerce(v)),
        })
      }
      defineHostProp('source', source, (v) => (typeof v === 'string' ? v : ''))
      defineHostProp('props', cem, (v) => (Array.isArray(v) ? (v as CemAttr[]) : null))
      defineHostProp('value', values, (v) => (v && typeof v === 'object' ? (v as Record<string, unknown>) : {}))
      defineHostProp('slotContent', slotContent, (v) => (typeof v === 'string' ? v : null))
    }

    return { sourceEl, snippetEl, copied, source, snippet, copy, clearCopyTimer, wireHost }
  },

  mount(state, host) {
    // The runtime hands the real host element here. Wire the structured props
    // now that it exists, then render source + snippet text via bindHtml.
    ;(state.wireHost as (h: EmitHost) => void)(host as EmitHost)

    const srcEl = state.$signals.sourceEl() as HTMLElement | null
    const snipEl = state.$signals.snippetEl() as HTMLElement | null
    const stopSource = srcEl ? bindHtml(srcEl, () => escapeText(state.$signals.source())) : () => {}
    const stopSnippet = snipEl ? bindHtml(snipEl, () => escapeText(state.$signals.snippet())) : () => {}

    return () => {
      stopSource()
      stopSnippet()
      ;(state.clearCopyTimer as () => void)()
    }
  },
})
