<!--
  Themes → Components facet — component token overrides (not the Components entity page).
  Storybook-ish: catalog rail | large stage | override inspector.
  Defaults resolve via DTCG color paths; overrides write component.* leaves.
-->
<scope>
  const sectionTitle = style({ text: fontSize(sm).fontWeight(semibold) })
  const fieldHint = style({ paint: fg(muted-foreground), text: fontSize(xs) })
  const pathMeta = style({ paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) })
  const doorTitle = style({ text: fontWeight(semibold) })
  const bandStatus = style({ box: padX(4).padY(2) })
  const bandError = style({ box: padX(4).padY(2), paint: borderBWidth(1).borderB(border) })
  const railCol = style({ box: padX(3).padT(4).padB(6).minW(0) })
  const inspectorCol = style({ box: padX(4).padT(4).padB(8).minW(0) })
  const fullBtn = style({ box: w(full) })
  const chipCol = style({ box: minW(0) })
  // Selected rail row: soft fill + subtle border (no nested cards).
  const listRow = style({ box: padX(2_5).padY(2), paint: radius(md).cursor(pointer).borderWidth(1).border(transparent) }).when(hover, { paint: bg(hover-bg) }).when(selected, { paint: bg(surface-2).border(border-subtle) })
  const eyebrow = style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).tracking(wide).textCase(uppercase) })
  const tokenStrip = style({ box: padX(2).padY(1), paint: fg(muted-foreground).radius(sm).bg(surface-0), text: fontSize(xs).fontFamily(mono) })
  const tokenRow = style({ box: padY(1_5), paint: borderBWidth(1).borderB(border-subtle) })
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

<column surface="themes" theme-components-surface components-chrome gap="0">
  style({  box: grow(1).minH(0).h(full).minW(0).overflow(hidden), paint: bg(background)  })

  <theme-facet-chrome session={session}></theme-facet-chrome>

  <if {session.hasError()}>
    <row components-error align="center" gap="2">
      style([ style(bandError), style({ box: shrink(0) }) ])

      <p>
        style({ box: grow(1).minW(0), paint: fg(destructive), text: fontSize(sm).truncate() })
{session.errorText()}</p>
      <button type="button" size="sm" variant="ghost" components-retry on:click={session.onRetry}>Retry</button>
    </row>
  </if>

  <if {session.loading()}>
    <row components-loading align="center" gap="2">
      style([ style(bandStatus), style({ box: shrink(0) }) ])

      <spinner size="sm"></spinner>
      <p>
        style(fieldHint)
Loading…</p>
    </row>
  </if>

  <!-- Storybook-ish: rail | LARGE stage | inspector -->
  <row components-workbench gap="0" nowrap align="stretch">
    style({  box: grow(1).minH(0).basis(0).h(full).minW(0).overflow(hidden), paint: bg(background)  })

    <!-- LEFT: component catalog rail -->
    <scroll-area components-rail>
      style({  box: shrink(0).minW(panel-min-w-sm).maxW(panel-max-w-sm).w(panel-max-w-sm).minH(0).h(full), paint: borderRWidth(1).borderR(border-subtle).bg(background)  })

      <column gap="3">
        style(railCol)

        <column gap="2">
          <p>
            style(eyebrow)
Components</p>
          <p>
            style({ text: fontSize(md).fontWeight(semibold).tracking(tight) })
Theme overrides</p>
          <p l1-facet-blurb>
            style(fieldHint)

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
            l1-filter
            value={components.filter()}
            on:input={components.onFilterInput}
            placeholder="Filter components…"
            spellcheck="false"
            label="Filter components"
          />
        </column>
        <column l1-list gap="1" on:click={components.onSelect}>
          <for {components.componentRows()} as="row" key={row.key}>
            <row
              l1={row.id}
              selected={row.selected ? 'true' : 'false'}
              selected={row.selected ? 'true' : 'false'}
              align="center"
              gap="2"
              nowrap

            >
              style(listRow)

              <column gap="0">
                style({ box: grow(1).minW(0) })

                <p>
                  style(doorTitle)
{row.label}</p>
                <p>
                  style(fieldHint)
{row.hint}</p>
              </column>
              <if {row.hasOverride}>
                <badge tone="primary" size="sm" variant="solid" pill>
                  style({ box: shrink(0) })
override</badge>
              </if>
            </row>
          </for>
        </column>
      </column>
    </scroll-area>

    <!-- CENTER: large live specimen stage -->
    <column specimen-dock components-stage gap="0">
      style({  box: grow(1).minW(0).minW(0).basis(0).minH(0).h(full).overflow(hidden), paint: bg(background)  })

      <column gap="0">
        style({  box: grow(1).minH(0).h(full).pad(4)  })

        <!--
          One themed host only. Stage content sits ON background —
          no nested white plate under the live specimen (light-mode clash).
          Frame chrome shows theme name + mode — no external Preview/light label.
        -->
        <card
          components-specimen
          preview-specimen
          mode={session.paintMode()}
          padding="none"

        >
          style({ box: minW(0).minH(0).h(full).overflow(hidden), paint: shadow(lg) })

          <column gap="0">
            style({  box: grow(1).minH(0).h(full)  })

            <row specimen-chrome align="center" gap="2" nowrap>
              style({  box: shrink(0).padX(3).h(8).minH(8).minW(0), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1)  })

              <row gap="2" align="center" nowrap hidden="true">
                style({  box: shrink(0).minW(12)  })

                <stack>
                  style({  box: shrink(0).w(2).h(2).minW(2).minH(2), paint: radius(full).bg(destructive)  })
</stack>
                <stack>
                  style({  box: shrink(0).w(2).h(2).minW(2).minH(2), paint: radius(full).bg(warning)  })
</stack>
                <stack>
                  style({  box: shrink(0).w(2).h(2).minW(2).minH(2), paint: radius(full).bg(success)  })
</stack>
              </row>
              <p>
                style({ box: grow(1).minW(0), paint: fg(muted-foreground), text: fontSize(xs).truncate() })

                {session.nameEdit()} · {components.selectedLabel()}
              </p>
              <p>
                style([ style(pathMeta), style({ box: shrink(0) }) ])
{session.paintMode()}</p>
            </row>

            <if {components.hasVariants()}>
              <row specimen-variants align="center" gap="2" nowrap on:click={components.onVariantPick}>
                style({  box: shrink(0).padX(3).padY(2).minW(0), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1)  })

                <p>
                  style([ style(pathMeta), style({ box: shrink(0) }) ])
{components.viewerVariantAttr()}:</p>
                <for {components.variantChips()} as="c" key={c.key}>
                  <button
                    type="button"
                    size="sm"
                    tone="primary"
                    variant={c.btnVariant}
                    variant={c.v}
                  >
                    {c.v}
                  </button>
                </for>
              </row>
            </if>

            <scroll-area>
              style({  box: grow(1).minH(0).basis(0).h(full), paint: bg(background)  })

              <column
                specimen-body
                l1-live-stage={components.selectedId()}
                gap="5"
                align="stretch"

              >
                style({ box: pad(5).minW(0).minH(full) })

                <column gap="1">
                  <p>
                    style({ text: fontSize(md).fontWeight(semibold).fontFamily(heading) })

                    {components.selectedLabel()}
                  </p>
                  <p>
                    style({ paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed).fontFamily(body) })

                    {components.viewerBlurb()}
                  </p>
                  <p>
                    style(pathMeta)
{components.registryNote()}</p>
                </column>

                <column gap="4" align="start" l1-stage-host>
                  style({ box: w(full) })


                  <!-- ── Inputs ─────────────────────────────────────── -->
                  <if {components.stage.button()}>
                    <column gap="3">
                      style({ box: w(full) })

                      <row gap="2" align="center" nowrap>
                        <button type="button" size="sm" variant={components.variant() || 'solid'} tone="primary">Primary</button>
                        <button type="button" size="sm" variant={components.variant() || 'solid'} tone="accent">Accent</button>
                        <button type="button" size="sm" variant={components.variant() || 'outline'} tone="primary">Outline</button>
                        <button type="button" size="sm" variant={components.variant() || 'ghost'} tone="muted">Ghost</button>
                      </row>
                    </column>
                  </if>
                  <if {components.stage.input()}>
                    <column gap="2">
                      style({  box: w(full).maxW(panel-max-w-sm)  })

                      <input type="text" size="sm" placeholder="Sample field" value={components.sampleText()} on:input={components.onSampleInput} label="Sample field" />
                    </column>
                  </if>
                  <if {components.stage.textarea()}>
                    <textarea size="sm" rows="3" placeholder="Multiline" label="Sample textarea">
                      style({  box: w(full).maxW(panel-max-w-sm)  })
</textarea>
                  </if>
                  <if {components.stage.select()}>
                    <select size="sm" label="Sample select">
                      style({  box: w(full).maxW(panel-max-w-sm)  })

                      <option>Option A</option>
                      <option>Option B</option>
                      <option>Option C</option>
                    </select>
                  </if>
                  <if {components.stage.combobox()}>
                    <combobox placeholder="Type to filter…">
                      style({  box: w(full).maxW(panel-max-w-sm)  })

                      <option value="sail">Sail</option>
                      <option value="react">React</option>
                      <option value="svelte">Svelte</option>
                    </combobox>
                  </if>
                  <if {components.stage.radio()}>
                    <radio value="a">Option A</radio>
                  </if>
                  <if {components.stage.radioGroup()}>
                    <radio-group label="Plan" name="theme-l1-plan">
                      style({  box: w(full).maxW(panel-max-w-sm)  })

                      <radio value="monthly">Monthly</radio>
                      <radio value="yearly">Yearly</radio>
                    </radio-group>
                  </if>
                  <if {components.stage.switchToggle()}>
                    <row gap="3" align="center">
                      <switch size={components.variant() || 'md'} tone="primary" checked={components.notify()} label="Sample switch"></switch>
                      <p>
                        style(fieldHint)
{components.notify() ? 'on' : 'off'}</p>
                    </row>
                  </if>
                  <if {components.stage.slider()}>
                    <column gap="2">
                      style({  box: w(full).maxW(panel-max-w-sm)  })

                      <slider value={components.load()} min="0" max="100" tone={components.variant() || 'primary'} label="Sample slider"></slider>
                      <p>
                        style(fieldHint)
{components.load()}%</p>
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
                    <field>
                      style({  box: w(full).maxW(panel-max-w-sm)  })

                      <label slot="label" for="theme-l1-field">Name</label>
                      <input id="theme-l1-field" size="sm" placeholder="Ada Lovelace" />
                      <p slot="hint">Hint under the control</p>
                    </field>
                  </if>
                  <if {components.stage.label()}>
                    <column gap="1">
                      style({  box: w(full).maxW(panel-max-w-sm)  })

                      <label for="theme-l1-label" size={components.variant() || 'md'} required>Email</label>
                      <input id="theme-l1-label" size="sm" placeholder="ada@example.com" />
                    </column>
                  </if>
                  <if {components.stage.fieldset()}>
                    <fieldset>
                      style({  box: w(full).maxW(panel-max-w-sm)  })

                      <legend>Account</legend>
                      <column gap="2">
                        <input size="sm" placeholder="Name" />
                        <input size="sm" placeholder="Email" />
                      </column>
                    </fieldset>
                  </if>
                  <if {components.stage.legend()}>
                    <fieldset>
                      style({  box: w(full).maxW(panel-max-w-sm)  })

                      <legend>Legend specimen</legend>
                      <p>
                        style(fieldHint)
Legend lives on a fieldset.</p>
                    </fieldset>
                  </if>
                  <if {components.stage.themePicker()}>
                    <theme-picker></theme-picker>
                  </if>

                  <!-- ── Feedback ───────────────────────────────────── -->
                  <if {components.stage.alert()}>
                    <alert tone={components.variant() || 'info'}>
                      style({ box: w(full) })

                      <strong>Heads up.</strong> Live alert under theme paint.
                    </alert>
                  </if>
                  <if {components.stage.progressBar()}>
                    <progress value={components.load()} max="100">
                      style({  box: w(full).maxW(panel-max-w-sm)  })
</progress>
                  </if>
                  <if {components.stage.meter()}>
                    <column gap="2">
                      style({  box: w(full).maxW(panel-max-w-sm)  })

                      <meter value="0.3"></meter>
                      <meter value="0.7"></meter>
                      <meter value="0.95"></meter>
                    </column>
                  </if>
                  <if {components.stage.spinner()}>
                    <spinner size={components.variant() || 'md'} tone="primary"></spinner>
                  </if>
                  <if {components.stage.emptyState()}>
                    <empty-state variant="inline" size="sm">
                      style({ box: w(full) })

                      <strong slot="title">No results</strong>
                      Try another filter.
                    </empty-state>
                  </if>
                  <if {components.stage.errorState()}>
                    <error-state variant="inline" code="ERR_THEME">
                      style({ box: w(full) })

                      Request failed — retry or check the connection.
                    </error-state>
                  </if>
                  <if {components.stage.errorBoundary()}>
                    <error-boundary message="Panel failed to load.">
                      <p>
                        style(fieldHint)
Healthy boundary is invisible — this is the wrapped tree.</p>
                    </error-boundary>
                  </if>
                  <if {components.stage.toast()}>
                    <toast tone="success" duration="0">
                      style({  box: w(full).maxW(panel-max-w-sm)  })

                      <strong>Saved.</strong> Sticky toast (duration 0).
                    </toast>
                  </if>
                  <if {components.stage.toastRegion()}>
                    <column gap="2">
                      style({  box: w(full).maxW(panel-max-w-sm).position(relative).minH(24)  })

                      <p>
                        style(fieldHint)
Toast region hosts stacked toasts (fixed in product).</p>
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
                    <card variant={components.variant() || 'default'} padding="sm">
                      style({  box: w(full).maxW(panel-max-w-sm)  })

                      <column gap="2">
                        <p>
                          style(doorTitle)
Card surface</p>
                        <p>
                          style(fieldHint)
Judges component.card.radius when overridden.</p>
                      </column>
                    </card>
                  </if>
                  <if {components.stage.avatar()}>
                    <row gap="3" align="center" nowrap>
                      <avatar initials="AL" size={components.variant() || 'md'} tone="primary" label="Ada"></avatar>
                      <avatar initials="GH" size={components.variant() || 'md'} shape="square" label="GH"></avatar>
                    </row>
                  </if>
                  <if {components.stage.icon()}>
                    <icon size={components.variant() || 'md'} name="sparkles" label="sparkles"></icon>
                  </if>
                  <if {components.stage.separator()}>
                    <column gap="2">
                      style({  box: w(full).maxW(panel-max-w-sm)  })

                      <p>
                        style(fieldHint)
above</p>
                      <separator variant={components.variant() || 'solid'}></separator>
                      <p>
                        style(fieldHint)
below</p>
                    </column>
                  </if>
                  <if {components.stage.details()}>
                    <details>
                      style({  box: w(full).maxW(panel-max-w-sm)  })

                      <summary>Advanced options</summary>
                      <p>
                        style(fieldHint)
Native disclosure body.</p>
                    </details>
                  </if>
                  <if {components.stage.summary()}>
                    <details open>
                      style({  box: w(full).maxW(panel-max-w-sm)  })

                      <summary>Summary specimen</summary>
                      <p>
                        style(fieldHint)
Summary is the disclosure trigger.</p>
                    </details>
                  </if>
                  <if {components.stage.collapsible()}>
                    <collapsible>
                      style({  box: w(full).maxW(panel-max-w-sm)  })

                      <strong slot="trigger">Show details</strong>
                      <p>
                        style(fieldHint)
Collapsible body content.</p>
                    </collapsible>
                  </if>
                  <if {components.stage.prose()}>
                    <prose>
                      style({  box: w(full).maxW(panel-max-w-sm)  })

                      <p>Body copy with <strong>strong</strong> and <em>emphasis</em>.</p>
                    </prose>
                  </if>
                  <if {components.stage.codeBlock()}>
                    <code-block code={'const x = 1\nconsole.log(x)'} lang="ts">
                      style({  box: w(full).maxW(panel-max-w-sm)  })
</code-block>
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
                        <column gap="1">
                          style({ box: pad(3) })

                          <p>
                            style(doorTitle)
Ada Lovelace</p>
                          <p>
                            style(fieldHint)
Hover card body.</p>
                        </column>
                      </hover-card>
                    </column>
                  </if>
                  <if {components.stage.popover()}>
                    <column gap="2" align="center">
                      <button type="button" size="sm" popovertarget="theme-l1-pop">Open popover</button>
                      <popover id="theme-l1-pop">
                        <p>
                          style({ box: pad(3) })
Popover panel under theme paint.</p>
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
                    <column gap="1">
                      style({  box: w(full).maxW(panel-max-w-sm).pad(1), paint: borderWidth(1).border(border).radius(md).bg(surface-1)  })

                      <menu-item>Edit theme</menu-item>
                      <menu-item>Duplicate</menu-item>
                      <menu-item>Export</menu-item>
                    </column>
                  </if>
                  <if {components.stage.contextMenu()}>
                    <column gap="2" align="center">
                      style({  box: w(full).maxW(panel-max-w-sm)  })

                      <column id="theme-l1-ctx" gap="1">
                        style({  box: w(full).pad(3), paint: borderWidth(1).border(border).radius(md)  })

                        <p>
                          style(fieldHint)
Right-click this area</p>
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
                    <tabs>
                      style({ box: w(full) })

                      <tab-list variant={components.variant() || 'line'}>
                        <tab>Overview</tab>
                        <tab>Shape</tab>
                        <tab>Assets</tab>
                      </tab-list>
                      <tab-panel><p>
                        style(fieldHint)
Overview panel</p></tab-panel>
                      <tab-panel><p>
                        style(fieldHint)
Shape panel</p></tab-panel>
                      <tab-panel><p>
                        style(fieldHint)
Assets panel</p></tab-panel>
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
                      <p>
                        style(fieldHint)
Tab panel body.</p>
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
                    <navbar bordered>
                      style({ box: w(full) })

                      <strong slot="brand">SHIP</strong>
                      <a href="/themes">Themes</a>
                      <a href="/themes">Assets</a>
                      <button type="button" slot="actions" size="sm" variant="outline">Sign in</button>
                    </navbar>
                  </if>

                  <!-- ── Data / layout ──────────────────────────────── -->
                  <if {components.stage.table()}>
                    <table dense>
                      style({ box: w(full) })

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
                    <scroll-area>
                      style({ box: maxH(32).w(full).maxW(panel-max-w-sm).pad(3), paint: borderWidth(1).border(border-subtle).radius(md) })

                      <column gap="2">
                        <p>
                          style(fieldHint)
Scroll me — themed scrollbars.</p>
                        <p>
                          style(fieldHint)
Line 2</p>
                        <p>
                          style(fieldHint)
Line 3</p>
                        <p>
                          style(fieldHint)
Line 4</p>
                        <p>
                          style(fieldHint)
Line 5</p>
                        <p>
                          style(fieldHint)
Line 6</p>
                        <p>
                          style(fieldHint)
Line 7</p>
                        <p>
                          style(fieldHint)
Line 8</p>
                      </column>
                    </scroll-area>
                  </if>
                  <if {components.stage.ticker()}>
                    <ticker>
                      style({  box: w(full).maxW(panel-max-w-sm)  })

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
    <scroll-area components-editor components-inspector>
      style({  box: shrink(0).minW(panel-min-w-sm).maxW(panel-max-w-sm).w(panel-max-w-sm).minH(0).h(full), paint: borderLWidth(1).borderL(border-subtle).bg(background)  })

      <column gap="5">
        style(inspectorCol)

        <column gap="2">
          <p>
            style(eyebrow)
Theme overrides</p>
          <p>
            style({ text: fontSize(xl).fontWeight(semibold).tracking(tight).fontFamily(heading) })

            {components.selectedLabel()}
          </p>
          <p>
            style(fieldHint)
{components.registryNote()}</p>
          <if {session.catalogHint()}>
            <p catalog-hint>
              style(fieldHint)
Add to project to edit.</p>
          </if>
        </column>

        <column section="component-tokens" gap="3">
          <row align="baseline" justify="between" gap="2" nowrap>
            <p>
              style(sectionTitle)
Component tokens</p>
            <p>
              style(pathMeta)
component.*</p>
          </row>
          <if {components.hasComponentKnobs()}>
            <column gap="3" component-knob-list>
              <for {components.componentKnobs()} as="tok" key={tok.key}>
                <column token-path={tok.path} gap="1">
                  style(tokenRow)

                  <row align="center" justify="between" gap="2" nowrap>
                    <column gap="0">
                      style({ box: grow(1).minW(0) })

                      <p>
                        style(doorTitle)
{tok.label}</p>
                      <p>
                        style(pathMeta)
{tok.path}</p>
                    </column>
                    <if {tok.isOverridden}>
                      <button type="button" size="sm" variant="ghost" tone="muted" token-path={tok.path} on:click={components.onClearComponent} disabled={!session.canEdit()}>
                        Clear
                      </button>
                    </if>
                  </row>
                  <if {tok.isColor}>
                    <row align="center" gap="2" nowrap>
                      <stack token-chip>
                        style({  box: shrink(0).position(relative).w(9).h(9).minW(9).minH(9).overflow(hidden), paint: radius(md).borderWidth(1).border(border-subtle).bg(surface-0)  })

                        <sail-color-picker token-path={tok.path} value={tok.hex} disabled={!session.canEdit()} variant="swatch" showHex={false} on:sail-change={components.onComponentColor}></sail-color-picker>
                      </stack>
                      <column gap="0">
                        style({ box: grow(1).minW(0) })

                        <p>
                          style(pathMeta)
{tok.hexLabel}</p>
                        <if {!tok.isOverridden}>
                          <p>
                            style(fieldHint)
Default · pick to override for this component only</p>
                        </if>
                      </column>
                    </row>
                  </if>
                  <if {!tok.isColor}>
                    <p>
                      style(fieldHint)
{tok.raw}</p>
                  </if>
                </column>
              </for>
            </column>
          </if>
          <if {!components.hasComponentKnobs()}>
            <p>
              style(fieldHint)
No component-scoped tokens for this face in the registry.</p>
          </if>
        </column>

        <column section="global-tokens" gap="3">
          <row align="baseline" justify="between" gap="2" nowrap>
            <p>
              style(sectionTitle)
Theme tokens this face uses</p>
            <p>
              style(pathMeta)
theme-wide</p>
          </row>
          <p>
            style(fieldHint)

            Edit a shared theme color here, or set a component token above to override only this component.
          </p>
          <if {components.hasGlobalKnobs()}>
            <column gap="3" global-knob-list>
              <for {components.globalKnobs()} as="tok" key={tok.key}>
                <column token-path={tok.path} gap="1">
                  style(tokenRow)

                  <column gap="0">
                    <p>
                      style(doorTitle)
{tok.label}</p>
                    <p>
                      style(pathMeta)
{tok.path}</p>
                  </column>
                  <if {tok.isColor}>
                    <row align="center" gap="2" nowrap>
                      <stack token-chip>
                        style({  box: shrink(0).position(relative).w(9).h(9).minW(9).minH(9).overflow(hidden), paint: radius(md).borderWidth(1).border(border-subtle).bg(surface-0)  })

                        <sail-color-picker token-path={tok.path} value={tok.hex} disabled={!session.canEdit()} variant="swatch" showHex={false} on:sail-change={components.onGlobalColor}></sail-color-picker>
                      </stack>
                      <p>
                        style(pathMeta)
{tok.hexLabel}</p>
                    </row>
                  </if>
                  <if {!tok.isColor}>
                    <p>
                      style(fieldHint)
{tok.raw}</p>
                  </if>
                </column>
              </for>
            </column>
          </if>
          <if {!components.hasGlobalKnobs()}>
            <p>
              style(fieldHint)
No related globals listed for this face.</p>
          </if>
        </column>
      </column>
    </scroll-area>
  </row>

  <toast-region position="bottom-end" components-toast-region>
    <for {session.toastItems()} as="t" key={t.key}>
      <toast tone={t.tone} components-toast on:sail-toast-dismiss={session.onToastDismiss}>
        {t.message}
      </toast>
    </for>
  </toast-region>
</column>
