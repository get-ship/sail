<!--
  /components — the component browser.

  Pack rail + category chips + search filter one catalog (registry.ts).
  Each tile is a catalog card that links to /components/<slug>, where the
  REAL component mounts on its detail page.

  Why the index is NOT a wall of live demos: after the Style cutover each
  L1 host carries a large class list + actions; mounting ~48 live tiles
  (and pack CEs like CodeMirror/Mermaid) freezes/OOMs the tab. Deck also
  eagerly imports every custom-element tag that appears in this route's
  template into the page graph — so pack tags here pulled multi‑MB modules
  even when filtered out of the DOM. Index = catalog; detail = live.
-->

<controller>
import { controller, signal, derived, filteredList, type ControllerScope } from '@ship/sail-runtime'
import type { Scope } from './page.sail'
import { CATALOG, PACKS, CATEGORIES, packCount, type CatalogEntry } from './registry'

export default controller((_ctx: ControllerScope<Scope>) => {
    const { query, filtered } = filteredList(
      () => CATALOG,
      (c: CatalogEntry, q: string) => (c.tag + ' ' + c.cat + ' ' + c.pack).toLowerCase().includes(q),
    )
    // Default to L1 — packs are still filterable; tiles are always link cards.
    const packSel = signal<string>('lang')
    const catSel = signal<string>('all')
    const pickPack = (id: string): void => packSel.set(id)
    const pickCat = (id: string): void => catSel.set(id)

    // Not cataloged with working routes yet (banner history / bundler gaps).
    const SKIP_TAGS = new Set([
      'sail-canvas', 'sail-board', 'sail-draw', 'sail-graph-canvas',
      'sail-annotation-layer', 'sail-annotation-list', 'sail-annotation-marker',
      'sail-message-list', 'sail-media-csv',
    ])

    const packBadgeTone = (pack: string): string => {
      switch (pack) {
        case 'lang': return 'primary'
        case 'charts': return 'accent'
        case 'media': return 'info'
        case 'chat': return 'primary'
        case 'mermaid': return 'success'
        case 'code-editor': return 'muted'
        default: return 'muted'
      }
    }
    const packBadgeLabel = (pack: string): string => {
      switch (pack) {
        case 'lang': return 'language'
        case 'code-editor': return 'code-editor'
        default: return pack
      }
    }

    // Flatten badge fields on each row — render expressions may only call
    // zero-arg signal reads (no packBadgeTone(e.pack) in the template).
    const visibleList = derived(() => {
      const p = packSel()
      const c = catSel()
      const out: Array<CatalogEntry & { badgeTone: string; badgeLabel: string }> = []
      for (const e of filtered()) {
        if (SKIP_TAGS.has(e.tag)) continue
        if ((p === 'all' || e.pack === p) && (c === 'all' || e.cat === c)) {
          out.push({
            ...e,
            badgeTone: packBadgeTone(e.pack),
            badgeLabel: packBadgeLabel(e.pack),
          })
        }
      }
      return out
    })

    const packs = () => PACKS.map((p) => ({ ...p, count: packCount(p.id) }))
    const cats = () => CATEGORIES.map((c) => c as string)
    const countLabel = derived(() => {
      const n = visibleList().length
      return n > 0 ? `${n} element${n === 1 ? '' : 's'} — open for live demo` : ''
    })
    const none = derived(() => visibleList().length === 0)

    return {
      query, packSel, catSel, pickPack, pickCat,
      packs, cats, visibleList, countLabel, none,
    }
})
</controller>

<scope>
  const page = style.w(full).minW(0).overflowX(hidden)
  const tile = style.overflow(hidden)
  const tileCol = style.h(full)
  const preview = style.pad(3).minH(9).grow(1).borderBWidth(1).borderB(border-subtle).bg(surface-0)
  const tileMeta = style.padX(3).padY(2)
  const tileName = style.fontSize(sm).fontWeight(semibold).fontFamily(mono)
  const cat = style.fontSize(xs).fg(muted-foreground).fontFamily(mono)
  const mutedSm = style.fontSize(sm).fg(muted-foreground)
  const eyebrow = style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(accent).mb(2)
  const gridCol = style.grow(1).minW(0).basis(0)
  const rail = style.w(panel-min-w-md).shrink(0)
  const packBtn = style.radius(md).cursor(pointer).borderWidth(1).border(border).pad(0).overflow(hidden).bg(transparent).textAlign(start).when({
    hover: style.bg(hover-bg),
  })
  const packLabel = style.fontSize(sm).grow(1)
  const packCount = style.fontSize(xs).fg(primary).fontFamily(mono)
  const railNote = style.fontSize(xs).fg(muted-foreground).leading(relaxed).padT(2)
  const search = style.maxW(panel-max-w-sm)
  const mutedXs = style.fontSize(xs).fg(muted-foreground)
  const count = style.fontSize(sm).fg(muted-foreground).fontFamily(mono)
</scope>

<column gap="4" sx={page}>
  <column gap="1">
    <p sx={eyebrow}>Components</p>
    <p sx={mutedSm}>
      Catalog of the language surface and packs. Open a tile for the live demo —
      the index stays light so it always loads.
    </p>
  </column>

  <row align="stretch" gap="4">
    <column gap="1" sx={rail}>
      <p sx={eyebrow}>Packs</p>
      {#for p of packs() key={p.id}}
        <button sx={packBtn} on:click={() => pickPack(p.id)} aria-pressed={packSel() === p.id ? 'true' : 'false'}>
          <column gap="0" sx={tileMeta}>
            <row gap="2" align="center">
              <strong sx={packLabel}>{p.label}</strong>
              <span sx={packCount}>{p.count}</span>
            </row>
            {#if p.sub}
              <p sx={cat}>{p.sub}</p>
            {/if}
          </column>
        </button>
      {/for}
      <p sx={railNote}>
        Packs read the same tokens as the language — install one and it
        already matches your theme. Live demos open on each detail page.
      </p>
    </column>

    <column gap="3" sx={gridCol}>
      <row gap="2" align="center" justify="between">
        <row gap="2">
          {#for c of cats() key={c}}
            <button size="sm" tone="primary" variant={catSel() === c ? 'solid' : 'ghost'} on:click={() => pickCat(c)}>{c}</button>
          {/for}
        </row>
        <input placeholder="Search — try 'tab' or 'overlay'" bind:value={query} sx={search} />
      </row>

      {#if countLabel()}
        <p sx={count}>{countLabel()}</p>
      {/if}

      <grid cols="4" gap="3">
        {#for e of visibleList() key={e.key}}
          <card padding="none" data-tile={e.tag} sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <p sx={mutedXs}>Open for live demo</p>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href={`/components/${e.slug}`} sx={tileName}>{e.tag}</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone={e.badgeTone} variant="soft">{e.badgeLabel}</badge>
                  <span sx={cat}>{e.cat}</span>
                </row>
              </column>
            </column>
          </card>
        {/for}
      </grid>

      {#if none()}
        <empty-state>
          <strong slot="title">Nothing matches</strong>
          Clear a filter or two — or search for less.
        </empty-state>
      {/if}
    </column>
  </row>

  <row sx={style.mt(8).padT(4).borderTWidth(1).borderT(border)} gap="2">
    <a variant="default" href="/sail/overview">← Introduction</a>
    <a variant="solid" tone="primary" href="/sail/language/layout">Layout →</a>
  </row>
</column>
