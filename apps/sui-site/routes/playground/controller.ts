// /playground — the interactive component storybook. Registers the viewer +
// primitive packs, boots the in-browser compile seam, and feeds the selected
// primitive's source / CEM / example slots into <sui-component-viewer>, which
// renders it live (tag mode) with the CEM-driven knobs.
import '@ship/sui-viewer' // side-effect: defines sui-component-viewer + children
import '@ship/sui-primitives' // side-effect: defines every previewed <sui-*>
import { controller, signal, derived, type ViewScope } from '@ship/sui-runtime'
import { ensureWasm, makeCompile, type CompileFn } from '@ship/sui-viewer'
import type { Scope } from './page.sui'
import manifest from '../../src/playground-manifest.json'
import { EXAMPLES } from '../../src/playground-examples'

/** Flatten a full CEM module to the custom element's `attributes[]` (what the
 *  viewer's knobs read). */
function cemAttrs(cem: unknown): unknown[] | null {
  const mods = (cem as { modules?: unknown[] } | null)?.modules ?? []
  for (const m of mods as Array<{ declarations?: unknown[] }>) {
    for (const d of (m.declarations ?? []) as Array<{
      customElement?: boolean
      attributes?: unknown[]
    }>) {
      if (d.customElement && Array.isArray(d.attributes)) return d.attributes
    }
  }
  return null
}

interface Entry {
  tag: string
  title: string
  source: string
  cemAttrs: unknown[] | null
  slots: string | null
}

const entries: Entry[] = (manifest as unknown as Omit<Entry, 'cemAttrs' | 'slots'>[]).map(
  (e) => ({ ...e, cemAttrs: cemAttrs((e as unknown as { cem: unknown }).cem), slots: EXAMPLES[e.tag] ?? null }),
)

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const sel = signal<string>(entries[0]?.tag ?? '')
    const compile = signal<CompileFn | null>(null)
    ensureWasm('/sui_compiler_bg.wasm')
      .then(() => compile.set(() => makeCompile()))
      .catch((err) => console.error('[sui-site] wasm compile seam failed:', err))

    const entryFor = (tag: string): Entry | undefined => entries.find((e) => e.tag === tag)

    return {
      entries: signal(entries),
      selected: sel,
      select: (tag: string): void => sel.set(() => tag),
      selectedTitle: derived(() => entryFor(sel())?.title ?? ''),
      selectedSource: derived(() => entryFor(sel())?.source ?? ''),
      selectedCem: derived(() => entryFor(sel())?.cemAttrs ?? null),
      selectedSlots: derived(() => entryFor(sel())?.slots ?? null),
      compile,
      count: entries.length,
    }
  },
})
