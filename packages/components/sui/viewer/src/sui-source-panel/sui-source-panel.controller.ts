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
  // The scope factory receives `host` (the sui-error-boundary pattern), so the
  // structured prop signals, the derived snippet, and the `copy` handler bound
  // by the template's `on:click={copy}` all live here as known scope members.
  scope: (_ctx: ViewScope<Scope>, host?: Element) => {
    const sourceEl = ref<HTMLElement>(null)
    const snippetEl = ref<HTMLElement>(null)
    const copied = signal(false)

    // --- host prop:* → signals (identity/snapshot; never a stringified attr) ---
    const source = signal<string>('')
    const cem = signal<CemAttr[] | null>(null)
    const values = signal<Record<string, unknown>>({})
    const slotContent = signal<string | null>(null)

    const defineHostProp = <T>(
      name: string,
      sig: { (): T; set: (u: (p: T) => T) => void },
      coerce: (v: unknown) => T,
    ): void => {
      if (!host) return
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

    // --- the recomputed snippet (default-diff; degraded when CEM absent) ---
    const snippet = derived(() => {
      const tag = (host as EmitHost | undefined)?.componentName || 'sui-component'
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
      ;(host as EmitHost | undefined)?.emit?.('sui-source-copy', { text })
    }
    const clearCopyTimer = (): void => {
      if (copiedTimer) clearTimeout(copiedTimer)
      copiedTimer = null
    }

    return { sourceEl, snippetEl, copied, source, snippet, copy, clearCopyTimer }
  },

  mount(state) {
    // Render source + snippet text via bindHtml (morph-backed, not a raw assign).
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
