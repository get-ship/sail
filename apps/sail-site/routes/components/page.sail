<!--
  /components — component browser.

  Two layers:
    1. FEATURED LIVE demos — a fixed, small set of real L1 hosts (button,
       inputs, badge, alert, …). Enough to prove the language without
       freezing the tab.
    2. FULL CATALOG — every registry entry as a card. Pack tiles render
       their real <sail-*> preview when that pack is selected; language
       entries and unselected packs show a link placeholder.

  The OOM that originally forced static pack tiles was traced to batched
  style reflections (fixed). Mount gating remains: packSel defaults to
  `lang`, so pack CEs only mount when the user picks a pack.
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
    const packSel = signal<string>('lang')
    const catSel = signal<string>('all')
    const pickPack = (id: string): void => packSel.set(id)
    const pickCat = (id: string): void => catSel.set(id)

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
      return n > 0 ? `${n} in catalog` : ''
    })
    const none = derived(() => visibleList().length === 0)

    // Featured live demos — small fixed set.
    const name = signal('')
    const agreed = signal(true)
    const notify = signal(true)
    const volume = signal(60)

    return {
      query, packSel, catSel, pickPack, pickCat,
      packs, cats, visibleList, countLabel, none,
      name, agreed, notify, volume,
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
  const sectionTitle = style.fontSize(sm).fontWeight(semibold).mt(2)
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
      Live demos below — real language elements. The full catalog under that
      links to each detail page (also live). Packs open on their own routes so
      this page stays fast.
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
        Language tiles live here. Pack previews mount only when you select a
        pack — the default stays light so the index always loads fast.
      </p>
    </column>

    <column gap="4" sx={gridCol}>
      <!-- ══ Featured live demos ═══════════════════════════════════════ -->
      <column gap="2">
        <p sx={sectionTitle}>Live — Component primitives</p>
        <grid cols="4" gap="3">
          <card padding="none" data-tile="button" data-live sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <row gap="2" align="center">
                  <button variant="solid" tone="primary" motion:press={{ scale: 'press' }}>Solid</button>
                  <button variant="outline" tone="primary">Outline</button>
                </row>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/button" sx={tileName}>button</a>
                <span sx={cat}>inputs · live</span>
              </column>
            </column>
          </card>

          <card padding="none" data-tile="input" data-live sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <input placeholder="Type — it's real" bind:value={name} />
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/input" sx={tileName}>input</a>
                <span sx={cat}>inputs · live</span>
              </column>
            </column>
          </card>

          <card padding="none" data-tile="checkbox" data-live sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <label><input type="checkbox" bind:checked={agreed} /> I agree</label>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/checkbox" sx={tileName}>checkbox</a>
                <span sx={cat}>inputs · live</span>
              </column>
            </column>
          </card>

          <card padding="none" data-tile="switch" data-live sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <row gap="2" align="center">
                  <switch bind:checked={notify}></switch>
                  <p sx={mutedXs}>{notify() ? 'notifying' : 'muted'}</p>
                </row>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/switch" sx={tileName}>switch</a>
                <span sx={cat}>inputs · live</span>
              </column>
            </column>
          </card>

          <card padding="none" data-tile="slider" data-live sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <row gap="2" align="center" sx={style.w(full)}>
                  <slider bind:value={volume} tone="primary" sx={style.grow(1)}></slider>
                  <span sx={cat}>{volume()}</span>
                </row>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/slider" sx={tileName}>slider</a>
                <span sx={cat}>inputs · live</span>
              </column>
            </column>
          </card>

          <card padding="none" data-tile="badge" data-live sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <row gap="2" align="center">
                  <badge tone="primary">Primary</badge>
                  <badge tone="success" variant="soft">OK</badge>
                  <badge tone="destructive" variant="outline">Hot</badge>
                </row>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/badge" sx={tileName}>badge</a>
                <span sx={cat}>display · live</span>
              </column>
            </column>
          </card>

          <card padding="none" data-tile="chip" data-live sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <row gap="2">
                  <chip>Alpha</chip>
                  <chip tone="primary">Beta</chip>
                </row>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/chip" sx={tileName}>chip</a>
                <span sx={cat}>display · live</span>
              </column>
            </column>
          </card>

          <card padding="none" data-tile="alert" data-live sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <alert tone="success"><strong>Saved.</strong> Changes are live.</alert>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/alert" sx={tileName}>alert</a>
                <span sx={cat}>feedback · live</span>
              </column>
            </column>
          </card>

          <card padding="none" data-tile="spinner" data-live sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <spinner></spinner>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/spinner" sx={tileName}>spinner</a>
                <span sx={cat}>feedback · live</span>
              </column>
            </column>
          </card>

          <card padding="none" data-tile="progress" data-live sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <progress value="0.62" sx={style.w(full)}></progress>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/progress" sx={tileName}>progress</a>
                <span sx={cat}>feedback · live</span>
              </column>
            </column>
          </card>

          <card padding="none" data-tile="card" data-live sx={tile}>
            <column gap="0" sx={tileCol}>
              <column data-preview sx={preview}>
                <card interactive sx={style.pad(3)}>
                  <strong>A surface</strong>
                  <p sx={mutedXs}>hover lift is one attribute</p>
                </card>
              </column>
              <column gap="1" sx={tileMeta}>
                <a href="/components/card" sx={tileName}>card</a>
                <span sx={cat}>display · live</span>
              </column>
            </column>
          </card>

          <card padding="none" data-tile="icon" data-live sx={tile}>
            <column gap="0" sx={tileCol}>
              <center data-preview sx={preview}>
                <row gap="3" align="center">
                  <icon name="check"></icon>
                  <icon name="search"></icon>
                  <icon name="settings"></icon>
                </row>
              </center>
              <column gap="1" sx={tileMeta}>
                <a href="/components/icon" sx={tileName}>icon</a>
                <span sx={cat}>display · live</span>
              </column>
            </column>
          </card>
        </grid>
      </column>

      <!-- ══ Catalog ═══════════════════════════════════════════════════ -->
      <column gap="2">
        <row gap="2" align="center" justify="between">
          <p sx={sectionTitle}>Catalog</p>
          <row gap="2">
            {#for c of cats() key={c}}
              <button size="sm" tone="primary" variant={catSel() === c ? 'solid' : 'ghost'} on:click={() => pickCat(c)}>{c}</button>
            {/for}
          </row>
        </row>
        <row gap="2" align="center" justify="between">
          {#if countLabel()}
            <p sx={count}>{countLabel()}</p>
          {/if}
          <input placeholder="Search — try 'tab' or 'overlay'" bind:value={query} sx={search} />
        </row>

        <grid cols="4" gap="3">
          {#for e of visibleList() key={e.key}}
            <card padding="none" data-tile={e.tag} sx={tile}>
              <column gap="0" sx={tileCol}>
                <center data-preview sx={preview}>
                  {#if e.pack !== 'lang' && (packSel() === 'all' || packSel() === e.pack)}
                    {#if e.tag === 'sail-sparkline'}
                      <sail-sparkline data="2,5,3,8,4,6,7" width="120" height="40"></sail-sparkline>
                    {:else if e.tag === 'sail-area-chart'}
                      <sail-area-chart series='[{"name":"a","data":[2,4,3,5]}]' labels="Q1,Q2,Q3,Q4" width="160" height="60"></sail-area-chart>
                    {:else if e.tag === 'sail-bar-chart'}
                      <sail-bar-chart data="2,5,3,8,4" labels="A,B,C,D,E" width="160" height="60"></sail-bar-chart>
                    {:else if e.tag === 'sail-donut'}
                      <sail-donut data='[{"label":"a","value":40},{"label":"b","value":30},{"label":"c","value":30}]' diameter="80" thickness="10"></sail-donut>
                    {:else if e.tag === 'sail-gauge'}
                      <sail-gauge value="72" diameter="80" thickness="10"></sail-gauge>
                    {:else if e.tag === 'sail-heatmap'}
                      <sail-heatmap rows="3" cols="5" values="1,2,3,4,3,4,5,6,5,4,3,2,1,2,3" width="160" height="60"></sail-heatmap>
                    {:else if e.tag === 'sail-line-chart'}
                      <sail-line-chart series='[{"name":"a","data":[2,4,3,5]}]' labels="Q1,Q2,Q3,Q4" width="160" height="60"></sail-line-chart>
                    {:else if e.tag === 'sail-media-image'}
                      <sail-media-image src="/sail-oss/sail-mark.svg" alt="Sail mark"></sail-media-image>
                    {:else if e.tag === 'sail-media-av'}
                      <sail-media-av></sail-media-av>
                    {:else if e.tag === 'sail-code-editor'}
                      <sail-code-editor value="const greeting = 'hi';" language="js" line-numbers size="sm"></sail-code-editor>
                    {:else if e.tag === 'sail-mermaid'}
                      <sail-mermaid value="graph TD; A[Idea]-->B[Build]; B-->C[Ship]"></sail-mermaid>
                    {:else if e.tag === 'sail-message'}
                      <sail-message role="user" state="user">Hello — the live chat message primitive.</sail-message>
                    {:else if e.tag === 'sail-citation'}
                      <sail-citation source="docs.sail.dev" href="https://docs.sail.dev">A citation from the docs</sail-citation>
                    {:else if e.tag === 'sail-composer'}
                      <sail-composer placeholder="Ask anything…"></sail-composer>
                    {:else if e.tag === 'sail-thinking-dots'}
                      <sail-thinking-dots></sail-thinking-dots>
                    {:else if e.tag === 'sail-tool-call'}
                      <sail-tool-call name="search" status="running">query: search</sail-tool-call>
                    {:else}
                      <p sx={mutedXs}>Open for full live demo</p>
                    {/if}
                  {:else}
                    <p sx={mutedXs}>Open for full live demo</p>
                  {/if}
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
    </column>
  </row>

  <row sx={style.mt(8).padT(4).borderTWidth(1).borderT(border)} gap="2">
    <a variant="default" href="/sail/overview">← Introduction</a>
    <a variant="solid" tone="primary" href="/sail/language/layout">Layout →</a>
  </row>
</column>
