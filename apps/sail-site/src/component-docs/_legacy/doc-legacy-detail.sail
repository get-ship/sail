<!--
  <doc-legacy-detail> — residual docs host for packages without a first-class
  doc-<slug> host yet. Full configurators live in first-class packages
  (doc-button, doc-badge, doc-fieldset); peel remaining stages out of here
  into component-docs/<slug>/ over time.

  Specimens are intentional live demos (hub-quality), not random float.
  Props/usage come from DETAILS + pack meta. Viewer variant chips drive tags
  that still expose a variantAttr in DETAILS.
-->

<controller>
import type { Scope } from './doc-legacy-detail.sail'
import { DETAILS, type DetailEntry } from '../../compositions/component-details'
import {
  DEMO_PHOTO,
  PACK_PROPS,
  PACK_TAGS,
  packBlurbFor,
  packLabelFor,
  packSlotCopy,
} from '../../compositions/component-pack-meta'
import { docMeta } from '../../compositions/component-docs-registry'

export default controller((ctx: Scope) => {
  const tag = derived(() => {
    const fromProp = ctx.docTag?.() ?? ''
    if (fromProp) return String(fromProp)
    if (typeof location !== 'undefined') {
      const m = location.pathname.match(/^\/components\/([^/]+)/)
      if (m?.[1]) return decodeURIComponent(m[1])
    }
    return ''
  })

  const meta = derived(() => docMeta(tag()))
  const entry = derived<DetailEntry | null>(() => DETAILS[tag()] ?? null)
  const knownPack = derived(() => PACK_TAGS.has(tag()))

  const variant = signal('')
  effect(() => {
    const e = entry()
    const v = e?.variants?.[0] ?? ''
    variant.set(v)
  })
  const onVariantChange = (detail: { value?: string } | string): void => {
    const v = typeof detail === 'string' ? detail : (detail?.value ?? '')
    variant.set(v)
  }

  // Interactive demo state shared across specimens.
  const email = signal('ada@ship.dev')
  const agreed = signal(true)
  const notify = signal(true)
  const volume = signal(60)
  const team = signal('Design')
  const plan = signal('pro')
  const bio = signal('')
  const name = signal('')
  const fw = signal('')
  const pickFw = (detail: { value: string }): void => fw.set(detail?.value ?? '')
  const pageNo = signal('2')
  const goPage = (detail: { page?: string; value?: string }): void =>
    pageNo.set(detail?.page ?? detail?.value ?? '1')
  const kept = signal(true)
  const dropChip = (): void => kept.set(false)
  const presses = signal(0)
  const press = (): void => presses.set((n) => n + 1)
  let toastSeq = 0
  const toastItems = signal<{ key: string }[]>([{ key: 'toast-0' }])
  const pushToast = (): void => {
    toastSeq += 1
    toastItems.set([{ key: `toast-${toastSeq}` }])
  }
  const onToastDismiss = (): void => {
    toastItems.set([])
  }
  effect(() => {
    if (tag() === 'toast') {
      toastSeq = 0
      toastItems.set([{ key: 'toast-0' }])
    }
  })

  // Pack media identity prop.
  effect(() => {
    void tag()
    requestAnimationFrame(() => {
      document.querySelectorAll('sail-media-image').forEach((el) => {
        const host = el as HTMLElement & { src?: string }
        if (host.src !== DEMO_PHOTO) host.src = DEMO_PHOTO
      })
    })
  })

  const heading = () => meta()?.heading ?? entry()?.heading ?? tag()
  const blurb = () => meta()?.blurb ?? entry()?.blurb ?? packBlurbFor(tag())
  const props = () => entry()?.props ?? []
  const usageCode = () => entry()?.usage ?? ''
  const packBlurb = () => packBlurbFor(tag())
  const packProps = () => PACK_PROPS[tag()] ?? []
  const packLabel = () => packLabelFor(tag())
  const slotCopy = () => (knownPack() ? packSlotCopy(tag()) : '')
  // Hide chips when package meta says stage owns knobs (first-class) or empty variants.
  const stageOwns = () => meta()?.stageOwnsKnobs === true
  const viewerVariants = () => (stageOwns() ? [] : entry()?.variants ?? [])
  const viewerVariantAttr = () => (stageOwns() ? '' : entry()?.variantAttr ?? '')

  // Per-tag stage flags — keep separate <if> blocks (no mega nested soup).
  const is = (slug: string) => () => tag() === slug
  const stage = {
    a: is('a'),
    input: is('input'),
    textarea: is('textarea'),
    select: is('select'),
    combobox: is('combobox'),
    checkbox: is('checkbox'),
    radio: is('radio'),
    radioGroup: is('radio-group'),
    switchToggle: is('switch'),
    slider: is('slider'),
    toggleGroup: is('toggle-group'),
    field: is('field'),
    label: is('label'),
    dropzone: is('dropzone'),
    themePicker: is('theme-picker'),
    alert: is('alert'),
    progressBar: is('progress'),
    meter: is('meter'),
    spinner: is('spinner'),
    skeleton: is('skeleton'),
    emptyState: is('empty-state'),
    errorState: is('error-state'),
    errorBoundary: is('error-boundary'),
    toast: is('toast'),
    chip: is('chip'),
    card: is('card'),
    avatar: is('avatar'),
    icon: is('icon'),
    separator: is('separator'),
    kbd: is('kbd'),
    codeBlock: is('code-block'),
    accordion: is('accordion'),
    prose: is('prose'),
    tooltip: is('tooltip'),
    hoverCard: is('hover-card'),
    popover: is('popover'),
    dropdown: is('dropdown'),
    contextMenu: is('context-menu'),
    dialog: is('dialog'),
    tabs: is('tabs'),
    breadcrumb: is('breadcrumb'),
    pagination: is('pagination'),
    navbar: is('navbar'),
    table: is('table'),
    layout: is('layout'),
    scrollArea: is('scroll-area'),
    ticker: is('ticker'),
    // packs
    sailAreaChart: is('sail-area-chart'),
    sailBarChart: is('sail-bar-chart'),
    sailDonut: is('sail-donut'),
    sailGauge: is('sail-gauge'),
    sailHeatmap: is('sail-heatmap'),
    sailLineChart: is('sail-line-chart'),
    sailSparkline: is('sail-sparkline'),
    sailMediaImage: is('sail-media-image'),
    sailMediaAv: is('sail-media-av'),
    sailMessage: is('sail-message'),
    sailCitation: is('sail-citation'),
    sailComposer: is('sail-composer'),
    sailThinkingDots: is('sail-thinking-dots'),
    sailToolCall: is('sail-tool-call'),
    sailMermaid: is('sail-mermaid'),
    sailCodeEditor: is('sail-code-editor'),
  }

  const people = () => [
    { id: 1, name: 'Ada Lovelace', team: 'Engines', status: 'active' },
    { id: 2, name: 'Grace Hopper', team: 'Compilers', status: 'active' },
    { id: 3, name: 'Alan Turing', team: 'Research', status: 'away' },
  ]
  const sheetSide = () => (variant() === 'sheet' ? 'end' : null)

  return {
    tag,
    heading,
    blurb,
    props,
    usageCode,
    knownPack,
    packBlurb,
    packProps,
    packLabel,
    slotCopy,
    viewerVariants,
    viewerVariantAttr,
    variant,
    onVariantChange,
    stage,
    email,
    agreed,
    notify,
    volume,
    team,
    plan,
    bio,
    name,
    fw,
    pickFw,
    pageNo,
    goPage,
    kept,
    dropChip,
    presses,
    press,
    toastItems,
    pushToast,
    onToastDismiss,
    people,
    sheetSide,
  }
})
</controller>

<scope>
  props: {
    docTag = ''
  }

  style page = { box: maxW(breakpoint-desktop).mx(auto).w(full) }
  style monoXs = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs) }
  style stageCol = { box: w(panel-max-w-sm).maxW(full) }
  style stageColMd = { box: w(panel-max-w-md).maxW(full) }
  style fullW = { box: w(full) }
  style mediaStage = { box: w(full).maxW(panel-max-w-sm).h(48).minH(48).overflow(hidden), paint: radius(md) }
  style fullMedia = { box: w(full).h(full) }
  style ticker = { box: w(full).maxW(panel-max-w-md).overflow(hidden).padY(3), paint: bg(muted).radius(md) }
  style tickerText = { box: padX(4), paint: fg(muted-foreground), text: fontSize(sm).fontFamily(mono).whitespace(nowrap) }
  style scrollDemo = { box: maxH(panel-max-h-sm).w(full).maxW(panel-max-w-sm).pad(3), paint: borderWidth(1).border(border-subtle).radius(md) }
  style centerBox = { box: pad(4), paint: borderWidth(1).border(border-subtle).radius(md) }
  style pressBtn = { animate: press({ scale: 'press' }) }
  style toastStage = { box: w(full).maxW(panel-max-w-sm) }
  style sepVRow = { box: h(10) }
  style note = { paint: fg(muted-foreground), text: fontSize(xs) }
</scope>

<column gap="4" style(page)>
  <sail-component-viewer
    name={heading()}
    blurb={knownPack() ? packBlurb() : blurb()}
    pack={knownPack() ? packLabel() : 'Component primitives'}
    variant-attr={viewerVariantAttr()}
    usage={usageCode()}
    variants={viewerVariants()}
    rows={knownPack() ? packProps() : props()}
    on-variant-change={onVariantChange}
  >
    <column slot="stage" gap="4" align="stretch" style({ box: w(full).minW(0) })>
      <if {stage.a()}>
        <column gap="3" align="center">
          <row gap="2" align="center">
            <a href="/docs">text link</a>
            <a href="/components" variant={variant() || 'solid'} tone="primary">Open catalog</a>
            <a href="/docs" variant="outline">Read docs</a>
          </row>
          <p style(monoXs)>bare = text link · variant set = button-link</p>
        </column>
      </if>
      <if {stage.input()}>
        <column gap="3" style(stageCol)>
          <input type="email" placeholder="you@company.com" bind:value={email} />
          <row gap="3" align="center">
            <label style(mutedSm)><input type="checkbox" bind:checked={agreed} /> agree</label>
            <input type="range" min="0" max="100" bind:value={volume} style(fullW) />
          </row>
          <p style(monoXs)>{email()} · check {agreed() ? 'on' : 'off'} · {volume()}</p>
        </column>
      </if>
      <if {stage.textarea()}>
        <column gap="2" style(stageColMd)>
          <textarea rows="3" placeholder="Tell us about yourself" bind:value={bio}></textarea>
          <p style(monoXs)>{bio().length} characters · component.field.*</p>
        </column>
      </if>
      <if {stage.select()}>
        <column gap="2" style(stageColMd)>
          <select bind:value={team}>
            <option>Engineering</option>
            <option>Design</option>
            <option>Research</option>
          </select>
          <p style(monoXs)>team: {team()}</p>
        </column>
      </if>
      <if {stage.combobox()}>
        <column gap="2" style(stageColMd)>
          <combobox placeholder="Pick a framework — type to filter" on-select={pickFw}>
            <option value="sail">Sail</option>
            <option value="react">React</option>
            <option value="svelte">Svelte</option>
            <option value="solid">Solid</option>
          </combobox>
          <p style(monoXs)>picked: {fw() || 'nothing yet'}</p>
        </column>
      </if>
      <if {stage.checkbox()}>
        <column gap="2" align="start" style(stageCol)>
          <label style(mutedSm)>
            <input type="checkbox" bind:checked={agreed} /> Email me product updates
          </label>
          <p style(note)>Catalog alias — <code>input type=checkbox</code>, not a separate tag.</p>
        </column>
      </if>
      <if {stage.radio()}>
        <column gap="2">
          <radio value="pro">Pro — team seats</radio>
          <radio value="free" disabled>Free (disabled)</radio>
          <p style(note)>Group wiring lives on <code>radio-group</code>.</p>
        </column>
      </if>
      <if {stage.radioGroup()}>
        <radio-group label="Plan" name="plan" value={plan()}>
          <radio value="free">Free</radio>
          <radio value="pro">Pro</radio>
          <radio value="enterprise">Enterprise</radio>
        </radio-group>
      </if>
      <if {stage.switchToggle()}>
        <row gap="3" align="center">
          <switch bind:checked={notify}></switch>
          <p style(mutedSm)>{notify() ? 'on' : 'off'}</p>
        </row>
      </if>
      <if {stage.slider()}>
        <column gap="2" style(stageColMd)>
          <row justify="between"><span style(mutedSm)>Volume</span><span style(monoXs)>{volume()}%</span></row>
          <slider bind:value={volume} min="0" max="100"></slider>
        </column>
      </if>
      <if {stage.toggleGroup()}>
        <toggle-group value="day" label="Range">
          <toggle value="day">Day</toggle>
          <toggle value="week">Week</toggle>
          <toggle value="month">Month</toggle>
        </toggle-group>
        <p style(note)>Segmented control — <code>toggle</code> cells, not a hub tile of their own.</p>
      </if>
      <if {stage.field()}>
        <field style(stageCol)>
          <label slot="label">Email</label>
          <input type="email" placeholder="you@ship.dev" bind:value={email} />
          <span slot="hint" style(mutedXs)>We will never share this.</span>
        </field>
      </if>
      <if {stage.label()}>
        <column gap="1" style(stageCol)>
          <label for="legacy-label-input" required>Display name</label>
          <input id="legacy-label-input" size="sm" placeholder="Ada Lovelace" bind:value={name} />
        </column>
      </if>
      <if {stage.dropzone()}>
        <card dropzone style({ box: pad(6).w(full).maxW(panel-max-w-sm) })>
          Drop a file, or click to browse
        </card>
        <p style(note)>Action / attribute — not an L1 host tag.</p>
      </if>
      <if {stage.themePicker()}>
        <theme-picker></theme-picker>
      </if>
      <if {stage.alert()}>
        <alert tone="info" style(stageColMd)>
          <strong slot="title">Heads up</strong>
          A new version is available.
        </alert>
      </if>
      <if {stage.progressBar()}>
        <column gap="2" style(stageColMd)>
          <progress value={volume()} max="100"></progress>
          <p style(monoXs)>{volume()}% · omit value for indeterminate</p>
        </column>
      </if>
      <if {stage.meter()}>
        <column gap="2" style(stageColMd)>
          <meter value={volume()} min="0" max="100" low="33" high="66" optimum="100"></meter>
          <p style(monoXs)>value {volume()}</p>
        </column>
      </if>
      <if {stage.spinner()}>
        <row gap="2" align="center">
          <spinner tone="primary"></spinner>
          <p style(mutedSm)>fetching…</p>
        </row>
      </if>
      <if {stage.skeleton()}>
        <column gap="2" style(stageCol)>
          <skeleton shape="text" size="md"></skeleton>
          <skeleton shape="text" size="sm"></skeleton>
        </column>
      </if>
      <if {stage.emptyState()}>
        <empty-state style(stageColMd)>
          <icon slot="icon" name="inbox"></icon>
          <strong slot="title">No results</strong>
          Try a different search, or clear a filter.
          <button slot="action" variant="outline" size="sm">Clear filters</button>
        </empty-state>
      </if>
      <if {stage.errorState()}>
        <error-state code="ERR_TIMEOUT" style(stageColMd)>
          <strong slot="title">Could not load your data</strong>
          The request failed after 3 retries.
          <button slot="action" type="button" variant="solid" tone="destructive" size="sm">Retry</button>
        </error-state>
      </if>
      <if {stage.errorBoundary()}>
        <error-boundary>
          <p style(mutedSm)>Child tree isolated from render faults.</p>
        </error-boundary>
      </if>
      <if {stage.toast()}>
        <column gap="3" align="center" style(toastStage)>
          <button type="button" variant="outline" size="sm" on:click={pushToast}>Show toast</button>
          <for {toastItems() as t} key={t.key}>
            <toast tone="primary" duration="0" on-dismiss={onToastDismiss} style(stageCol)>
              <strong>Saved.</strong> Sticky at duration 0.
            </toast>
          </for>
        </column>
      </if>
      <if {stage.chip()}>
        <row gap="2" align="center">
          <if {kept()}>
            <chip variant="solid" tone="primary" removable on-remove={dropChip}>typescript</chip>
          </if>
          <chip variant="outline" tone="muted">a11y</chip>
          <chip variant="outline">ssr</chip>
        </row>
      </if>
      <if {stage.card()}>
        <card style({ box: pad(4).w(full).maxW(panel-max-w-sm) })>
          <strong>Harbor Signals</strong>
          <p style(mutedXs)>4 tracks · last played today</p>
        </card>
      </if>
      <if {stage.avatar()}>
        <row gap="2" align="center">
          <avatar name="Ada Lovelace"></avatar>
          <avatar name="Grace Hopper" tone="accent"></avatar>
          <avatar initials="SK" size="sm"></avatar>
        </row>
      </if>
      <if {stage.icon()}>
        <row gap="3" align="center">
          <icon name="check"></icon>
          <icon name="search"></icon>
          <icon name="settings"></icon>
          <icon name="sparkles" size="lg"></icon>
        </row>
      </if>
      <if {stage.separator()}>
        <column gap="2" style(stageCol)>
          <p style(mutedSm)>Section A</p>
          <separator></separator>
          <p style(mutedSm)>Section B</p>
          <row gap="3" align="center" style(sepVRow)>
            <span style(mutedSm)>Left</span>
            <separator orientation="vertical"></separator>
            <span style(mutedSm)>Right</span>
          </row>
        </column>
      </if>
      <if {stage.kbd()}>
        <row gap="1" align="center">
          <kbd>Ctrl</kbd>
          <span style(mutedXs)>+</span>
          <kbd>K</kbd>
        </row>
      </if>
      <if {stage.codeBlock()}>
        <code-block
          code={'const n = signal(0)\nconst double = derived(() => n() * 2)'}
          lang="ts"
          style(stageColMd)
        ></code-block>
      </if>
      <if {stage.accordion()}>
        <accordion style(stageColMd)>
          <span slot="trigger">Webhook endpoints</span>
          <p style(mutedSm)>POST /v1/hooks · signed with your workspace secret.</p>
        </accordion>
      </if>
      <if {stage.prose()}>
        <prose style(stageColMd)>
          <h2>Prose scope</h2>
          <p>Body type with <strong>emphasis</strong>, <code>inline()</code>, and lists.</p>
        </prose>
      </if>
      <if {stage.tooltip()}>
        <tooltip content="Copy URL" placement="top">
          <span style(mutedSm)>ship.dev/d/ab12</span>
        </tooltip>
      </if>
      <if {stage.hoverCard()}>
        <column gap="2" align="center">
          <a href="#" popovertarget="legacy-hc">@ada</a>
          <hover-card id="legacy-hc">
            <strong>Ada Lovelace</strong>
            <p style(mutedXs)>Mathematician · first programmer</p>
          </hover-card>
        </column>
      </if>
      <if {stage.popover()}>
        <column gap="2" align="center">
          <button size="sm" variant="outline" popovertarget="legacy-pop">Filters</button>
          <popover id="legacy-pop">
            <column gap="2">
              <p style(mutedSm)>Status</p>
              <label style(mutedSm)><input type="checkbox" checked /> Healthy</label>
            </column>
          </popover>
        </column>
      </if>
      <if {stage.dropdown()}>
        <column gap="2" align="center">
          <button size="sm" variant="outline" popovertarget="legacy-dd">Row actions ▾</button>
          <dropdown id="legacy-dd">
            <menu-item value="edit">Edit</menu-item>
            <menu-item value="duplicate">Duplicate</menu-item>
            <menu-item value="archive">Archive</menu-item>
          </dropdown>
        </column>
      </if>
      <if {stage.contextMenu()}>
        <column gap="2" align="center">
          <p id="legacy-cm-target" style(centerBox)>Right-click this card</p>
          <context-menu for="legacy-cm-target" id="legacy-cm">
            <menu-item value="copy">Copy id</menu-item>
            <menu-item value="delete">Delete</menu-item>
          </context-menu>
        </column>
      </if>
      <if {stage.dialog()}>
        <column gap="2" align="center">
          <button size="sm" tone="primary" command="show-modal" commandfor="legacy-dlg">Open dialog</button>
          <dialog id="legacy-dlg" side={sheetSide()}>
            <header>Invite to workspace</header>
            <section>
              <field>
                <label slot="label">Email</label>
                <input type="email" placeholder="name@company.com" />
              </field>
            </section>
            <footer>
              <button command="close" commandfor="legacy-dlg" variant="ghost" size="sm">Cancel</button>
              <button command="close" commandfor="legacy-dlg" variant="solid" tone="primary" size="sm">Send</button>
            </footer>
          </dialog>
          <p style(note)>variant chips: modal · sheet</p>
        </column>
      </if>
      <if {stage.tabs()}>
        <tabs style(stageColMd)>
          <tab-list variant={variant() || 'line'}>
            <tab value="one">Overview</tab>
            <tab value="two">Activity</tab>
            <tab value="three">Settings</tab>
          </tab-list>
          <tab-panel value="one"><p style(mutedSm)>Overview panel</p></tab-panel>
          <tab-panel value="two"><p style(mutedSm)>Activity panel</p></tab-panel>
          <tab-panel value="three"><p style(mutedSm)>Settings panel</p></tab-panel>
        </tabs>
      </if>
      <if {stage.breadcrumb()}>
        <breadcrumb>
          <a href="/components">components</a>
          <a href="/docs">docs</a>
          <small>current</small>
        </breadcrumb>
      </if>
      <if {stage.pagination()}>
        <column gap="2" align="center">
          <pagination value={pageNo()} on-page-change={goPage}>
            <page value="1">1</page>
            <page value="2">2</page>
            <page value="3">3</page>
          </pagination>
          <p style(monoXs)>page {pageNo()} · <code>page</code> cells are not a hub tile</p>
        </column>
      </if>
      <if {stage.navbar()}>
        <navbar placement="top" bordered style(fullW)>
          <strong slot="brand">Ship</strong>
          <a href="/components">Components</a>
          <a href="/docs">Docs</a>
          <button slot="actions" size="sm" variant="outline">Sign in</button>
        </navbar>
      </if>
      <if {stage.table()}>
        <table style(stageColMd)>
          <thead>
            <tr><th>Name</th><th>Team</th><th>Status</th></tr>
          </thead>
          <tbody>
            <for {people() as p} key={p.id}>
              <tr>
                <td>{p.name}</td>
                <td>{p.team}</td>
                <td>{p.status}</td>
              </tr>
            </for>
          </tbody>
        </table>
      </if>
      <if {stage.layout()}>
        <column gap="3" style(stageColMd)>
          <row gap="2">
            <card style({ box: pad(3).grow(1) })>row</card>
            <card style({ box: pad(3).grow(1) })>gap</card>
          </row>
          <grid cols="2" gap="2">
            <card style({ box: pad(3) })>grid</card>
            <card style({ box: pad(3) })>cols=2</card>
          </grid>
          <center style(centerBox)>center</center>
          <p style(note)>Keywords — not a SOURCES host. row · column · grid · center.</p>
        </column>
      </if>
      <if {stage.scrollArea()}>
        <scroll-area orientation="vertical" style(scrollDemo)>
          <p style(mutedSm)>Line 1 — overflow content</p>
          <p style(mutedSm)>Line 2</p>
          <p style(mutedSm)>Line 3</p>
          <p style(mutedSm)>Line 4</p>
          <p style(mutedSm)>Line 5</p>
          <p style(mutedSm)>Line 6</p>
          <p style(mutedSm)>Line 7</p>
          <p style(mutedSm)>Line 8</p>
        </scroll-area>
      </if>
      <if {stage.ticker()}>
        <ticker duration="normal" style(ticker)>
          <p style(tickerText)>tokens · signals · a11y · hot reload · SSR ·&nbsp;</p>
        </ticker>
      </if>

      <!-- Pack specimens (when pack tag is known) -->
      <if {stage.sailAreaChart()}>
        <sail-area-chart style(stageColMd)></sail-area-chart>
      </if>
      <if {stage.sailBarChart()}>
        <sail-bar-chart style(stageColMd)></sail-bar-chart>
      </if>
      <if {stage.sailDonut()}>
        <sail-donut></sail-donut>
      </if>
      <if {stage.sailGauge()}>
        <sail-gauge></sail-gauge>
      </if>
      <if {stage.sailHeatmap()}>
        <sail-heatmap style(stageColMd)></sail-heatmap>
      </if>
      <if {stage.sailLineChart()}>
        <sail-line-chart style(stageColMd)></sail-line-chart>
      </if>
      <if {stage.sailSparkline()}>
        <sail-sparkline></sail-sparkline>
      </if>
      <if {stage.sailMediaImage()}>
        <div style(mediaStage)>
          <sail-media-image style(fullMedia)></sail-media-image>
        </div>
      </if>
      <if {stage.sailMediaAv()}>
        <sail-media-av style(stageColMd)></sail-media-av>
      </if>
      <if {stage.sailMessage()}>
        <sail-message style(stageColMd)></sail-message>
      </if>
      <if {stage.sailCitation()}>
        <sail-citation></sail-citation>
      </if>
      <if {stage.sailComposer()}>
        <sail-composer style(stageColMd)></sail-composer>
      </if>
      <if {stage.sailThinkingDots()}>
        <sail-thinking-dots></sail-thinking-dots>
      </if>
      <if {stage.sailToolCall()}>
        <sail-tool-call></sail-tool-call>
      </if>
      <if {stage.sailMermaid()}>
        <sail-mermaid style(stageColMd)></sail-mermaid>
      </if>
      <if {stage.sailCodeEditor()}>
        <sail-code-editor style(stageColMd)></sail-code-editor>
      </if>

      <if {slotCopy()}>
        <p style(monoXs)>{slotCopy()}</p>
      </if>
    </column>
  </sail-component-viewer>

  <row gap="2">
    <a variant="default" href="/components">← All components</a>
    <a variant="solid" tone="primary" href="/docs/reference">API reference</a>
  </row>
</column>
