<!--
  /components — component browser.

  One catalog grid: every registry entry shows a LIVE preview when its pack
  is in scope (lang always; pack CEs only when that pack is selected, so
  CodeMirror/Mermaid stay off the default path). No dual "featured demo"
  wall and no "Open for full live demo" placeholders.
-->

<controller>
import { filteredList, type ControllerScope } from '@ship/sail-runtime'
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
        case 'lang': return 'primitives'
        case 'code-editor': return 'code-editor'
        default: return pack
      }
    }

    // Live mount only when the tile's pack is selected (or All packs).
    // Lang (component primitives) always mounts — reflections are batched.
    const livePack = (pack: string): boolean => {
      const p = packSel()
      return pack === 'lang' || p === 'all' || p === pack
    }

    const visibleList = derived(() => {
      const p = packSel()
      const c = catSel()
      const out: Array<CatalogEntry & { badgeTone: string; badgeLabel: string; live: boolean }> = []
      for (const e of filtered()) {
        if (SKIP_TAGS.has(e.tag)) continue
        if ((p === 'all' || e.pack === p) && (c === 'all' || e.cat === c)) {
          out.push({
            ...e,
            badgeTone: packBadgeTone(e.pack),
            badgeLabel: packBadgeLabel(e.pack),
            live: livePack(e.pack),
          })
        }
      }
      return out
    })

    const packs = () => PACKS.map((p) => ({ ...p, count: packCount(p.id) }))
    const cats = () => CATEGORIES.map((c) => c as string)
    const countLabel = derived(() => {
      const n = visibleList().length
      return n > 0 ? `${n} components` : ''
    })
    const none = derived(() => visibleList().length === 0)

    const name = signal('')
    const agreed = signal(true)
    const notify = signal(true)
    const volume = signal(60)
    const team = signal('Design')
    const kept = signal(true)
    const dropChip = (): void => kept.set(false)

    return {
      query, packSel, catSel, pickPack, pickCat,
      packs, cats, visibleList, countLabel, none,
      name, agreed, notify, volume, team, kept, dropChip,
    }
})
</controller>

<scope>
  const page = style.w(full).minW(0).maxW(breakpoint-desktop).mx(auto)
  const tile = style.overflow(hidden)
  const tileCol = style.h(full)
  // Flex via <row> (not <center>/grid place-items:center): auto grid tracks
  // collapse to max-content, so width:100% bars (progress/meter) measure 0.
  // Flex parent has a definite tile width, so w(full) children get a real track.
  const preview = style.pad(4).minH(11).grow(1).w(full).borderBWidth(1).borderB(border-subtle).bg(surface-0)
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
  const dormant = style.fontSize(xs).fg(muted-foreground).fontFamily(mono).textAlign(center)
</scope>

<column gap="5" sx={page}>
  <column gap="1">
    <p sx={eyebrow}>Components</p>
    <p sx={mutedSm}>
      Live previews — real elements. Click a name for the detail page (API, variants, usage).
      Heavy packs (charts, editor, mermaid) mount only when you select them.
    </p>
  </column>

  <row align="stretch" gap="5">
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
        Component primitives always live. Select Charts, Media, Chat, Mermaid, or Code editor to mount those packs.
      </p>
    </column>

    <column gap="4" sx={gridCol}>
      <row gap="2" align="center" justify="between">
        <row gap="2" align="center">
          {#if countLabel()}
            <p sx={count}>{countLabel()}</p>
          {/if}
        </row>
        <row gap="2">
          {#for c of cats() key={c}}
            <button size="sm" tone="primary" variant={catSel() === c ? 'solid' : 'ghost'} on:click={() => pickCat(c)}>{c}</button>
          {/for}
        </row>
      </row>
      <row gap="2" align="center" justify="end">
        <input placeholder="Search — try 'tab' or 'overlay'" bind:value={query} sx={search} />
      </row>

      <grid cols="4" gap="3">
        {#for e of visibleList() key={e.key}}
          <card padding="none" data-tile={e.tag} data-live={e.live ? 'true' : null} sx={tile}>
            <column gap="0" sx={tileCol}>
              <row data-preview align="center" justify="center" sx={preview}>
                {#if !e.live}
                  <p sx={dormant}>Select pack to load</p>
                {:else if e.tag === 'button'}
                  <row gap="2" align="center">
                    <button variant="solid" tone="primary" motion:press={{ scale: 'press' }}>Solid</button>
                    <button variant="outline" tone="primary">Outline</button>
                  </row>
                {:else if e.tag === 'input'}
                  <input placeholder="Type — it's real" bind:value={name} />
                {:else if e.tag === 'textarea'}
                  <textarea placeholder="Notes" rows="2"></textarea>
                {:else if e.tag === 'select'}
                  <select bind:value={team}>
                    <option>Engineering</option>
                    <option>Design</option>
                    <option>Research</option>
                  </select>
                {:else if e.tag === 'combobox'}
                  <combobox placeholder="Filter frameworks…">
                    <option value="sail">Sail</option>
                    <option value="react">React</option>
                  </combobox>
                {:else if e.tag === 'checkbox'}
                  <label><input type="checkbox" bind:checked={agreed} /> I agree</label>
                {:else if e.tag === 'radio-group'}
                  <radio-group name="plan">
                    <label><input type="radio" name="plan" value="a" /> Free</label>
                    <label><input type="radio" name="plan" value="b" checked /> Pro</label>
                  </radio-group>
                {:else if e.tag === 'switch'}
                  <row gap="2" align="center">
                    <switch bind:checked={notify}></switch>
                    <p sx={mutedXs}>{notify() ? 'on' : 'off'}</p>
                  </row>
                {:else if e.tag === 'slider'}
                  <row gap="2" align="center" sx={style.w(full)}>
                    <slider bind:value={volume} tone="primary" sx={style.grow(1)}></slider>
                    <span sx={cat}>{volume()}</span>
                  </row>
                {:else if e.tag === 'toggle-group'}
                  <toggle-group>
                    <button>Day</button>
                    <button>Week</button>
                    <button>Month</button>
                  </toggle-group>
                {:else if e.tag === 'field'}
                  <field>
                    <label slot="label">Email</label>
                    <input type="email" placeholder="you@ship.dev" />
                  </field>
                {:else if e.tag === 'label'}
                  <label>Label</label>
                {:else if e.tag === 'fieldset'}
                  <fieldset>
                    <legend>Plan</legend>
                    <label><input type="checkbox" /> Include trials</label>
                  </fieldset>
                {:else if e.tag === 'dropzone'}
                  <div dropzone>Drop files</div>
                {:else if e.tag === 'theme-picker'}
                  <theme-picker></theme-picker>
                {:else if e.tag === 'alert'}
                  <alert tone="success"><strong>Saved.</strong> Changes are live.</alert>
                {:else if e.tag === 'progress'}
                  <progress value="0.62" sx={style.w(full)}></progress>
                {:else if e.tag === 'meter'}
                  <meter value="0.7" min="0" max="1" sx={style.w(full)}></meter>
                {:else if e.tag === 'spinner'}
                  <spinner></spinner>
                {:else if e.tag === 'skeleton'}
                  <span skeleton size="md" sx={style.w(full).h(3)}></span>
                {:else if e.tag === 'empty-state'}
                  <empty-state>
                    <strong slot="title">Nothing here</strong>
                    Try another filter.
                  </empty-state>
                {:else if e.tag === 'error-state'}
                  <error-state code="ERR_TIMEOUT">
                    <strong slot="title">Something went wrong</strong>
                    No route matches.
                  </error-state>
                {:else if e.tag === 'error-boundary'}
                  <error-boundary>
                    <p sx={mutedXs}>Wrapped safely</p>
                  </error-boundary>
                {:else if e.tag === 'toast'}
                  <toast tone="primary" duration="0">
                    Toast body
                    <button data-toast-close variant="ghost" size="sm">×</button>
                  </toast>
                {:else if e.tag === 'badge'}
                  <row gap="2" align="center">
                    <badge tone="primary">Primary</badge>
                    <badge tone="success" variant="soft">OK</badge>
                    <badge tone="destructive" variant="outline">Hot</badge>
                  </row>
                {:else if e.tag === 'chip'}
                  <row gap="2">
                    {#if kept()}
                      <chip tone="primary" removable on:sail-dismiss={dropChip}>Alpha</chip>
                    {/if}
                    <chip>Beta</chip>
                  </row>
                {:else if e.tag === 'status-dot'}
                  <row gap="2" align="center">
                    <span dot tone="success"></span>
                    <span dot tone="warning" pulse></span>
                    <span dot tone="destructive"></span>
                  </row>
                {:else if e.tag === 'card'}
                  <card interactive sx={style.pad(3)}>
                    <strong>A surface</strong>
                    <p sx={mutedXs}>hover lift is one attribute</p>
                  </card>
                {:else if e.tag === 'avatar'}
                  <avatar name="Ada Lovelace"></avatar>
                {:else if e.tag === 'icon'}
                  <row gap="3" align="center">
                    <icon name="check"></icon>
                    <icon name="search"></icon>
                    <icon name="settings"></icon>
                  </row>
                {:else if e.tag === 'separator' || e.tag === 'hr'}
                  <hr sx={style.w(full)} />
                {:else if e.tag === 'details'}
                  <details>
                    <summary>Details</summary>
                    Hidden until open.
                  </details>
                {:else if e.tag === 'collapsible'}
                  <collapsible>
                    <button slot="trigger">Toggle</button>
                    Collapsed body.
                  </collapsible>
                {:else if e.tag === 'prose'}
                  <prose><p>Prose surface — body type, links, lists.</p></prose>
                {:else if e.tag === 'tooltip'}
                  <tooltip>
                    <button slot="trigger" size="sm">Hover</button>
                    Tooltip copy
                  </tooltip>
                {:else if e.tag === 'hover-card'}
                  <hover-card>
                    <button slot="trigger" size="sm" variant="ghost">Card</button>
                    Hover card body
                  </hover-card>
                {:else if e.tag === 'popover'}
                  <popover>
                    <button slot="trigger" size="sm">Open</button>
                    Popover body
                  </popover>
                {:else if e.tag === 'dropdown'}
                  <dropdown>
                    <button slot="trigger" size="sm">Menu</button>
                    <button>Item A</button>
                    <button>Item B</button>
                  </dropdown>
                {:else if e.tag === 'context-menu'}
                  <context-menu>
                    <button slot="trigger" size="sm">Right-click</button>
                    <button>Copy</button>
                    <button>Paste</button>
                  </context-menu>
                {:else if e.tag === 'dialog'}
                  <dialog>
                    <button slot="trigger" size="sm" tone="primary">Open dialog</button>
                    <strong slot="title">Dialog</strong>
                    Live dialog surface.
                  </dialog>
                {:else if e.tag === 'tabs'}
                  <tabs>
                    <tab-list>
                      <tab>One</tab>
                      <tab>Two</tab>
                    </tab-list>
                    <tab-panel>Panel one</tab-panel>
                    <tab-panel>Panel two</tab-panel>
                  </tabs>
                {:else if e.tag === 'breadcrumb'}
                  <breadcrumb>
                    <a href="/components">components</a>
                    <small>badge</small>
                  </breadcrumb>
                {:else if e.tag === 'pagination'}
                  <pagination value="2">
                    <button data-page="1">1</button>
                    <button data-page="2">2</button>
                    <button data-page="3">3</button>
                  </pagination>
                {:else if e.tag === 'navbar'}
                  <navbar>
                    <a href="/components">Brand</a>
                    <a href="/components/button">Docs</a>
                  </navbar>
                {:else if e.tag === 'table'}
                  <table>
                    <thead><tr><th>Name</th><th>Role</th></tr></thead>
                    <tbody><tr><td>Ada</td><td>Eng</td></tr></tbody>
                  </table>
                {:else if e.tag === 'layout'}
                  <row gap="2"><div sx={style.pad(2).bg(muted).radius(sm)}>A</div><div sx={style.pad(2).bg(muted).radius(sm)}>B</div></row>
                {:else if e.tag === 'scroll-area'}
                  <scroll-area sx={style.h(8).w(full)}>
                    <p sx={mutedXs}>Scrollable region — more content than fits.</p>
                    <p sx={mutedXs}>Line two.</p>
                    <p sx={mutedXs}>Line three.</p>
                  </scroll-area>
                {:else if e.tag === 'sail-sparkline'}
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
                  <sail-media-image src="/favicon.svg" alt="Sail mark"></sail-media-image>
                {:else if e.tag === 'sail-media-av'}
                  <sail-media-av></sail-media-av>
                {:else if e.tag === 'sail-code-editor'}
                  <sail-code-editor value="const greeting = 'hi';" language="js" line-numbers size="sm"></sail-code-editor>
                {:else if e.tag === 'sail-mermaid'}
                  <sail-mermaid value="graph TD; A[Idea]-->B[Build]; B-->C[Ship]"></sail-mermaid>
                {:else if e.tag === 'sail-message'}
                  <sail-message role="user" state="user">Hello — live chat message.</sail-message>
                {:else if e.tag === 'sail-citation'}
                  <sail-citation source="docs.sail.dev" href="https://docs.sail.dev">Citation</sail-citation>
                {:else if e.tag === 'sail-composer'}
                  <sail-composer placeholder="Ask anything…"></sail-composer>
                {:else if e.tag === 'sail-thinking-dots'}
                  <sail-thinking-dots></sail-thinking-dots>
                {:else if e.tag === 'sail-tool-call'}
                  <sail-tool-call name="search" status="running">query: search</sail-tool-call>
                {:else}
                  <p sx={dormant}>{e.tag}</p>
                {/if}
              </row>
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

  <row sx={style.mt(6).padT(4).borderTWidth(1).borderT(border)} gap="2">
    <a variant="default" href="/sail/overview">← Introduction</a>
    <a variant="solid" tone="primary" href="/sail/language/layout">Layout →</a>
  </row>
</column>
