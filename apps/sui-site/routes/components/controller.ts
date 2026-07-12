// /components — the browser. Three filters compose over one catalog (the
// full 48-element L1 surface plus the fx actions): the pack rail, the
// category chips, and the search box (the runtime's filteredList
// composition). Their intersection is a set of visible tile keys; every
// tile in page.sui gates itself on `show.<key>()`, so the markup for each
// live component stays hand-written while the filtering stays data-driven.
//
// Every rendered tile is the REAL component. The .sui packs (charts, media,
// canvas, chat, mermaid, code-editor) still carry raw <style> blocks, which
// THIS compiler's floor rejects — pulling them into the graph kills the
// build (the exact coupling src/register-all.ts removed). So they are
// `gated` in the rail rather than faked in the grid: selecting one explains
// the gap. @ship/sui-fx is pure TS and runs live.

import { controller, signal, derived, filteredList, type ViewScope } from '@ship/sui-runtime'
import { spotlight, dotGrid } from '@ship/sui-fx'
import type { Scope } from './page.sui'
import { CATALOG, PACKS, CATEGORIES, packCount, type CatalogEntry, type Pack } from './registry'

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const { query, filtered } = filteredList(
      () => CATALOG,
      (c: CatalogEntry, q: string) => (c.tag + ' ' + c.cat + ' ' + c.pack).toLowerCase().includes(q),
    )
    const packSel = signal<string>('all')
    const catSel = signal<string>('all')
    const pickPack = (id: string): void => packSel.set(id)
    const pickCat = (id: string): void => catSel.set(id)

    const visible = derived(() => {
      const p = packSel()
      const c = catSel()
      const pass = new Set<string>()
      for (const e of filtered()) {
        if ((p === 'all' || e.pack === p) && (c === 'all' || e.cat === c)) pass.add(e.key)
      }
      return pass
    })

    // One zero-arg visibility read per tile — the only call shape a
    // renderable expression allows (`show.badge()`, never `isShown('badge')`).
    const show = Object.fromEntries(
      CATALOG.map((e) => [e.key, () => visible().has(e.key)]),
    ) as Record<string, () => boolean>

    const packs = () => PACKS.map((p) => ({ ...p, count: packCount(p.id) }))
    const cats = () => CATEGORIES.map((c) => c as string)
    // Split the count by pack so the headline number is the L1 surface
    // ("48 elements") and the fx actions are counted apart, never inflating it.
    const countLabel = derived(() => {
      const v = visible()
      let lang = 0
      let fx = 0
      for (const e of CATALOG) {
        if (!v.has(e.key)) continue
        if (e.pack === 'fx') fx += 1
        else lang += 1
      }
      const parts: string[] = []
      if (lang > 0) parts.push(`${lang} element${lang === 1 ? '' : 's'}`)
      if (fx > 0) parts.push(`${fx} fx action${fx === 1 ? '' : 's'}`)
      return parts.length > 0 ? `${parts.join(' · ')} — all live` : ''
    })

    // The selected pack, when it is floor-gated — drives the honest
    // explanation card instead of a grid of fakes.
    const gatedPack = derived<Pack | null>(() => {
      const p = PACKS.find((x) => x.id === packSel())
      return p?.gated ? p : null
    })
    const gatedSel = derived(() => gatedPack() !== null)
    const gatedLabel = () => gatedPack()?.label ?? ''
    const gatedSub = () => gatedPack()?.sub ?? ''
    const gatedWaiting = () => gatedPack()?.waiting ?? 0
    const none = derived(() => visible().size === 0 && !gatedSel())

    // Demo state the interactive tiles mutate — real signals, not props.
    const name = signal('')
    const agreed = signal(true)
    const notify = signal(true)
    const volume = signal(60)

    return {
      query, packSel, catSel, pickPack, pickCat,
      packs, cats, show, countLabel, none,
      gatedSel, gatedLabel, gatedSub, gatedWaiting,
      name, agreed, notify, volume,
      spotlight, dotGrid,
    }
  },
})
