// Gallery controller — the LIVE primitive viewer. The sidebar selects a
// primitive; the stage is a <sui-component-viewer> fed the selected `.sui`
// source + CEM attributes, compiling in-browser via the wasm seam (prop:compile).
import { controller, derived, signal, type ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

interface GalleryEntry {
  tag: string
  title: string
  source: string
  cemAttrs: unknown[] | null
}
type CompileFn = (source: string, opts?: unknown) => unknown

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const g = ctx.clients.gallery as {
      entries: GalleryEntry[]
      selected: { (): string; set: (u: (p: string) => string) => void }
      compile: { (): CompileFn | null }
      select: (tag: string) => void
      entryFor: (tag: string) => GalleryEntry | undefined
    }

    const selectedTitle = derived(() => g.entryFor(g.selected())?.title ?? '')
    const selectedSource = derived(() => g.entryFor(g.selected())?.source ?? '')
    const selectedCem = derived(() => g.entryFor(g.selected())?.cemAttrs ?? null)
    // The catalog list is static, but {#for} iterates a reactive source.
    const entries = signal(g.entries)

    return {
      entries,
      selected: g.selected,
      select: g.select,
      selectedTitle,
      selectedSource,
      selectedCem,
      compile: g.compile,
      count: g.entries.length,
    }
  },
})
