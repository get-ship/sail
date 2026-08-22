<!--
  <theme-l1-overlay> — Themes → Components live stage: Overlay / nav / data faces.
  Host bind: components={components}. One face visible via stage gates.
-->
<scope>
  style fieldHint = { fg(muted).type(body-sm) }
  // no preset: type weight
  style doorTitle = { type({ weight: semibold }) }
  style iconBtn = { size(control-sm) }
  style tapFloor = { size(control-sm) }
  style stageHost = { size(max) }
  // no preset: size lock
  style sampleMax = { size({ w: 100%, max-w: overlay-max-w-sm }) }
  // no preset: size lock
  style menuItemShell = { size({ w: 100%, max-w: overlay-max-w-sm }).pad(1).border(subtle).radius(md).bg(surface) }
  style ctxBox = { size(max).pad(3).border(subtle).radius(md) }
  // no preset: size lock
  style scrollVert = { size({ w: 100%, max-w: overlay-max-w-sm, max-h: 32 }).pad(3).border(subtle).radius(md) }
  // no preset: size lock
  style scrollHoriz = { size({ w: 100%, max-w: overlay-max-w-sm }).pad(3).border(subtle).radius(md) }
</scope>

<controller>
import type { UiEvent } from '../compositions/ui-event'

type StageBag = {
  stage: Record<string, (() => boolean) | undefined>
  variant: () => string
  sampleText: () => string
  onSampleInput: (e: UiEvent) => void
  load: () => number
  notify: () => boolean
  pageNo: () => string
  onPageChange: (detail: { page?: string }) => void
}

export default controller(() => {
  const bag = signal<StageBag | null>(null)
  mount((host) => {
    const pre = (host as { components?: StageBag | null }).components
    if (pre != null) bag.set(pre)
    Object.defineProperty(host, 'components', {
      configurable: true,
      get: () => bag(),
      set: (s: StageBag | null) => bag.set(s),
    })
  })
  const components = {
    get stage() {
      return new Proxy({} as Record<string, () => boolean>, {
        get(_t, prop: string) {
          if (typeof prop !== 'string') return () => false
          return bag()?.stage?.[prop] ?? (() => false)
        },
      })
    },
    variant: () => bag()?.variant() ?? '',
    sampleText: () => bag()?.sampleText() ?? '',
    onSampleInput: (e: UiEvent) => bag()?.onSampleInput(e),
    load: () => bag()?.load() ?? 40,
    notify: () => bag()?.notify() ?? false,
    pageNo: () => bag()?.pageNo() ?? '1',
    onPageChange: (d: { page?: string }) => bag()?.onPageChange(d),
  }
  return { components }
})
</controller>

<column gap="4" align="start" theme-l1-overlay l1-stage-host style(stageHost)>
<if {components.stage.tooltip()}>
  <tooltip content="Cannot be undone" placement={components.variant() || 'top'}>
    <button type="button" size="sm" variant="outline" tone="danger">Hover me</button>
  </tooltip>
</if>
<if {components.stage.hoverCard()}>
  <column gap="2" align="center">
    <button type="button" size="sm" variant="ghost" popovertarget="theme-l1-hc">@ada</button>
    <hover-card id="theme-l1-hc">
      <column gap="1" style({ pad(3) })>
        <p style(doorTitle)>Ada Lovelace</p>
        <p style(fieldHint)>Hover card body.</p>
      </column>
    </hover-card>
  </column>
</if>
<if {components.stage.popover()}>
  <column gap="2" align="center">
    <button type="button" size="sm" popovertarget="theme-l1-pop">Open popover</button>
    <popover id="theme-l1-pop">
      <p style({ pad(3) })>Popover panel under theme paint.</p>
    </popover>
  </column>
</if>
<if {components.stage.dropdown()}>
  <column gap="2" align="center">
    <button type="button" size="sm" popovertarget="theme-l1-dd">Actions ▾</button>
    <dropdown id="theme-l1-dd">
      <menu-item>Edit</menu-item>
      <menu-item>Duplicate</menu-item>
      <menu-item>Delete</menu-item>
    </dropdown>
  </column>
</if>
<if {components.stage.menuItem()}>
  <!-- no preset: size lock -->
  <column gap="1" style(menuItemShell)>
    <menu-item>Edit theme</menu-item>
    <menu-item>Duplicate</menu-item>
    <menu-item>Export</menu-item>
  </column>
</if>
<if {components.stage.contextMenu()}>
  <column gap="2" align="center" style(sampleMax)>
    <column id="theme-l1-ctx" gap="1" style(ctxBox)>
      <p style(fieldHint)>Right-click this area</p>
    </column>
    <context-menu for="theme-l1-ctx">
      <menu-item value="cut">Cut</menu-item>
      <menu-item value="copy">Copy</menu-item>
      <menu-item value="paste">Paste</menu-item>
    </context-menu>
  </column>
</if>
<if {components.stage.dialog()}>
  <column gap="2" align="center">
    <button type="button" size="sm" command="show-modal" commandfor="theme-l1-dlg">Open dialog</button>
    <dialog id="theme-l1-dlg" size="sm">
      <header>Confirm</header>
      <section>Live dialog under theme paint — Escape closes.</section>
      <footer>
        <button type="button" size="sm" variant="ghost" command="close" commandfor="theme-l1-dlg">Cancel</button>
        <button type="button" size="sm" variant="solid" tone="primary" command="close" commandfor="theme-l1-dlg">OK</button>
      </footer>
    </dialog>
  </column>
</if>

<!-- ── Navigation ─────────────────────────────────── -->
<if {components.stage.tabs()}>
  <tabs style(stageHost)>
    <tab-list variant={components.variant() || 'line'}>
      <tab title="Overview"></tab>
      <tab title="Shape"></tab>
      <tab title="Assets"></tab>
    </tab-list>
    <tab-panel><p style(fieldHint)>Overview panel</p></tab-panel>
    <tab-panel><p style(fieldHint)>Shape panel</p></tab-panel>
    <tab-panel><p style(fieldHint)>Assets panel</p></tab-panel>
  </tabs>
</if>
<if {components.stage.tabList()}>
  <tab-list variant="line">
    <tab title="One"></tab>
    <tab title="Two"></tab>
    <tab title="Three"></tab>
  </tab-list>
</if>
<if {components.stage.tab()}>
  <tab title="Selected tab chrome"></tab>
</if>
<if {components.stage.tabPanel()}>
  <tab-panel>
    <p style(fieldHint)>Tab panel body.</p>
  </tab-panel>
</if>
<if {components.stage.breadcrumb()}>
  <breadcrumb>
    <a href="/themes">Themes</a>
    <a href="#">Studio</a>
    <small>breadcrumb</small>
  </breadcrumb>
</if>
<if {components.stage.pagination()}>
  <pagination value={components.pageNo()} on-page-change={components.onPageChange}>
    <page-item value="1">1</page-item>
    <page-item value="2">2</page-item>
    <page-item value="3">3</page-item>
    <page-item value="4">4</page-item>
  </pagination>
</if>
<if {components.stage.pageItem()}>
  <row gap="1" align="center">
    <page-item value="1">1</page-item>
    <page-item value="2">2</page-item>
    <page-item value="3">3</page-item>
  </row>
</if>
<if {components.stage.page()}>
  <page>
    <h1>Edition</h1>
  </page>
</if>
<if {components.stage.navbar()}>
  <navbar bordered style(stageHost)>
    <strong slot="brand">SHIP</strong>
    <a href="/themes">Themes</a>
    <a href="/themes">Assets</a>
    <button type="button" slot="actions" size="sm" variant="outline">Sign in</button>
  </navbar>
</if>

<!-- ── Data / layout ──────────────────────────────── -->
<if {components.stage.table()}>
  <table dense style(stageHost)>
    <thead>
      <tr><th>Name</th><th>Role</th></tr>
    </thead>
    <tbody>
      <tr><td>Ada</td><td>Engineer</td></tr>
      <tr><td>Grace</td><td>Lead</td></tr>
    </tbody>
  </table>
</if>
<if {components.stage.scrollArea()}>
  <column gap="3" style(sampleMax)>
    <column gap="2">
      <p style(fieldHint)>Vertical · rest thumb is border · hover border-strong</p>
      <!-- no preset: size lock -->
      <scroll-area orientation="vertical" style(scrollVert)>
        <column gap="2">
          <p style(fieldHint)>Scroll me — themed thin scrollbar.</p>
          <p style(fieldHint)>Line 2</p>
          <p style(fieldHint)>Line 3</p>
          <p style(fieldHint)>Line 4</p>
          <p style(fieldHint)>Line 5</p>
          <p style(fieldHint)>Line 6</p>
          <p style(fieldHint)>Line 7</p>
          <p style(fieldHint)>Line 8</p>
        </column>
      </scroll-area>
    </column>
    <column gap="2">
      <p style(fieldHint)>Horizontal · orientation="horizontal"</p>
      <!-- no preset: size lock -->
      <scroll-area orientation="horizontal" style(scrollHoriz)>
        <!-- no preset: type wrap -->
        <p style({ pad(1).type({ wrap: nowrap }) })>
          Wide band that overflows on the x axis → scroll with thin themed thumb
        </p>
      </scroll-area>
    </column>
  </column>
</if>
<if {components.stage.ticker()}>
  <column gap="3" style(sampleMax)>
    <p style(fieldHint)>
      Geometry host + compile-time loop · author paint
    </p>
    <ticker style(sampleMax, { overflow(hidden).pad(2).bg(muted).radius(md) })>
      <span style({ pad(3).fg(muted).type(mono) })>
        Live ticker · muted band ·&nbsp;
      </span>
    </ticker>
    <ticker style(sampleMax, { overflow(hidden).pad(2).bg(primary).radius(md) })>
      <span style({ pad(3).fg(primary-foreground).type(body-sm) })>
        Brand paint ·&nbsp;
      </span>
    </ticker>
  </column>
</if>
<if {components.stage.a()}>
  <row gap="3" align="center" nowrap>
    <a href="#">Text link</a>
    <a href="#" variant="solid" tone="primary" size="sm">Button link</a>
  </row>
</if>
</column>
