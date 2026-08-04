<!--
  Themes → Components facet — component token overrides (not the Components entity page).
  Storybook-ish: catalog rail | large stage | override inspector.
  Defaults resolve via DTCG color paths; overrides write component.* leaves.
-->
<scope>
  const sectionTitle = style.fontSize(sm).fontWeight(semibold)
  const fieldHint = style.fontSize(xs).fg(muted-foreground)
  const pathMeta = style.fontSize(xs).fontFamily(mono).fg(muted-foreground)
  const doorTitle = style.fontWeight(semibold)
  const bandStatus = style.padX(4).padY(2)
  const bandError = style.padX(4).padY(2).borderBWidth(1).borderB(border)
  const railCol = style.padX(3).padT(4).padB(6).minW(0)
  const inspectorCol = style.padX(4).padT(4).padB(8).minW(0)
  const fullBtn = style.w(full)
  const chipCol = style.minW(0)
  // Selected rail row: soft fill + subtle border (no nested cards).
  const listRow = style.padX(2_5).padY(2).radius(md).cursor(pointer).borderWidth(1).border(transparent).when({
    hover: style.bg(hover-bg),
    selected: style.bg(surface-2).border(border-subtle),
  })
  const eyebrow = style.fontSize(xs).fontWeight(semibold).tracking(wide).textCase(uppercase).fg(muted-foreground)
  const tokenStrip = style.fontSize(xs).fontFamily(mono).fg(muted-foreground).padX(2).padY(1).radius(sm).bg(surface-0)
  const tokenRow = style.padY(1_5).borderBWidth(1).borderB(border-subtle)
</scope>

<controller>
import { createThemeSession } from '../theme-session/theme-session'
import { watchPreviewHost } from '../theme-session/bind-preview-host'
import { createComponentsEdit } from './theme-components-edit'
export default controller(() => {
  const session = createThemeSession({
    facet: 'components',
    editGuardMessage: 'Add this theme to the project to edit component tokens.',
    saveSuccessMessage: 'Component overrides saved.',
  })
  const components = createComponentsEdit(session)
  mount((host) => watchPreviewHost(host, (el) => session.attachPreviewHost(el)))
  return { session, components }
})
</controller>

<column
  data-surface="themes"
  data-theme-components-surface
  data-components-chrome
  gap="0"
  grow
  sx={style.minH(0).h(full).minW(0).overflow(hidden).bg(background)}
>
  <theme-facet-chrome session={session}></theme-facet-chrome>

  <if {session.hasError()}>
    <row data-components-error align="center" gap="2" shrink="0" sx={bandError}>
      <p grow min-w="0" sx={style.fontSize(sm).fg(destructive).truncate()}>{session.errorText()}</p>
      <button type="button" size="sm" variant="ghost" data-components-retry on:click={session.onRetry}>Retry</button>
    </row>
  </if>

  <if {session.loading()}>
    <row data-components-loading align="center" gap="2" shrink="0" sx={bandStatus}>
      <spinner size="sm"></spinner>
      <p sx={fieldHint}>Loading…</p>
    </row>
  </if>

  <!-- Storybook-ish: rail | LARGE stage | inspector -->
  <row
    data-components-workbench
    grow
    gap="0"
    nowrap
    align="stretch"
    sx={style.minH(0).basis(0).h(full).minW(0).overflow(hidden).bg(background)}
  >
    <!-- LEFT: component catalog rail -->
    <scroll-area
      data-components-rail
      shrink="0"
      sx={style.minW(panel-min-w-sm).maxW(panel-max-w-sm).w(panel-max-w-sm).minH(0).h(full).borderRWidth(1).borderR(border-subtle).bg(background)}
    >
      <column gap="3" sx={railCol}>
        <column gap="2">
          <p sx={eyebrow}>Components</p>
          <p sx={style.fontSize(md).fontWeight(semibold).tracking(tight)}>Theme overrides</p>
          <p sx={fieldHint} data-l1-facet-blurb>
            Per-element exceptions for this theme — not the project Components library.
          </p>
          <row gap="2" align="center" nowrap>
            <badge tone="muted" size="sm" variant="outline" pill>{components.l1Count()} components</badge>
            <if {components.overrideCount()}>
              <badge tone="primary" size="sm" variant="soft" pill>{components.overrideCount()} overrides</badge>
            </if>
          </row>
          <input
            type="search"
            size="sm"
            data-l1-filter
            value={components.filter()}
            on:input={components.onFilterInput}
            placeholder="Filter components…"
            spellcheck="false"
            aria-label="Filter components"
          />
        </column>
        <column data-l1-list gap="1" on:click={components.onSelect}>
          <for {components.componentRows()} as="row" key={row.key}>
            <row
              data-l1={row.id}
              data-selected={row.selected ? 'true' : 'false'}
              aria-selected={row.selected ? 'true' : 'false'}
              align="center"
              gap="2"
              nowrap
              sx={listRow}
            >
              <column gap="0" grow min-w="0">
                <p sx={doorTitle}>{row.label}</p>
                <p sx={fieldHint}>{row.hint}</p>
              </column>
              <if {row.hasOverride}>
                <badge tone="primary" size="sm" variant="solid" pill shrink="0">override</badge>
              </if>
            </row>
          </for>
        </column>
      </column>
    </scroll-area>

    <!-- CENTER: large live specimen stage -->
    <column
      data-specimen-dock
      data-components-stage
      gap="0"
      grow
      min-w="0"
      sx={style.minW(0).basis(0).minH(0).h(full).bg(background).overflow(hidden)}
    >
      <column grow gap="0" sx={style.minH(0).h(full).pad(4)}>
        <!--
          One themed host only. Stage content sits ON background —
          no nested white plate under the live specimen (light-mode clash).
          Frame chrome shows theme name + mode — no external Preview/light label.
        -->
        <card
          data-components-specimen
          data-preview-specimen
          data-mode={session.paintMode()}
          padding="none"
          sx={style.minW(0).minH(0).h(full).overflow(hidden).shadow(lg)}
        >
          <column gap="0" grow sx={style.minH(0).h(full)}>
            <row
              data-specimen-chrome
              align="center"
              gap="2"
              nowrap
              shrink="0"
              sx={style.padX(3).h(8).minH(8).minW(0).borderBWidth(1).borderB(border-subtle).bg(surface-1)}
            >
              <row gap="2" align="center" nowrap shrink="0" aria-hidden="true" sx={style.minW(12)}>
                <stack shrink="0" sx={style.w(2).h(2).minW(2).minH(2).radius(full).bg(destructive)}></stack>
                <stack shrink="0" sx={style.w(2).h(2).minW(2).minH(2).radius(full).bg(warning)}></stack>
                <stack shrink="0" sx={style.w(2).h(2).minW(2).minH(2).radius(full).bg(success)}></stack>
              </row>
              <p grow min-w="0" sx={style.fontSize(xs).fg(muted-foreground).truncate()}>
                {session.nameEdit()} · {components.selectedLabel()}
              </p>
              <p sx={pathMeta} shrink="0">{session.paintMode()}</p>
            </row>

            <if {components.hasVariants()}>
              <row
                data-specimen-variants
                align="center"
                gap="2"
                nowrap
                shrink="0"
                on:click={components.onVariantPick}
                sx={style.padX(3).padY(2).minW(0).borderBWidth(1).borderB(border-subtle).bg(surface-1)}
              >
                <p sx={pathMeta} shrink="0">{components.viewerVariantAttr()}:</p>
                <for {components.variantChips()} as="c" key={c.key}>
                  <button
                    type="button"
                    size="sm"
                    tone="primary"
                    variant={c.btnVariant}
                    data-variant={c.v}
                  >
                    {c.v}
                  </button>
                </for>
              </row>
            </if>

            <scroll-area grow sx={style.minH(0).basis(0).h(full).bg(background)}>
              <column
                data-specimen-body
                data-l1-live-stage={components.selectedId()}
                gap="5"
                align="stretch"
                sx={style.pad(5).minW(0).minH(full)}
              >
                <column gap="1">
                  <p sx={style.fontSize(md).fontWeight(semibold).fontFamily(heading)}>
                    {components.selectedLabel()}
                  </p>
                  <p sx={style.fontSize(sm).fg(muted-foreground).leading(relaxed).fontFamily(body)}>
                    {components.viewerBlurb()}
                  </p>
                  <p sx={pathMeta}>{components.registryNote()}</p>
                </column>

                <column gap="4" align="start" w="full" data-l1-stage-host>

                  <!-- ── Inputs ─────────────────────────────────────── -->
                  <if {components.stage.button()}>
                    <column gap="3" w="full">
                      <row gap="2" align="center" nowrap>
                        <button type="button" size="sm" variant={components.variant() || 'solid'} tone="primary">Primary</button>
                        <button type="button" size="sm" variant={components.variant() || 'solid'} tone="accent">Accent</button>
                        <button type="button" size="sm" variant={components.variant() || 'outline'} tone="primary">Outline</button>
                        <button type="button" size="sm" variant={components.variant() || 'ghost'} tone="muted">Ghost</button>
                      </row>
                    </column>
                  </if>
                  <if {components.stage.input()}>
                    <column gap="2" w="full" sx={style.maxW(panel-max-w-sm)}>
                      <input type="text" size="sm" placeholder="Sample field" value={components.sampleText()} on:input={components.onSampleInput} aria-label="Sample field" />
                    </column>
                  </if>
                  <if {components.stage.textarea()}>
                    <textarea size="sm" rows="3" placeholder="Multiline" aria-label="Sample textarea" w="full" sx={style.maxW(panel-max-w-sm)}></textarea>
                  </if>
                  <if {components.stage.select()}>
                    <select size="sm" aria-label="Sample select" w="full" sx={style.maxW(panel-max-w-sm)}>
                      <option>Option A</option>
                      <option>Option B</option>
                      <option>Option C</option>
                    </select>
                  </if>
                  <if {components.stage.combobox()}>
                    <combobox placeholder="Type to filter…" w="full" sx={style.maxW(panel-max-w-sm)}>
                      <option value="sail">Sail</option>
                      <option value="react">React</option>
                      <option value="svelte">Svelte</option>
                    </combobox>
                  </if>
                  <if {components.stage.radio()}>
                    <radio value="a">Option A</radio>
                  </if>
                  <if {components.stage.radioGroup()}>
                    <radio-group label="Plan" name="theme-l1-plan" w="full" sx={style.maxW(panel-max-w-sm)}>
                      <radio value="monthly">Monthly</radio>
                      <radio value="yearly">Yearly</radio>
                    </radio-group>
                  </if>
                  <if {components.stage.switchToggle()}>
                    <row gap="3" align="center">
                      <switch size={components.variant() || 'md'} tone="primary" checked={components.notify()} aria-label="Sample switch"></switch>
                      <p sx={fieldHint}>{components.notify() ? 'on' : 'off'}</p>
                    </row>
                  </if>
                  <if {components.stage.slider()}>
                    <column gap="2" w="full" sx={style.maxW(panel-max-w-sm)}>
                      <slider value={components.load()} min="0" max="100" tone={components.variant() || 'primary'} aria-label="Sample slider"></slider>
                      <p sx={fieldHint}>{components.load()}%</p>
                    </column>
                  </if>
                  <if {components.stage.toggle()}>
                    <toggle value="on" size="sm">Toggle</toggle>
                  </if>
                  <if {components.stage.toggleGroup()}>
                    <toggle-group variant={components.variant() || 'outline'}>
                      <toggle value="left">Left</toggle>
                      <toggle value="center">Center</toggle>
                      <toggle value="right">Right</toggle>
                    </toggle-group>
                  </if>
                  <if {components.stage.field()}>
                    <field w="full" sx={style.maxW(panel-max-w-sm)}>
                      <label slot="label" for="theme-l1-field">Name</label>
                      <input id="theme-l1-field" size="sm" placeholder="Ada Lovelace" />
                      <p slot="hint">Hint under the control</p>
                    </field>
                  </if>
                  <if {components.stage.label()}>
                    <column gap="1" w="full" sx={style.maxW(panel-max-w-sm)}>
                      <label for="theme-l1-label" size={components.variant() || 'md'} required>Email</label>
                      <input id="theme-l1-label" size="sm" placeholder="ada@example.com" />
                    </column>
                  </if>
                  <if {components.stage.fieldset()}>
                    <fieldset w="full" sx={style.maxW(panel-max-w-sm)}>
                      <legend>Account</legend>
                      <column gap="2">
                        <input size="sm" placeholder="Name" />
                        <input size="sm" placeholder="Email" />
                      </column>
                    </fieldset>
                  </if>
                  <if {components.stage.legend()}>
                    <fieldset w="full" sx={style.maxW(panel-max-w-sm)}>
                      <legend>Legend specimen</legend>
                      <p sx={fieldHint}>Legend lives on a fieldset.</p>
                    </fieldset>
                  </if>
                  <if {components.stage.themePicker()}>
                    <theme-picker></theme-picker>
                  </if>

                  <!-- ── Feedback ───────────────────────────────────── -->
                  <if {components.stage.alert()}>
                    <alert tone={components.variant() || 'info'} w="full">
                      <strong>Heads up.</strong> Live alert under theme paint.
                    </alert>
                  </if>
                  <if {components.stage.progressBar()}>
                    <progress value={components.load()} max="100" w="full" sx={style.maxW(panel-max-w-sm)}></progress>
                  </if>
                  <if {components.stage.meter()}>
                    <column gap="2" w="full" sx={style.maxW(panel-max-w-sm)}>
                      <meter value="0.3"></meter>
                      <meter value="0.7"></meter>
                      <meter value="0.95"></meter>
                    </column>
                  </if>
                  <if {components.stage.spinner()}>
                    <spinner size={components.variant() || 'md'} tone="primary"></spinner>
                  </if>
                  <if {components.stage.emptyState()}>
                    <empty-state variant="inline" size="sm" w="full">
                      <strong slot="title">No results</strong>
                      Try another filter.
                    </empty-state>
                  </if>
                  <if {components.stage.errorState()}>
                    <error-state variant="inline" code="ERR_THEME" w="full">
                      Request failed — retry or check the connection.
                    </error-state>
                  </if>
                  <if {components.stage.errorBoundary()}>
                    <error-boundary message="Panel failed to load.">
                      <p sx={fieldHint}>Healthy boundary is invisible — this is the wrapped tree.</p>
                    </error-boundary>
                  </if>
                  <if {components.stage.toast()}>
                    <toast tone="success" duration="0" w="full" sx={style.maxW(panel-max-w-sm)}>
                      <strong>Saved.</strong> Sticky toast (duration 0).
                    </toast>
                  </if>
                  <if {components.stage.toastRegion()}>
                    <column gap="2" w="full" sx={style.maxW(panel-max-w-sm).position(relative).minH(24)}>
                      <p sx={fieldHint}>Toast region hosts stacked toasts (fixed in product).</p>
                      <toast tone="info" duration="0">Region child toast</toast>
                    </column>
                  </if>

                  <!-- ── Display ────────────────────────────────────── -->
                  <if {components.stage.badge()}>
                    <row gap="2" align="center" nowrap>
                      <badge variant={components.variant() || 'soft'} tone="primary" size="sm">primary</badge>
                      <badge variant={components.variant() || 'soft'} tone="success" size="sm">success</badge>
                      <badge variant={components.variant() || 'soft'} tone="warning" size="sm" pill>warn</badge>
                    </row>
                  </if>
                  <if {components.stage.chip()}>
                    <row gap="2" align="center" nowrap>
                      <chip variant={components.variant() || 'solid'} selected>Selected</chip>
                      <chip variant={components.variant() || 'solid'} tone="muted">Chip</chip>
                    </row>
                  </if>
                  <if {components.stage.card()}>
                    <card variant={components.variant() || 'default'} padding="sm" w="full" sx={style.maxW(panel-max-w-sm)}>
                      <column gap="2">
                        <p sx={doorTitle}>Card surface</p>
                        <p sx={fieldHint}>Judges component.card.radius when overridden.</p>
                      </column>
                    </card>
                  </if>
                  <if {components.stage.avatar()}>
                    <row gap="3" align="center" nowrap>
                      <avatar initials="AL" size={components.variant() || 'md'} tone="primary" aria-label="Ada"></avatar>
                      <avatar initials="GH" size={components.variant() || 'md'} shape="square" aria-label="GH"></avatar>
                    </row>
                  </if>
                  <if {components.stage.icon()}>
                    <icon size={components.variant() || 'md'} name="sparkles" label="sparkles"></icon>
                  </if>
                  <if {components.stage.separator()}>
                    <column gap="2" w="full" sx={style.maxW(panel-max-w-sm)}>
                      <p sx={fieldHint}>above</p>
                      <separator variant={components.variant() || 'solid'}></separator>
                      <p sx={fieldHint}>below</p>
                    </column>
                  </if>
                  <if {components.stage.details()}>
                    <details w="full" sx={style.maxW(panel-max-w-sm)}>
                      <summary>Advanced options</summary>
                      <p sx={fieldHint}>Native disclosure body.</p>
                    </details>
                  </if>
                  <if {components.stage.summary()}>
                    <details w="full" sx={style.maxW(panel-max-w-sm)} open>
                      <summary>Summary specimen</summary>
                      <p sx={fieldHint}>Summary is the disclosure trigger.</p>
                    </details>
                  </if>
                  <if {components.stage.collapsible()}>
                    <collapsible w="full" sx={style.maxW(panel-max-w-sm)}>
                      <strong slot="trigger">Show details</strong>
                      <p sx={fieldHint}>Collapsible body content.</p>
                    </collapsible>
                  </if>
                  <if {components.stage.prose()}>
                    <prose w="full" sx={style.maxW(panel-max-w-sm)}>
                      <p>Body copy with <strong>strong</strong> and <em>emphasis</em>.</p>
                    </prose>
                  </if>
                  <if {components.stage.codeBlock()}>
                    <code-block code={'const x = 1\nconsole.log(x)'} lang="ts" w="full" sx={style.maxW(panel-max-w-sm)}></code-block>
                  </if>

                  <!-- ── Overlay ────────────────────────────────────── -->
                  <if {components.stage.tooltip()}>
                    <tooltip content="Cannot be undone" placement={components.variant() || 'top'}>
                      <button type="button" size="sm" variant="outline" tone="destructive">Hover me</button>
                    </tooltip>
                  </if>
                  <if {components.stage.hoverCard()}>
                    <column gap="2" align="center">
                      <button type="button" size="sm" variant="ghost" popovertarget="theme-l1-hc">@ada</button>
                      <hover-card id="theme-l1-hc">
                        <column gap="1" sx={style.pad(3)}>
                          <p sx={doorTitle}>Ada Lovelace</p>
                          <p sx={fieldHint}>Hover card body.</p>
                        </column>
                      </hover-card>
                    </column>
                  </if>
                  <if {components.stage.popover()}>
                    <column gap="2" align="center">
                      <button type="button" size="sm" popovertarget="theme-l1-pop">Open popover</button>
                      <popover id="theme-l1-pop">
                        <p sx={style.pad(3)}>Popover panel under theme paint.</p>
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
                    <column gap="1" w="full" sx={style.maxW(panel-max-w-sm).pad(1).borderWidth(1).border(border).radius(md).bg(surface-1)}>
                      <menu-item>Edit theme</menu-item>
                      <menu-item>Duplicate</menu-item>
                      <menu-item>Export</menu-item>
                    </column>
                  </if>
                  <if {components.stage.contextMenu()}>
                    <column gap="2" align="center" w="full" sx={style.maxW(panel-max-w-sm)}>
                      <column id="theme-l1-ctx" gap="1" w="full" sx={style.pad(3).borderWidth(1).border(border).radius(md)}>
                        <p sx={fieldHint}>Right-click this area</p>
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
                    <tabs w="full">
                      <tab-list variant={components.variant() || 'line'}>
                        <tab>Overview</tab>
                        <tab>Shape</tab>
                        <tab>Assets</tab>
                      </tab-list>
                      <tab-panel><p sx={fieldHint}>Overview panel</p></tab-panel>
                      <tab-panel><p sx={fieldHint}>Shape panel</p></tab-panel>
                      <tab-panel><p sx={fieldHint}>Assets panel</p></tab-panel>
                    </tabs>
                  </if>
                  <if {components.stage.tabList()}>
                    <tab-list variant="line">
                      <tab>One</tab>
                      <tab>Two</tab>
                      <tab>Three</tab>
                    </tab-list>
                  </if>
                  <if {components.stage.tab()}>
                    <tab>Selected tab chrome</tab>
                  </if>
                  <if {components.stage.tabPanel()}>
                    <tab-panel>
                      <p sx={fieldHint}>Tab panel body.</p>
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
                    <pagination value={components.pageNo()} size={components.variant() || 'md'} on:sail-page-change={components.onPageChange}>
                      <page value="1">1</page>
                      <page value="2">2</page>
                      <page value="3">3</page>
                      <page value="4">4</page>
                    </pagination>
                  </if>
                  <if {components.stage.page()}>
                    <row gap="1" align="center">
                      <page value="1">1</page>
                      <page value="2">2</page>
                      <page value="3">3</page>
                    </row>
                  </if>
                  <if {components.stage.navbar()}>
                    <navbar bordered w="full">
                      <strong slot="brand">SHIP</strong>
                      <a href="/themes">Themes</a>
                      <a href="/themes">Assets</a>
                      <button type="button" slot="actions" size="sm" variant="outline">Sign in</button>
                    </navbar>
                  </if>

                  <!-- ── Data / layout ──────────────────────────────── -->
                  <if {components.stage.table()}>
                    <table w="full" dense>
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
                    <scroll-area sx={style.maxH(32).w(full).maxW(panel-max-w-sm).pad(3).borderWidth(1).border(border-subtle).radius(md)}>
                      <column gap="2">
                        <p sx={fieldHint}>Scroll me — themed scrollbars.</p>
                        <p sx={fieldHint}>Line 2</p>
                        <p sx={fieldHint}>Line 3</p>
                        <p sx={fieldHint}>Line 4</p>
                        <p sx={fieldHint}>Line 5</p>
                        <p sx={fieldHint}>Line 6</p>
                        <p sx={fieldHint}>Line 7</p>
                        <p sx={fieldHint}>Line 8</p>
                      </column>
                    </scroll-area>
                  </if>
                  <if {components.stage.ticker()}>
                    <ticker w="full" sx={style.maxW(panel-max-w-sm)}>
                      Live ticker · theme paint · radius overrides ·
                    </ticker>
                  </if>
                  <if {components.stage.a()}>
                    <row gap="3" align="center" nowrap>
                      <a href="#">Text link</a>
                      <a href="#" variant="solid" tone="primary" size="sm">Button link</a>
                    </row>
                  </if>

                </column>
              </column>
            </scroll-area>
          </column>
        </card>
      </column>
    </column>

    <!-- RIGHT: override inspector -->
    <scroll-area
      data-components-editor
      data-components-inspector
      shrink="0"
      sx={style.minW(panel-min-w-sm).maxW(panel-max-w-sm).w(panel-max-w-sm).minH(0).h(full).borderLWidth(1).borderL(border-subtle).bg(background)}
    >
      <column gap="5" sx={inspectorCol}>
        <column gap="2">
          <p sx={eyebrow}>Theme overrides</p>
          <p sx={style.fontSize(xl).fontWeight(semibold).tracking(tight).fontFamily(heading)}>
            {components.selectedLabel()}
          </p>
          <p sx={fieldHint}>{components.registryNote()}</p>
          <if {session.catalogHint()}>
            <p data-catalog-hint sx={fieldHint}>Add to project to edit.</p>
          </if>
        </column>

        <column data-section="component-tokens" gap="3">
          <row align="baseline" justify="between" gap="2" nowrap>
            <p sx={sectionTitle}>Component tokens</p>
            <p sx={pathMeta}>component.*</p>
          </row>
          <if {components.hasComponentKnobs()}>
            <column gap="3" data-component-knob-list>
              <for {components.componentKnobs()} as="tok" key={tok.key}>
                <column data-token-path={tok.path} gap="1" sx={tokenRow}>
                  <row align="center" justify="between" gap="2" nowrap>
                    <column gap="0" grow min-w="0">
                      <p sx={doorTitle}>{tok.label}</p>
                      <p sx={pathMeta}>{tok.path}</p>
                    </column>
                    <if {tok.isOverridden}>
                      <button type="button" size="sm" variant="ghost" tone="muted" data-token-path={tok.path} on:click={components.onClearComponent} disabled={!session.canEdit()}>
                        Clear
                      </button>
                    </if>
                  </row>
                  <if {tok.isColor}>
                    <row align="center" gap="2" nowrap>
                      <stack data-token-chip shrink="0" sx={style.position(relative).w(9).h(9).minW(9).minH(9).radius(md).borderWidth(1).border(border-subtle).overflow(hidden).bg(surface-0)}>
                        <sail-color-picker data-token-path={tok.path} value={tok.hex} disabled={!session.canEdit()} variant="swatch" showHex={false} on:sail-change={components.onComponentColor}></sail-color-picker>
                      </stack>
                      <column gap="0" grow min-w="0">
                        <p sx={pathMeta}>{tok.hexLabel}</p>
                        <if {!tok.isOverridden}>
                          <p sx={fieldHint}>Default · pick to override for this component only</p>
                        </if>
                      </column>
                    </row>
                  </if>
                  <if {!tok.isColor}>
                    <p sx={fieldHint}>{tok.raw}</p>
                  </if>
                </column>
              </for>
            </column>
          </if>
          <if {!components.hasComponentKnobs()}>
            <p sx={fieldHint}>No component-scoped tokens for this face in the registry.</p>
          </if>
        </column>

        <column data-section="global-tokens" gap="3">
          <row align="baseline" justify="between" gap="2" nowrap>
            <p sx={sectionTitle}>Theme tokens this face uses</p>
            <p sx={pathMeta}>theme-wide</p>
          </row>
          <p sx={fieldHint}>
            Edit a shared theme color here, or set a component token above to override only this component.
          </p>
          <if {components.hasGlobalKnobs()}>
            <column gap="3" data-global-knob-list>
              <for {components.globalKnobs()} as="tok" key={tok.key}>
                <column data-token-path={tok.path} gap="1" sx={tokenRow}>
                  <column gap="0">
                    <p sx={doorTitle}>{tok.label}</p>
                    <p sx={pathMeta}>{tok.path}</p>
                  </column>
                  <if {tok.isColor}>
                    <row align="center" gap="2" nowrap>
                      <stack data-token-chip shrink="0" sx={style.position(relative).w(9).h(9).minW(9).minH(9).radius(md).borderWidth(1).border(border-subtle).overflow(hidden).bg(surface-0)}>
                        <sail-color-picker data-token-path={tok.path} value={tok.hex} disabled={!session.canEdit()} variant="swatch" showHex={false} on:sail-change={components.onGlobalColor}></sail-color-picker>
                      </stack>
                      <p sx={pathMeta}>{tok.hexLabel}</p>
                    </row>
                  </if>
                  <if {!tok.isColor}>
                    <p sx={fieldHint}>{tok.raw}</p>
                  </if>
                </column>
              </for>
            </column>
          </if>
          <if {!components.hasGlobalKnobs()}>
            <p sx={fieldHint}>No related globals listed for this face.</p>
          </if>
        </column>
      </column>
    </scroll-area>
  </row>

  <toast-region position="bottom-end" data-components-toast-region>
    <for {session.toastItems()} as="t" key={t.key}>
      <toast tone={t.tone} data-components-toast on:sail-toast-dismiss={session.onToastDismiss}>
        {t.message}
      </toast>
    </for>
  </toast-region>
</column>
