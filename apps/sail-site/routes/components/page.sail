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

    const email = signal('ada@ship.dev')
    const agreed = signal(true)
    const notify = signal(true)
    const volume = signal(60)
    const team = signal('Design')
    const plan = signal('pro')
    const kept = signal(true)
    const dropChip = (): void => kept.set(false)
    // Multi-line sample for the code-editor tile (can't put `function` in a
    // template expr — renderable expressions ban that keyword).
    const editorSample = () =>
      "const greet = (name) => {\n  return `hello, ${name}`\n}\n\ngreet('sail')\n"
    const toolArgs = () => '{ "env": "prod", "limit": 5 }'
    const toolResult = () => '[{ "id": "d-1842", "status": "healthy" }]'
    // Real photo (Unsplash). `src` is a JS identity property, not a reflect attr
    // (source-in-never-fetch). Requires deck CSP img-src to allow https:.
    const DEMO_PHOTO =
      'https://images.unsplash.com/photo-1506905925346-21bda4d32df4?w=640&h=400&fit=crop&auto=format&q=80'
    effect(() => {
      void packSel()
      void visibleList()
      requestAnimationFrame(() => {
        document.querySelectorAll('[data-tile="sail-media-image"] sail-media-image').forEach((el) => {
          const host = el as HTMLElement & { src?: string }
          if (host.src !== DEMO_PHOTO) host.src = DEMO_PHOTO
        })
      })
    })

    return {
      query, packSel, catSel, pickPack, pickCat,
      packs, cats, visibleList, countLabel, none,
      email, agreed, notify, volume, team, plan, kept, dropChip, editorSample, toolArgs, toolResult,
    }
})
</controller>

<scope>
  // Gutter: catalog was flush to the viewport edge (rail + grid). Real pad.
  const page = style.maxW(breakpoint-desktop).mx(auto).padX(4).padY(4).w(full).when({
    tablet: style.padX(6).padY(5),
  })
  // Visible so tooltips/hover-cards can escape the tile; media stages clip themselves.
  const tile = style.overflow(visible)
  const tileCol = style.h(full).overflow(visible)
  // Flex via <row> (not <center>/grid place-items:center): auto grid tracks
  // collapse to max-content, so width:100% bars (progress/meter) measure 0.
  // Flex parent has a definite tile width, so w="full" children get a real track.
  const preview = style.pad(4).minH(11).borderBWidth(1).borderB(border-subtle).bg(surface-0)
  const tileMeta = style.padX(3).padY(2)
  const tileName = style.fontSize(sm).fontWeight(semibold).fontFamily(mono)
  const cat = style.fontSize(xs).fg(muted-foreground).fontFamily(mono)
  const mutedSm = style.fontSize(sm).fg(muted-foreground).leading(relaxed).maxW(panel-max-w-lg)
  const eyebrow = style.fontFamily(mono).fontSize(xs).textCase(uppercase).tracking(wide).fg(accent).mb(2)
  const sectionTitle = style.fontSize(sm).fontWeight(semibold).mt(2)
  // Packs are a wrapping chip strip above the grid (not a side rail).
  // Side-by-side via flexDir(row)+when(tablet) loses the cascade war:
  // base w(full)/flexDir(column) can beat the tablet override, grid → 0 width.
  const packStack = style.display(flex).flexDir(row).gap(2).overflowX(auto).padB(1).maxW(full)
  const packBtn = style.radius(md).cursor(pointer).borderWidth(1).border(solid).border(border).pad(0).overflow(hidden).bg(transparent).textAlign(start).whitespace(nowrap).when({
    hover: style.bg(hover-bg),
    pressed: style.bg(muted).border(primary).shadow(sm),
  })
  const packLabel = style.fontSize(sm)
  const packCount = style.fontSize(xs).fg(primary).fontFamily(mono)
  const railNote = style.fontSize(xs).fg(muted-foreground).leading(relaxed)
  const search = style.w(full).maxW(panel-max-w-sm)
  const mutedXs = style.fontSize(xs).fg(muted-foreground)
  const count = style.fontSize(sm).fg(muted-foreground).fontFamily(mono)
  const dormant = style.fontSize(xs).fg(muted-foreground).fontFamily(mono).textAlign(center)
  const fieldLabel = style.fontSize(sm).fontWeight(medium)
  // Definite box: CE host is display:contents, so the stage needs a real
  // height or the <img> collapses when the source fails / is aspect-less.
  const mediaFrame = style.w(full).h(40).minH(40).radius(md).overflow(hidden).bg(muted).borderWidth(1).border(border)
  const mediaStage = style.w(full).h(full).minH(40)
  const catRow = style.overflowX(auto).maxW(full)
</scope>

<column gap="5" w="full" min-w="0" sx={page}>
  <column gap="1">
    <p sx={eyebrow}>Components</p>
    <h1 sx={style.fontSize(2xl).fontWeight(semibold).tracking(tight)}>Catalog</h1>
    <p sx={mutedSm}>
      Every tile is a live element — click a name for API, variants, and usage.
      Packs other than Component primitives load when you select them.
    </p>
  </column>

  <column gap="2" w="full" min-w="0">
    <p sx={eyebrow}>Packs</p>
    <div sx={packStack} role="toolbar" aria-label="Component packs">
      <for {packs()} as="p" key={p.id}>
        <button shrink="0" sx={packBtn} on:click={() => pickPack(p.id)} aria-pressed={packSel() === p.id ? 'true' : 'false'}>
          <row gap="2" align="center" sx={tileMeta}>
            <strong sx={packLabel}>{p.label}</strong>
            <span sx={packCount}>{p.count}</span>
          </row>
        </button>
      </for>
    </div>
    <p sx={railNote}>
      Primitives compile in with no import. Charts, media, chat, mermaid, and the code editor mount from their pack when selected.
    </p>
  </column>

  <column gap="4" w="full" min-w="0">
    <row gap="2" align="center" justify="between">
      <row gap="2" align="center">
        <if {countLabel()}>
          <p sx={count}>{countLabel()}</p>
        </if>
      </row>
      <row gap="1" align="center" sx={catRow}>
        <for {cats()} as="c" key={c}>
          <button size="sm" tone="primary" variant={catSel() === c ? 'solid' : 'ghost'} on:click={() => pickCat(c)} aria-pressed={catSel() === c ? 'true' : 'false'}>{c}</button>
        </for>
      </row>
    </row>
    <row gap="2" align="center" justify="end">
      <input type="search" placeholder="Search components…" bind:value={query} sx={search} />
    </row>

    <grid cols="2" gap="3">
        <for {visibleList()} as="e" key={e.key}>
          <card padding="none" data-tile={e.tag} data-live={e.live ? 'true' : null} sx={tile}>
            <column gap="0" sx={tileCol}>
              <row data-preview align="center" justify="center" grow w="full" sx={preview}>
                <if {!e.live}>
                  <p sx={dormant}>Select pack in the rail to mount</p>
                <else-if {e.tag === 'button'}>
                  <row gap="2" align="center">
                    <button variant="solid" tone="primary">Save</button>
                    <button variant="outline">Cancel</button>
                    <button variant="ghost" tone="destructive" size="sm">Delete</button>
                  </row>
                </else-if><else-if {e.tag === 'input'}>
                  <column gap="1" w="full">
                    <label sx={fieldLabel}>Work email</label>
                    <input type="email" placeholder="you@company.com" bind:value={email} />
                  </column>
                </else-if><else-if {e.tag === 'textarea'}>
                  <column gap="1" w="full">
                    <label sx={fieldLabel}>Release notes</label>
                    <textarea placeholder="What changed in this ship?" rows="2"></textarea>
                  </column>
                </else-if><else-if {e.tag === 'select'}>
                  <column gap="1" w="full">
                    <label sx={fieldLabel}>Team</label>
                    <select bind:value={team}>
                      <option>Engineering</option>
                      <option>Design</option>
                      <option>Research</option>
                    </select>
                  </column>
                </else-if><else-if {e.tag === 'combobox'}>
                  <column gap="1" w="full">
                    <label sx={fieldLabel}>Framework</label>
                    <combobox placeholder="Filter…">
                      <option value="sail">Sail</option>
                      <option value="react">React</option>
                      <option value="svelte">Svelte</option>
                    </combobox>
                  </column>
                </else-if><else-if {e.tag === 'checkbox'}>
                  <column gap="2">
                    <label sx={style.fontSize(sm)}>
                      <input type="checkbox" bind:checked={agreed} /> Email me product updates
                    </label>
                    <label sx={style.fontSize(sm).fg(muted-foreground)}>
                      <input type="checkbox" disabled /> Admin-only (disabled)
                    </label>
                  </column>
                </else-if><else-if {e.tag === 'radio-group'}>
                  <radio-group name="catalog-plan" value={plan()}>
                    <label sx={style.fontSize(sm)}><input type="radio" name="catalog-plan" value="free" /> Free — community</label>
                    <label sx={style.fontSize(sm)}><input type="radio" name="catalog-plan" value="pro" /> Pro — team seats</label>
                    <label sx={style.fontSize(sm)}><input type="radio" name="catalog-plan" value="enterprise" /> Enterprise</label>
                  </radio-group>
                </else-if><else-if {e.tag === 'switch'}>
                  <row gap="2" align="center">
                    <switch bind:checked={notify} aria-label="Desktop notifications"></switch>
                    <p sx={mutedXs}>{notify() ? 'Notifications on' : 'Notifications off'}</p>
                  </row>
                </else-if><else-if {e.tag === 'slider'}>
                  <column gap="1" w="full">
                    <row justify="between" align="center">
                      <span sx={fieldLabel}>Volume</span>
                      <span sx={cat}>{volume()}%</span>
                    </row>
                    <slider bind:value={volume} tone="primary" w="full"></slider>
                  </column>
                </else-if><else-if {e.tag === 'toggle-group'}>
                  <toggle-group value="day" aria-label="Range">
                    <toggle value="day">Day</toggle>
                    <toggle value="week">Week</toggle>
                    <toggle value="month">Month</toggle>
                  </toggle-group>
                </else-if><else-if {e.tag === 'field'}>
                  <field w="full">
                    <label slot="label">Email</label>
                    <input type="email" placeholder="you@ship.dev" />
                    <span slot="hint" sx={mutedXs}>We'll never share this.</span>
                  </field>
                </else-if><else-if {e.tag === 'label'}>
                  <column gap="1">
                    <label for="demo-label-input">Display name</label>
                    <input id="demo-label-input" placeholder="Ada Lovelace" />
                  </column>
                </else-if><else-if {e.tag === 'fieldset'}>
                  <fieldset>
                    <legend>Billing options</legend>
                    <column gap="1">
                      <label sx={style.fontSize(sm)}><input type="checkbox" checked /> Include trials</label>
                      <label sx={style.fontSize(sm)}><input type="checkbox" /> Annual discount</label>
                    </column>
                  </fieldset>
                </else-if><else-if {e.tag === 'dropzone'}>
                  <div dropzone w="full" sx={style.minH(14).w(full).pad(3)}>
                    <p sx={style.fontSize(sm).fontWeight(medium)}>Drop manifests here</p>
                    <p sx={mutedXs}>or click to browse · .json, .yaml</p>
                  </div>
                </else-if><else-if {e.tag === 'theme-picker'}>
                  <theme-picker></theme-picker>
                </else-if><else-if {e.tag === 'ticker'}>
                  <ticker duration="normal" sx={style.w(full).overflow(hidden).bg(muted).padY(2).radius(md)}>
                    <p sx={style.fontSize(xs).fontFamily(mono).fg(muted-foreground).whitespace(nowrap).padX(3)}>tokens · signals · a11y · hot reload · SSR ·&nbsp;</p>
                  </ticker>
                </else-if><else-if {e.tag === 'alert'}>
                  <alert tone="success"><strong>Deployed.</strong> v0.4.2 is live on production.</alert>
                </else-if><else-if {e.tag === 'progress'}>
                  <column gap="1" w="full">
                    <span sx={mutedXs}>Uploading assets… 62%</span>
                    <progress value="0.62" w="full"></progress>
                  </column>
                </else-if><else-if {e.tag === 'meter'}>
                  <column gap="1" w="full">
                    <span sx={mutedXs}>Storage used</span>
                    <meter value="0.7" min="0" max="1" w="full"></meter>
                  </column>
                </else-if><else-if {e.tag === 'spinner'}>
                  <row gap="2" align="center">
                    <spinner></spinner>
                    <span sx={mutedXs}>Syncing…</span>
                  </row>
                </else-if><else-if {e.tag === 'skeleton'}>
                  <column gap="2" w="full">
                    <span skeleton size="md" sx={style.w(full).h(3)}></span>
                    <span skeleton size="sm" sx={style.w(28).h(2)}></span>
                  </column>
                </else-if><else-if {e.tag === 'empty-state'}>
                  <empty-state>
                    <strong slot="title">No deploys yet</strong>
                    Ship something from the CLI to populate this list.
                  </empty-state>
                </else-if><else-if {e.tag === 'error-state'}>
                  <error-state code="ERR_TIMEOUT">
                    <strong slot="title">Upstream timed out</strong>
                    Retry in a moment — the registry is slow.
                  </error-state>
                </else-if><else-if {e.tag === 'error-boundary'}>
                  <error-boundary>
                    <p sx={mutedXs}>Child tree isolated from render faults.</p>
                  </error-boundary>
                </else-if><else-if {e.tag === 'toast'}>
                  <toast tone="primary" duration="0">Invite sent to ada@ship.dev</toast>
                </else-if><else-if {e.tag === 'badge'}>
                  <row gap="2" align="center">
                    <badge tone="primary">v0.4</badge>
                    <badge tone="success" variant="soft">Healthy</badge>
                    <badge tone="destructive" variant="outline">3 errors</badge>
                  </row>
                </else-if><else-if {e.tag === 'chip'}>
                  <row gap="2" align="center">
                    <if {kept()}>
                      <chip size="sm" tone="primary" removable on:sail-dismiss={dropChip}>typescript</chip>
                    </if>
                    <chip size="sm" variant="outline">a11y</chip>
                    <chip size="sm" variant="outline">ssr</chip>
                  </row>
                </else-if><else-if {e.tag === 'status-dot'}>
                  <column gap="1">
                    <row gap="2" align="center"><span dot tone="success"></span><span sx={mutedXs}>Online</span></row>
                    <row gap="2" align="center"><span dot tone="warning" pulse></span><span sx={mutedXs}>Degraded</span></row>
                    <row gap="2" align="center"><span dot tone="destructive"></span><span sx={mutedXs}>Down</span></row>
                  </column>
                </else-if><else-if {e.tag === 'card'}>
                  <card interactive sx={style.pad(3).w(full)}>
                    <strong>Harbor Signals</strong>
                    <p sx={mutedXs}>4 tracks · last played today</p>
                  </card>
                </else-if><else-if {e.tag === 'avatar'}>
                  <row gap="2" align="center">
                    <avatar name="Ada Lovelace" src="https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=128&h=128&fit=crop&auto=format&q=80"></avatar>
                    <avatar name="Grace Hopper" src="https://images.unsplash.com/photo-1438761681033-6461ffad8d80?w=128&h=128&fit=crop&auto=format&q=80" tone="accent"></avatar>
                    <avatar initials="SK" size="sm"></avatar>
                  </row>
                </else-if><else-if {e.tag === 'icon'}>
                  <row gap="3" align="center">
                    <icon name="check"></icon>
                    <icon name="search"></icon>
                    <icon name="settings"></icon>
                  </row>
                </else-if><else-if {e.tag === 'separator'}>
                  <column gap="2" w="full">
                    <p sx={mutedXs}>Section A</p>
                    <separator w="full"></separator>
                    <p sx={mutedXs}>Section B</p>
                    <row gap="2" align="center">
                      <span sx={mutedXs}>Left</span>
                      <separator orientation="vertical"></separator>
                      <span sx={mutedXs}>Right</span>
                    </row>
                  </column>
                </else-if><else-if {e.tag === 'details'}>
                  <details w="full">
                    <summary>Advanced options</summary>
                    <p sx={mutedXs}>Native disclosure — no JS controller required.</p>
                  </details>
                </else-if><else-if {e.tag === 'collapsible'}>
                  <collapsible w="full">
                    <span slot="trigger" sx={style.fontSize(sm).fontWeight(semibold)}>Webhook endpoints</span>
                    <p sx={mutedXs}>POST /v1/hooks · signed with your workspace secret.</p>
                  </collapsible>
                </else-if><else-if {e.tag === 'prose'}>
                  <prose><p>Body type with <strong>emphasis</strong>, <code>inline()</code>, and lists — no class soup.</p></prose>
                </else-if><else-if {e.tag === 'tooltip'}>
                  <!-- placement bottom so the bubble stays inside the tile band -->
                  <tooltip content="Copy URL" placement="bottom">
                    <span sx={style.fontSize(sm).fg(muted-foreground).borderBWidth(1).borderB(border).border(dashed).cursor(help)}>ship.dev/d/ab12</span>
                  </tooltip>
                </else-if><else-if {e.tag === 'hover-card'}>
                  <column gap="2" align="center">
                    <a href="#" popovertarget="grid-hc" sx={style.fontSize(sm).fg(primary)}>@ada</a>
                    <hover-card id="grid-hc">
                      <row gap="2" align="center">
                        <avatar name="Ada Lovelace" src="https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=64&h=64&fit=crop&auto=format&q=80" size="sm"></avatar>
                        <column gap="0">
                          <strong>Ada Lovelace</strong>
                          <p sx={mutedXs}>Mathematician · first programmer</p>
                        </column>
                      </row>
                    </hover-card>
                  </column>
                </else-if><else-if {e.tag === 'popover'}>
                  <column gap="2" align="center">
                    <button size="sm" variant="outline" popovertarget="grid-pop">Filters</button>
                    <popover id="grid-pop">
                      <column gap="2">
                        <p sx={style.fontSize(sm).fontWeight(medium)}>Status</p>
                        <label sx={style.fontSize(sm)}><input type="checkbox" checked /> Healthy</label>
                        <label sx={style.fontSize(sm)}><input type="checkbox" /> Failed</label>
                      </column>
                    </popover>
                  </column>
                </else-if><else-if {e.tag === 'dropdown'}>
                  <column gap="2" align="center">
                    <button size="sm" variant="outline" popovertarget="grid-dd">Row actions ▾</button>
                    <dropdown id="grid-dd">
                      <menu-item value="edit">Edit</menu-item>
                      <menu-item value="duplicate">Duplicate</menu-item>
                      <menu-item value="archive">Archive</menu-item>
                    </dropdown>
                  </column>
                </else-if><else-if {e.tag === 'context-menu'}>
                  <column gap="2" align="center">
                    <p sx={style.fontSize(sm).fg(muted-foreground).pad(3).borderWidth(1).border(border).border(dashed).radius(md)} id="grid-cm-target">Right-click this card</p>
                    <context-menu for="grid-cm-target" id="grid-cm">
                      <menu-item value="copy">Copy id</menu-item>
                      <menu-item value="paste">Paste</menu-item>
                      <menu-item value="delete">Delete</menu-item>
                    </context-menu>
                  </column>
                </else-if><else-if {e.tag === 'dialog'}>
                  <column gap="2" align="center">
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
                  </column>
                </else-if><else-if {e.tag === 'tabs'}>
                  <tabs w="full">
                    <tab-list>
                      <tab>Overview</tab>
                      <tab>Metrics</tab>
                    </tab-list>
                    <tab-panel><p sx={mutedXs}>Uptime 99.98% · 12 deploys this week.</p></tab-panel>
                    <tab-panel><p sx={mutedXs}>p95 latency 42ms.</p></tab-panel>
                  </tabs>
                </else-if><else-if {e.tag === 'breadcrumb'}>
                  <breadcrumb>
                    <a href="/components">components</a>
                    <a href="/components">inputs</a>
                    <small>field</small>
                  </breadcrumb>
                </else-if><else-if {e.tag === 'pagination'}>
                  <pagination value="2" aria-label="Pages">
                    <page value="1">1</page>
                    <page value="2">2</page>
                    <page value="3">3</page>
                  </pagination>
                </else-if><else-if {e.tag === 'navbar'}>
                  <navbar bordered w="full">
                    <strong slot="brand" sx={style.fontSize(sm)}>Harbor</strong>
                    <span sx={style.padX(2).padY(1).radius(md).fontSize(sm).fg(muted-foreground)}>Docs</span>
                    <span sx={style.padX(2).padY(1).radius(md).fontSize(sm).fontWeight(medium).bg(muted)}>App</span>
                  </navbar>
                </else-if><else-if {e.tag === 'table'}>
                  <table w="full">
                    <thead><tr><th>Service</th><th>Status</th></tr></thead>
                    <tbody>
                      <tr><td>api</td><td>healthy</td></tr>
                      <tr><td>worker</td><td>degraded</td></tr>
                    </tbody>
                  </table>
                </else-if><else-if {e.tag === 'layout'}>
                  <row gap="2" w="full">
                    <div grow sx={style.pad(2).bg(muted).radius(sm).fontSize(xs).fontFamily(mono)}>main</div>
                    <div sx={style.pad(2).bg(muted).radius(sm).fontSize(xs).fontFamily(mono).w(16)}>aside</div>
                  </row>
                </else-if><else-if {e.tag === 'scroll-area'}>
                  <scroll-area sx={style.h(32).w(full).minH(32).borderWidth(1).border(border).radius(md).pad(3).bg(surface-0)}>
                    <column gap="2">
                      <p sx={mutedXs}>Build 1842 · shipped · 14:02</p>
                      <p sx={mutedXs}>Build 1841 · shipped · 11:40</p>
                      <p sx={mutedXs}>Build 1840 · failed · 09:12</p>
                      <p sx={mutedXs}>Build 1839 · shipped · yesterday</p>
                      <p sx={mutedXs}>Build 1838 · shipped · yesterday</p>
                      <p sx={mutedXs}>Build 1837 · shipped · Mon</p>
                    </column>
                  </scroll-area>
                </else-if><else-if {e.tag === 'sail-sparkline'}>
                  <sail-sparkline data="2,5,3,8,4,6,7" width="120" height="40"></sail-sparkline>
                </else-if><else-if {e.tag === 'sail-area-chart'}>
                  <sail-area-chart series='[{"name":"a","data":[2,4,3,5]}]' labels="Q1,Q2,Q3,Q4" width="160" height="60"></sail-area-chart>
                </else-if><else-if {e.tag === 'sail-bar-chart'}>
                  <sail-bar-chart data="2,5,3,8,4" labels="A,B,C,D,E" width="160" height="60"></sail-bar-chart>
                </else-if><else-if {e.tag === 'sail-donut'}>
                  <sail-donut data='[{"label":"a","value":40},{"label":"b","value":30},{"label":"c","value":30}]' diameter="80" thickness="10"></sail-donut>
                </else-if><else-if {e.tag === 'sail-gauge'}>
                  <sail-gauge value="72" diameter="80" thickness="10"></sail-gauge>
                </else-if><else-if {e.tag === 'sail-heatmap'}>
                  <sail-heatmap rows="3" cols="5" values="1,2,3,4,3,4,5,6,5,4,3,2,1,2,3" cell-size="18" gap="3"></sail-heatmap>
                </else-if><else-if {e.tag === 'sail-line-chart'}>
                  <sail-line-chart series='[{"name":"a","data":[2,4,3,5]}]' labels="Q1,Q2,Q3,Q4" width="160" height="60"></sail-line-chart>
                </else-if><else-if {e.tag === 'sail-media-image'}>
                  <div sx={mediaFrame}>
                    <sail-media-image alt="Mountain lake at dusk" fit="cover" zoomable={false} sx={mediaStage}></sail-media-image>
                  </div>
                </else-if><else-if {e.tag === 'sail-media-av'}>
                  <sail-media-av></sail-media-av>
                </else-if><else-if {e.tag === 'sail-code-editor'}>
                  <sail-code-editor value={editorSample()} language="javascript" line-numbers></sail-code-editor>
                </else-if><else-if {e.tag === 'sail-mermaid'}>
                  <sail-mermaid value="graph TD; A[Idea]-->B[Build]; B-->C[Ship]"></sail-mermaid>
                </else-if><else-if {e.tag === 'sail-message'}>
                  <column gap="2" w="full">
                    <sail-message role="user" status="sent">Can you summarize yesterday's deploy?</sail-message>
                    <sail-message role="assistant" status="sent">v0.4.2 went out at 14:02 — 3 services, zero rollbacks.</sail-message>
                  </column>
                </else-if><else-if {e.tag === 'sail-citation'}>
                  <sail-citation source="docs.ship.dev" href="https://docs.ship.dev">Deploy pipeline overview</sail-citation>
                </else-if><else-if {e.tag === 'sail-composer'}>
                  <sail-composer placeholder="Message the agent…" w="full" sx={style.w(full).minW(0)}></sail-composer>
                </else-if><else-if {e.tag === 'sail-thinking-dots'}>
                  <row gap="2" align="center">
                    <sail-thinking-dots></sail-thinking-dots>
                    <span sx={mutedXs}>Agent is thinking…</span>
                  </row>
                </else-if><else-if {e.tag === 'sail-tool-call'}>
                  <sail-tool-call name="list_deploys" status="ok" open w="full">
                    {toolArgs()}
                    <span slot="result">{toolResult()}</span>
                  </sail-tool-call>
                </else-if><else>
                  <p sx={dormant}>{e.tag}</p>
                </else></if>
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
