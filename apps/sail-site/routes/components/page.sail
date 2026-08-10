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
import { CATALOG, PACKS, CATEGORIES, packCount, type CatalogEntry } from '../../src/compositions/component-catalog'

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

    const email = signal('ada@ship.dev')
    const agreed = signal(true)
    const notify = signal(true)
    const volume = signal(60)
    const team = signal('Design')
    const plan = signal('pro')
    const kept = signal(true)
    const dropChip = (): void => kept.set(false)
    // Catalog toast tile: dismiss removes the instance; re-push remounts.
    let catalogToastSeq = 0
    const catalogToastItems = signal<{ key: string }[]>([{ key: 'catalog-toast-0' }])
    const pushCatalogToast = (): void => {
      catalogToastSeq += 1
      catalogToastItems.set([{ key: `catalog-toast-${catalogToastSeq}` }])
    }
    const onCatalogToastDismiss = (): void => {
      catalogToastItems.set([])
    }
    // Code-editor tile sample — template exprs ban the `function` keyword.
    const editorSample = () =>
      "const greet = (name) => {\n  return `hello, ${name}`\n}\n\ngreet('sail')\n"
    const toolArgs = () => '{ "env": "prod", "limit": 5 }'
    const toolResult = () => '[{ "id": "d-1842", "status": "healthy" }]'
    // sail-media-image src is JS identity (not reflect); needs CSP img-src https:.
    const DEMO_PHOTO =
      'https://images.unsplash.com/photo-1506905925346-21bda4d32df4?w=640&h=400&fit=crop&auto=format&q=80'
    effect(() => {
      void packSel()
      void visibleList()
      requestAnimationFrame(() => {
        document
          .querySelectorAll(
            'sail-media-image, [tile="sail-media-image"] sail-media-image, [data-tile="sail-media-image"] sail-media-image',
          )
          .forEach((el) => {
            const host = el as HTMLElement & { src?: string }
            if (host.src !== DEMO_PHOTO) host.src = DEMO_PHOTO
          })
      })
    })

    return {
      query, packSel, catSel, pickPack, pickCat,
      packs, cats, visibleList, countLabel, none,
      email, agreed, notify, volume, team, plan, kept, dropChip,
      catalogToastItems, pushCatalogToast, onCatalogToastDismiss,
      editorSample, toolArgs, toolResult,
    }
})
</controller>

<scope>
  style page = { box: maxW(breakpoint-desktop).mx(auto).padX(4).padY(4).w(full), when: { tablet: { box: padX(6).padY(5) } } }
  // Tooltips/hover-cards escape the tile; media stages clip themselves.
  style tile = { box: overflow(visible) }
  style tileCol = { box: h(full).overflow(visible) }
  // <row> preview: grid max-content tracks zero out w-full progress/meter.
  style preview = { box: pad(4).minH(11), paint: borderBWidth(1).borderB(border-subtle).bg(surface-0) }
  style tileMeta = { box: padX(3).padY(2) }
  style tileName = { text: fontSize(sm).fontWeight(semibold).fontFamily(mono) }
  style cat = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedSm = { box: maxW(panel-max-w-lg), paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) }
  style eyebrow = { box: mb(2), paint: fg(accent), text: fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide) }
  style sectionTitle = { box: mt(2), text: fontSize(sm).fontWeight(semibold) }
  // Pack chip strip (row flex + overflowX); avoid column base that zeros the grid.
  style packStack = { box: display(flex).flexDir(row).gap(2).overflowX(auto).padB(1).maxW(full) }
  style packBtn = { box: pad(0).overflow(hidden), paint: radius(md).cursor(pointer).borderWidth(1).border(solid).border(border).bg(transparent), text: textAlign(start).whitespace(nowrap), when: { hover: { paint: bg(hover-bg) }, pressed: { paint: bg(muted).border(primary).shadow(sm) } } }
  style packLabel = { text: fontSize(sm) }
  style packCount = { paint: fg(primary), text: fontSize(xs).fontFamily(mono) }
  style railNote = { paint: fg(muted-foreground), text: fontSize(xs).leading(relaxed) }
  style search = { box: w(full).maxW(panel-max-w-sm) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style count = { paint: fg(muted-foreground), text: fontSize(sm).fontFamily(mono) }
  style dormant = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono).textAlign(center) }
  style fieldLabel = { text: fontSize(sm).fontWeight(medium) }
  // Definite box: CE host is display:contents — without height the img collapses.
  style mediaFrame = { box: w(full).h(40).minH(40).overflow(hidden), paint: radius(md).bg(muted).borderWidth(1).border(border) }
  style mediaStage = { box: w(full).h(full).minH(40) }
  style catRow = { box: overflowX(auto).maxW(full) }
</scope>

<column gap="5" style(page, { box: w(full).minW(0) })>
  <column gap="1">
    <p style(eyebrow)>Components</p>
    <h1 style({ text: fontSize(2xl).fontWeight(semibold).tracking(tight) })>Catalog</h1>
    <p style(mutedSm)>
      Every tile is a live element — click a name for API, variants, and usage.
      Packs other than Component primitives load when you select them.
    </p>
  </column>

  <column gap="2" style({ box: w(full).minW(0) })>
    <p style(eyebrow)>Packs</p>
    <div label="Component packs" style(packStack)>
      <for {packs() as p} key={p.id}>
        <button on:click={() => pickPack(p.id)} pressed={packSel() === p.id ? 'true' : 'false'} style(packBtn, { box: shrink(0) })>

          <row gap="2" align="center" style(tileMeta)>
            <strong style(packLabel)>{p.label}</strong>
            <span style(packCount)>{p.count}</span>
          </row>
        </button>
      </for>
    </div>
    <p style(railNote)>
      Primitives compile in with no import. Charts, media, chat, mermaid, and the code editor mount from their pack when selected.
    </p>
  </column>

  <column gap="4" style({ box: w(full).minW(0) })>
    <row gap="2" align="center" justify="between">
      <row gap="2" align="center">
        <if {countLabel()}>
          <p style(count)>{countLabel()}</p>
        </if>
      </row>
      <row gap="1" align="center" style(catRow)>
        <for {cats() as c} key={c}>
          <button size="sm" tone="primary" variant={catSel() === c ? 'solid' : 'ghost'} on:click={() => pickCat(c)} pressed={catSel() === c ? 'true' : 'false'}>{c}</button>
        </for>
      </row>
    </row>
    <row gap="2" align="center" justify="end">
      <input type="search" placeholder="Search components…" bind:value={query} style(search)>
    </row>

    <grid cols="2" gap="3">
        <for {visibleList() as e} key={e.key}>
          <card padding="none" tile={e.tag} live={e.live ? 'true' : null} style(tile)>
            <column gap="0" style(tileCol)>
              <row align="center" justify="center" style(preview, { box: grow(1).w(full) })>
                <if {!e.live}>
                  <p style(dormant)>Select pack in the rail to mount</p>
                {else-if {e.tag === 'button'} => <row gap="2" align="center">
                    <button variant="solid" tone="primary">Save</button>
                    <button variant="outline">Cancel</button>
                    <button variant="ghost" tone="destructive" size="sm">Delete</button>
                  </row>}{else-if {e.tag === 'input'} => <column gap="1" style({ box: w(full) })>
                    <label style(fieldLabel)>Work email</label>
                    <input type="email" placeholder="you@company.com" bind:value={email} />
                  </column>}{else-if {e.tag === 'textarea'} => <column gap="1" style({ box: w(full) })>
                    <label style(fieldLabel)>Release notes</label>
                    <textarea placeholder="What changed in this ship?" rows="2"></textarea>
                  </column>}{else-if {e.tag === 'select'} => <column gap="1" style({ box: w(full) })>
                    <label style(fieldLabel)>Team</label>
                    <select bind:value={team}>
                      <option>Engineering</option>
                      <option>Design</option>
                      <option>Research</option>
                    </select>
                  </column>}{else-if {e.tag === 'combobox'} => <column gap="1" style({ box: w(full) })>
                    <label style(fieldLabel)>Framework</label>
                    <combobox placeholder="Filter…">
                      <option value="sail">Sail</option>
                      <option value="react">React</option>
                      <option value="svelte">Svelte</option>
                    </combobox>
                  </column>}{else-if {e.tag === 'checkbox'} => <column gap="2">
                    <label style({ text: fontSize(sm) })>
                      <input type="checkbox" bind:checked={agreed} /> Email me product updates
                    </label>
                    <label style({ paint: fg(muted-foreground), text: fontSize(sm) })>
                      <input type="checkbox" disabled /> Admin-only (disabled)
                    </label>
                  </column>}{else-if {e.tag === 'radio-group'} => <radio-group name="catalog-plan" value={plan()}>
                    <label style({ text: fontSize(sm) })><input type="radio" name="catalog-plan" value="free" /> Free — community</label>
                    <label style({ text: fontSize(sm) })><input type="radio" name="catalog-plan" value="pro" /> Pro — team seats</label>
                    <label style({ text: fontSize(sm) })><input type="radio" name="catalog-plan" value="enterprise" /> Enterprise</label>
                  </radio-group>}{else-if {e.tag === 'switch'} => <row gap="2" align="center">
                    <switch bind:checked={notify} label="Desktop notifications"></switch>
                    <p style(mutedXs)>{notify() ? 'Notifications on' : 'Notifications off'}</p>
                  </row>}{else-if {e.tag === 'slider'} => <column gap="1" style({ box: w(full) })>
                    <row justify="between" align="center">
                      <span style(fieldLabel)>Volume</span>
                      <span style(cat)>{volume()}%</span>
                    </row>
                    <slider bind:value={volume} style({ box: w(full) })></slider>
                  </column>}{else-if {e.tag === 'toggle-group'} => <toggle-group value="day" label="Range">
                    <toggle value="day">Day</toggle>
                    <toggle value="week">Week</toggle>
                    <toggle value="month">Month</toggle>
                  </toggle-group>}{else-if {e.tag === 'field'} => <field style({ box: w(full) })>
                    <label slot="label">Email</label>
                    <input type="email" placeholder="you@ship.dev" />
                    <span slot="hint" style(mutedXs)>We’ll never share this.</span>
                  </field>}{else-if {e.tag === 'label'} => <column gap="1">
                    <label for="demo-label-input">Display name</label>
                    <input id="demo-label-input" placeholder="Ada Lovelace" />
                  </column>}{else-if {e.tag === 'fieldset'} => <fieldset>
                    <legend>Billing options</legend>
                    <column gap="1">
                      <label style({ text: fontSize(sm) })><input type="checkbox" checked /> Include trials</label>
                      <label style({ text: fontSize(sm) })><input type="checkbox" /> Annual discount</label>
                    </column>
                  </fieldset>}{else-if {e.tag === 'dropzone'} => <div dropzone style({  box: w(full).minH(14).pad(3)  })>
                    <p style({ text: fontSize(sm).fontWeight(medium) })>Drop manifests here</p>
                    <p style(mutedXs)>or click to browse · .json, .yaml</p>
                  </div>}{else-if {e.tag === 'theme-picker'} => <theme-picker></theme-picker>}{else-if {e.tag === 'ticker'} => <ticker duration="normal" style({ box: w(full).overflow(hidden).padY(2), paint: bg(muted).radius(md) })>
                    <p style({ box: padX(3), paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono).whitespace(nowrap) })>tokens · signals · a11y · hot reload · SSR ·&nbsp;</p>
                  </ticker>}{else-if {e.tag === 'alert'} => <alert tone="success"><strong>Deployed.</strong> v0.4.2 is live on production.</alert>}{else-if {e.tag === 'progress'} => <column gap="1" style({ box: w(full) })>
                    <span style(mutedXs)>Uploading assets… 62%</span>
                    <progress value="0.62" style({ box: w(full) })></progress>
                  </column>}{else-if {e.tag === 'meter'} => <column gap="1" style({ box: w(full) })>
                    <span style(mutedXs)>Storage used</span>
                    <meter value="0.7" min="0" max="1" style({ box: w(full) })></meter>
                  </column>}{else-if {e.tag === 'spinner'} => <row gap="2" align="center">
                    <spinner></spinner>
                    <span style(mutedXs)>Syncing…</span>
                  </row>}{else-if {e.tag === 'skeleton'} => <column gap="2" style({ box: w(full) })>
                    <skeleton size="md" style({ box: w(full).h(3) })></skeleton>
                    <skeleton size="sm" style({ box: w(28).h(2) })></skeleton>
                  </column>}{else-if {e.tag === 'empty-state'} => <empty-state>
                    <strong slot="title">No deploys yet</strong>
                    Ship something from the CLI to populate this list.
                  </empty-state>}{else-if {e.tag === 'error-state'} => <error-state code="ERR_TIMEOUT">
                    <strong slot="title">Upstream timed out</strong>
                    Retry in a moment — the registry is slow.
                  </error-state>}{else-if {e.tag === 'error-boundary'} => <error-boundary>
                    <p style(mutedXs)>Child tree isolated from render faults.</p>
                  </error-boundary>}{else-if {e.tag === 'toast'} => <column gap="2" style({ box: w(full).minW(0) })>
                    <button type="button" size="sm" variant="ghost" on:click={pushCatalogToast} toast-show>Show toast</button>
                    <for {catalogToastItems() as t} key={t.key}>
                      <toast tone="primary" duration="0" on-dismiss={onCatalogToastDismiss}>Invite sent to ada@ship.dev</toast>
                    </for>
                  </column>}{else-if {e.tag === 'badge'} => <row gap="2" align="center">
                    <badge tone="primary">v0.4</badge>
                    <badge tone="success" variant="soft">Healthy</badge>
                    <badge tone="destructive" variant="outline">3 errors</badge>
                  </row>}{else-if {e.tag === 'chip'} => <row gap="2" align="center">
                    <if {kept()}>
                      <chip size="sm" tone="primary" removable on-dismiss={dropChip}>typescript</chip>
                    </if>
                    <chip size="sm" variant="outline">a11y</chip>
                    <chip size="sm" variant="outline">ssr</chip>
                  </row>}{else-if {e.tag === 'card'} => <card interactive style({ box: pad(3).w(full) })>
                    <strong>Harbor Signals</strong>
                    <p style(mutedXs)>4 tracks · last played today</p>
                  </card>}{else-if {e.tag === 'avatar'} => <row gap="2" align="center">
                    <avatar name="Ada Lovelace" src="https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=128&h=128&fit=crop&auto=format&q=80"></avatar>
                    <avatar name="Grace Hopper" src="https://images.unsplash.com/photo-1438761681033-6461ffad8d80?w=128&h=128&fit=crop&auto=format&q=80" tone="accent"></avatar>
                    <avatar initials="SK" size="sm"></avatar>
                  </row>}{else-if {e.tag === 'icon'} => <row gap="3" align="center">
                    <icon name="check"></icon>
                    <icon name="search"></icon>
                    <icon name="settings"></icon>
                  </row>}{else-if {e.tag === 'separator'} => <column gap="2" style({ box: w(full) })>
                    <p style(mutedXs)>Section A</p>
                    <separator style({ box: w(full) })></separator>
                    <p style(mutedXs)>Section B</p>
                    <row gap="2" align="center">
                      <span style(mutedXs)>Left</span>
                      <separator orientation="vertical"></separator>
                      <span style(mutedXs)>Right</span>
                    </row>
                  </column>}{else-if {e.tag === 'accordion'} => <accordion style({ box: w(full) })>
                    <span slot="trigger" style({ text: fontSize(sm).fontWeight(semibold) })>Webhook endpoints</span>
                    <p style(mutedXs)>POST /v1/hooks · signed with your workspace secret.</p>
                  </accordion>}{else-if {e.tag === 'kbd'} => <row gap="1" align="center">
                    <kbd>Ctrl</kbd>
                    <span style(mutedXs)>+</span>
                    <kbd>K</kbd>
                  </row>}{else-if {e.tag === 'prose'} => <prose><p>Body type with <strong>emphasis</strong>, <code>inline()</code>, and lists — no class soup.</p></prose>}{else-if {e.tag === 'tooltip'} => <!-- placement bottom so the bubble stays inside the tile band -->
                  <tooltip content="Copy URL" placement="bottom">
                    <span style({ paint: fg(muted-foreground).borderBWidth(1).borderB(border).border(dashed).cursor(help), text: fontSize(sm) })>ship.dev/d/ab12</span>
                  </tooltip>}{else-if {e.tag === 'hover-card'} => <column gap="2" align="center">
                    <a href="#" popovertarget="grid-hc" style({ paint: fg(primary), text: fontSize(sm) })>@ada</a>
                    <hover-card id="grid-hc">
                      <row gap="2" align="center">
                        <avatar name="Ada Lovelace" src="https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=64&h=64&fit=crop&auto=format&q=80" size="sm"></avatar>
                        <column gap="0">
                          <strong>Ada Lovelace</strong>
                          <p style(mutedXs)>Mathematician · first programmer</p>
                        </column>
                      </row>
                    </hover-card>
                  </column>}{else-if {e.tag === 'popover'} => <column gap="2" align="center">
                    <button size="sm" variant="outline" popovertarget="grid-pop">Filters</button>
                    <popover id="grid-pop">
                      <column gap="2">
                        <p style({ text: fontSize(sm).fontWeight(medium) })>Status</p>
                        <label style({ text: fontSize(sm) })><input type="checkbox" checked /> Healthy</label>
                        <label style({ text: fontSize(sm) })><input type="checkbox" /> Failed</label>
                      </column>
                    </popover>
                  </column>}{else-if {e.tag === 'dropdown'} => <column gap="2" align="center">
                    <button size="sm" variant="outline" popovertarget="grid-dd">Row actions ▾</button>
                    <dropdown id="grid-dd">
                      <menu-item value="edit">Edit</menu-item>
                      <menu-item value="duplicate">Duplicate</menu-item>
                      <menu-item value="archive">Archive</menu-item>
                    </dropdown>
                  </column>}{else-if {e.tag === 'context-menu'} => <column gap="2" align="center">
                    <p id="grid-cm-target" style({ box: pad(3), paint: fg(muted-foreground).borderWidth(1).border(border).border(dashed).radius(md), text: fontSize(sm) })>Right-click this card</p>
                    <context-menu for="grid-cm-target" id="grid-cm">
                      <menu-item value="copy">Copy id</menu-item>
                      <menu-item value="paste">Paste</menu-item>
                      <menu-item value="delete">Delete</menu-item>
                    </context-menu>
                  </column>}{else-if {e.tag === 'dialog'} => <column gap="2" align="center">
                    <button size="sm" tone="primary" command="show-modal" commandfor="grid-dlg">Invite teammate</button>
                    <dialog id="grid-dlg">
                      <header>Invite to workspace</header>
                      <section>
                        <field>
                          <label slot="label">Email</label>
                          <input type="email" placeholder="name@company.com" />
                        </field>
                      </section>
                      <footer>
                        <button command="close" commandfor="grid-dlg" variant="ghost" size="sm">Cancel</button>
                        <button command="close" commandfor="grid-dlg" variant="solid" tone="primary" size="sm">Send invite</button>
                      </footer>
                    </dialog>
                  </column>}{else-if {e.tag === 'tabs'} => <tabs style({ box: w(full) })>
                    <tab-list>
                      <tab>Overview</tab>
                      <tab>Metrics</tab>
                    </tab-list>
                    <tab-panel><p style(mutedXs)>Uptime 99.98% · 12 deploys this week.</p></tab-panel>
                    <tab-panel><p style(mutedXs)>p95 latency 42ms.</p></tab-panel>
                  </tabs>}{else-if {e.tag === 'breadcrumb'} => <breadcrumb>
                    <a href="/components">components</a>
                    <a href="/components">inputs</a>
                    <small>field</small>
                  </breadcrumb>}{else-if {e.tag === 'pagination'} => <pagination value="2" label="Pages">
                    <page value="1">1</page>
                    <page value="2">2</page>
                    <page value="3">3</page>
                  </pagination>}{else-if {e.tag === 'navbar'} => <navbar bordered style({ box: w(full) })>
                    <strong slot="brand" style({ text: fontSize(sm) })>Harbor</strong>
                    <span style({ box: padX(2).padY(1), paint: radius(md).fg(muted-foreground), text: fontSize(sm) })>Docs</span>
                    <span style({ box: padX(2).padY(1), paint: radius(md).bg(muted), text: fontSize(sm).fontWeight(medium) })>App</span>
                  </navbar>}{else-if {e.tag === 'table'} => <table style({ box: w(full) })>
                    <thead><tr><th>Service</th><th>Status</th></tr></thead>
                    <tbody>
                      <tr><td>api</td><td>healthy</td></tr>
                      <tr><td>worker</td><td>degraded</td></tr>
                    </tbody>
                  </table>}{else-if {e.tag === 'layout'} => <row gap="2" style({ box: w(full) })>
                    <div style({  box: grow(1).pad(2), paint: bg(muted).radius(sm)  })>                      <span style({ box: display(contents), text: fontSize(xs).fontFamily(mono) })>                        main
                      </span>
</div>
                    <div style({ box: pad(2).w(16), paint: bg(muted).radius(sm) })>                      <span style({ box: display(contents), text: fontSize(xs).fontFamily(mono) })>                        aside
                      </span>
</div>
                  </row>}{else-if {e.tag === 'scroll-area'} => <scroll-area style({ box: h(32).w(full).minH(32).pad(3), paint: borderWidth(1).border(border).radius(md).bg(surface-0) })>
                    <column gap="2">
                      <p style(mutedXs)>Build 1842 · shipped · 14:02</p>
                      <p style(mutedXs)>Build 1841 · shipped · 11:40</p>
                      <p style(mutedXs)>Build 1840 · failed · 09:12</p>
                      <p style(mutedXs)>Build 1839 · shipped · yesterday</p>
                      <p style(mutedXs)>Build 1838 · shipped · yesterday</p>
                      <p style(mutedXs)>Build 1837 · shipped · Mon</p>
                    </column>
                  </scroll-area>}{else-if {e.tag === 'sail-sparkline'} => <sail-sparkline data="2,5,3,8,4,6,7" width="120" height="40"></sail-sparkline>}{else-if {e.tag === 'sail-area-chart'} => <sail-area-chart series='[{"name":"a","data":[2,4,3,5]}]' labels="Q1,Q2,Q3,Q4" width="160" height="60"></sail-area-chart>}{else-if {e.tag === 'sail-bar-chart'} => <sail-bar-chart data="2,5,3,8,4" labels="A,B,C,D,E" width="160" height="60"></sail-bar-chart>}{else-if {e.tag === 'sail-donut'} => <sail-donut data='[{"label":"a","value":40},{"label":"b","value":30},{"label":"c","value":30}]' diameter="80" thickness="10"></sail-donut>}{else-if {e.tag === 'sail-gauge'} => <sail-gauge value="72" diameter="80" thickness="10"></sail-gauge>}{else-if {e.tag === 'sail-heatmap'} => <sail-heatmap rows="3" cols="5" values="1,2,3,4,3,4,5,6,5,4,3,2,1,2,3" cell-size="18" gap="3" tone="primary"></sail-heatmap>}{else-if {e.tag === 'sail-line-chart'} => <sail-line-chart series='[{"name":"a","data":[2,4,3,5]}]' labels="Q1,Q2,Q3,Q4" width="160" height="60"></sail-line-chart>}{else-if {e.tag === 'sail-media-image'} => <div style(mediaFrame)>
                    <sail-media-image alt="Mountain lake at dusk" fit="cover" zoomable={false} style(mediaStage)></sail-media-image>
                  </div>}{else-if {e.tag === 'sail-media-av'} => <sail-media-av></sail-media-av>}{else-if {e.tag === 'sail-code-editor'} => <sail-code-editor value={editorSample()} language="javascript" line-numbers></sail-code-editor>}{else-if {e.tag === 'sail-mermaid'} => <sail-mermaid value="graph TD; A[Idea]-->B[Build]; B-->C[Ship]"></sail-mermaid>}{else-if {e.tag === 'sail-message'} => <column gap="2" style({ box: w(full) })>
                    <sail-message from="user" status="sent">Can you summarize yesterday’s deploy?</sail-message>
                    <sail-message from="assistant" status="sent">v0.4.2 went out at 14:02 — 3 services, zero rollbacks.</sail-message>
                  </column>}{else-if {e.tag === 'sail-citation'} => <sail-citation source="docs.ship.dev" href="https://docs.ship.dev">Deploy pipeline overview</sail-citation>}{else-if {e.tag === 'sail-composer'} => <sail-composer placeholder="Message the agent…" style({  box: w(full).minW(0)  })></sail-composer>}{else-if {e.tag === 'sail-thinking-dots'} => <row gap="2" align="center">
                    <sail-thinking-dots></sail-thinking-dots>
                    <span style(mutedXs)>Agent is thinking…</span>
                  </row>}{else-if {e.tag === 'sail-tool-call'} => <sail-tool-call name="list_deploys" status="ok" open style({ box: w(full) })>
                    {toolArgs()}
                    <span slot="result">{toolResult()}</span>
                  </sail-tool-call>}{else => <p style(dormant)>{e.tag}</p>}</if>
              </row>
              <column gap="1" style(tileMeta)>
                <a href={`/components/${e.slug}`} style(tileName)>{e.tag}</a>
                <row gap="2" align="center" justify="between">
                  <badge size="sm" tone={e.badgeTone} variant="soft">{e.badgeLabel}</badge>
                  <span style(cat)>{e.cat}</span>
                </row>
              </column>
            </column>
          </card>
        </for>
      </grid>

    <if {none()}>
      <empty-state>
        <strong slot="title">Nothing matches</strong>
        Clear a filter or two — or search for less.
      </empty-state>
    </if>
  </column>

</column>
