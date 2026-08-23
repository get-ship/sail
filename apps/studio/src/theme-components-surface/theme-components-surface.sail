<!--
  Themes → Components facet — live L1 specimens + theme role knobs
  (not the project Components library). Catalog rail | live stage | role inspector.
-->

<scope>
  style sectionTitle = { type(body-sm) }
  style fieldHint = { fg(muted).type(body-sm) }
  style pathMeta = { fg(muted).type(mono-sm) }
  // no preset: type weight
  style doorTitle = { type(body-semibold) }
  style bandStatus = { pad(4) }
  // no preset: edge-only border
  style bandError = { pad(4).border(default-b) }
  // no preset: min-w lock
  style railCol = { pad(3).size({ min-w: 0 }) }
  // no preset: min-w lock
  style inspectorCol = { pad(4).size({ min-w: 0 }) }
  // Selected rail row: soft fill + subtle border.
  // no preset: transparent border
  style listRow = { pad(2).radius(md).cursor(pointer).border({ color: transparent, width: 1 }), when: { hover: bg(hover-bg), selected: bg(surface).border(subtle) } }
  style eyebrow = { fg(muted).type(label) }
  // no preset: edge-only border
  style tokenRow = { pad(2).border(subtle-b) }
  style surfaceRoot = { grow(1).overflow(hidden).bg(app) }
  style workbench = { size(grow).overflow(hidden).bg(app) }
  // no preset: panel width lock + edge-only border
  style panelRail = { size({ w: overlay-max-w-sm, h: 100%, min-h: 0, max-w: overlay-max-w-sm }).border(subtle-r).bg(app) }
  // no preset: panel width lock + edge-only border
  style panelInspector = { size({ w: overlay-max-w-sm, h: 100%, min-h: 0, max-w: overlay-max-w-sm }).border(subtle-l).bg(app) }
  style specimenDock = { size(grow).overflow(hidden).bg(app) }
  style specimenPad = { size(grow).pad(4) }
  style specimenFrame = { size(max).overflow(hidden).elevation(lg) }
  // no preset: min-w lock + edge-only border
  style specimenChrome = { pad(3).size({ h: 8, min-w: 0, min-h: 8 }).border(subtle-b).bg(surface) }
  // no preset: min-w lock + edge-only border
  style specimenVariants = { pad(3).size({ min-w: 0 }).border(subtle-b).bg(surface) }
  // no preset: size lock
  style trafficDots = { size({ min-w: 12 }) }
  // no preset: size lock
  style trafficDot = { size({ w: 2, h: 2, min-w: 2, min-h: 2 }).radius(full) }
  style chromeTitle = { size(grow).fg(muted).type(body-sm) }
  style errorText = { size(grow).fg(danger).type(body-sm) }
  style growMin = { size(grow) }
  // Card body fill — participation form; card is not a layout host.
  style previewFill = { grow(1).size({ min-h: 0 }) }
  // no preset: size lock + position
  style toastRegionHost = { size({ w: 100%, min-h: 24, max-w: overlay-max-w-sm }).position(relative) }
  style shrink0 = { size(hug) }
  style tapFloor = { size(control-sm) }
  style iconBtn = { size(control-sm) }
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
  style scrollGrow = { size(grow).bg(app) }
  // no preset: min-w lock
  style stageBody = { pad(5).size({ min-w: 0, min-h: 100% }) }
  style stageTitle = { type(heading) }
  style stageBlurb = { fg(muted).type(body-sm) }
  style inspectorTitle = { type(heading) }
  // no preset: size lock + position
  style tokenChip = { position(relative).size({ w: 9, h: 9, min-w: 9, min-h: 9 }).overflow(hidden).radius(md).border(subtle).bg(app) }
  type ComponentRailRowProps = {
    id: string
    label: string
    hint: string
    selected: boolean
    hasOverride: boolean
  }
</scope>

<controller>
import { createThemeSession } from '../compositions/theme-session/theme-session'
import { watchPreviewHost } from '../compositions/theme-session/bind-preview-host'
import { createComponentsEdit } from '../compositions/theme-components-surface/theme-components-edit'
export default controller(() => {
  const session = createThemeSession({
    facet: 'components',
    editGuardMessage: 'Add this theme to the project to edit theme roles.',
    saveSuccessMessage: 'Theme roles saved.',
  })
  const components = createComponentsEdit(session)
  mount((host) => watchPreviewHost(host, (el) => session.attachPreviewHost(el)))
  return { session, components }
})
</controller>

<partial component-rail-row props: ComponentRailRowProps>
  <row
    l1={id}
    selected={selected ? 'true' : 'false'}
    align="center"
    gap="2"
    nowrap
    style(listRow)
  >
    <column gap="0" style(growMin)>
      <p style(doorTitle)>{label}</p>
      <p style(fieldHint)>{hint}</p>
    </column>
    <if {hasOverride}>
      <badge tone="primary" size="sm" variant="solid" pill style(shrink0)>override</badge>
    </if>
  </row>
</partial>

<column surface="themes" theme-components-surface components-chrome gap="0" style(surfaceRoot)>
  <theme-facet-chrome session={session}></theme-facet-chrome>

  <if {session.hasError()}>
    <row components-error align="center" gap="2" style(bandError, shrink0)>
      <p style(errorText)>{session.errorText()}</p>
      <button type="button" size="sm" variant="ghost" components-retry on:click={session.onRetry}>Retry</button>
    </row>
  </if>

  <if {session.loading()}>
    <row components-loading align="center" gap="2" style(bandStatus, shrink0)>
      <spinner size="sm"></spinner>
      <p style(fieldHint)>Loading…</p>
    </row>
  </if>

  <row components-workbench gap="0" nowrap align="stretch" style(workbench)>
    <scroll-area components-rail style(panelRail)>
      <column gap="3" style(railCol)>
        <column gap="2">
          <p style(eyebrow)>Components</p>
          <p style({ type(heading) })>Components</p>
          <p l1-facet-blurb style(fieldHint)>
            Theme roles this face spends — not the project Components library.
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
            value={components.filterQuery()}
            on:input={components.onFilterInput}
            placeholder="Filter components…"
            spellcheck="false"
            label="Filter components"
          />
        </column>
        <column l1-list gap="1" on:click={components.onSelect}>
          <for {components.componentRows() as row} key={row.key}>
            <component-rail-row
              id={row.id}
              label={row.label}
              hint={row.hint}
              selected={row.selected}
              hasOverride={row.hasOverride}
            ></component-rail-row>
          </for>
        </column>
      </column>
    </scroll-area>

    <column specimen-dock components-stage gap="0" style(specimenDock)>
      <column gap="0" style(specimenPad)>
        <!-- Preview host: draft paint via applyScope (theme + mode). -->
        <card
          components-specimen
          preview-specimen
          padding="none" style(specimenFrame)>
          <column gap="0" style(previewFill)>
            <row preview-chrome align="center" gap="2" nowrap style(specimenChrome)>
              <row gap="2" align="center" nowrap hidden="true" style(trafficDots)>
                <stack style(trafficDot, { bg(danger) })></stack>
                <stack style(trafficDot, { bg(warning) })></stack>
                <stack style(trafficDot, { bg(success) })></stack>
              </row>
              <p style(chromeTitle)>
                Preview · {session.nameEdit()} · {components.selectedLabel()}
              </p>
              <p style(pathMeta, shrink0)>{session.paintMode()}</p>
            </row>

            <if {components.hasVariants()}>
              <row specimen-variants align="center" gap="2" nowrap on:click={components.onVariantPick} style(specimenVariants)>
                <p style(pathMeta, shrink0)>{components.viewerVariantAttr()}:</p>
                <for {components.variantChips() as c} key={c.key}>
                  <button
                    type="button"
                    size="sm"
                    tone="primary"
                    variant={c.v}
                    style(tapFloor)
                  >
                    {c.v}
                  </button>
                </for>
              </row>
            </if>

            <scroll-area style(scrollGrow)>
              <column
                specimen-body
                l1-live-stage={components.selectedId()}
                gap="5"
                align="stretch" style(stageBody)>
                <column gap="1">
                  <p style(stageTitle)>
                    {components.selectedLabel()}
                  </p>
                  <p style(stageBlurb)>
                    {components.viewerBlurb()}
                  </p>
                  <p style(pathMeta)>{components.registryNote()}</p>
                </column>

                <if {!components.hasSpecimen()}>
                  <p style(fieldHint)>No live specimen for this host.</p>
                </if>
                <column gap="4" align="start" l1-stage-host style(stageHost)>

                  <!-- ── Inputs ─────────────────────────────────────── -->
                  <if {components.stage.button()}>
                    <column gap="4" style(stageHost)>
                      <column gap="2">
                        <p style(fieldHint)>Default · solid primary · no extra attrs</p>
                        <row gap="2" align="center" nowrap>
                          <button type="button" size="sm">Save</button>
                          <button type="button" size="sm" variant="solid">Solid</button>
                          <button type="button" size="sm" variant="soft">Soft</button>
                          <button type="button" size="sm" variant="outline">Outline</button>
                          <button type="button" size="sm" variant="ghost">Ghost</button>
                        </row>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Solid · tones</p>
                        <row gap="2" align="center" nowrap>
                          <button type="button" size="sm" variant="solid" tone="primary">Primary</button>
                          <button type="button" size="sm" variant="solid" tone="success">Success</button>
                          <button type="button" size="sm" variant="solid" tone="warning">Warning</button>
                          <button type="button" size="sm" variant="solid" tone="danger">Danger</button>
                          <button type="button" size="sm" variant="solid" tone="accent">Accent</button>
                          <button type="button" size="iconSm" variant="solid" tone="primary" label="Settings" style(iconBtn)>
                            <icon name="settings"></icon>
                          </button>
                        </row>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Soft · outline · ghost · toned</p>
                        <row gap="2" align="center" nowrap>
                          <button type="button" size="sm" variant="soft" tone="primary">Soft</button>
                          <button type="button" size="sm" variant="outline" tone="primary">Outline</button>
                          <button type="button" size="sm" variant="ghost" tone="muted">Ghost</button>
                          <button type="button" size="sm" variant={components.variant() || 'solid'} tone="primary">Active chip</button>
                        </row>
                      </column>
                    </column>
                  </if>
                  <if {components.stage.a()}>
                    <column gap="4" style(stageHost)>
                      <column gap="2">
                        <p style(fieldHint)>Bare text · + rest button-links (primary / foreground)</p>
                        <row gap="2" align="center" nowrap>
                          <a href="#">text link</a>
                          <a href="#" variant="default">default</a>
                          <a href="#" variant="soft">soft</a>
                          <a href="#" variant="outline">outline</a>
                          <a href="#" variant="ghost">ghost</a>
                        </row>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Toned button-links · active chip</p>
                        <row gap="2" align="center" nowrap>
                          <a href="#" variant={components.variant() || 'solid'} tone="primary">Primary</a>
                          <a href="#" variant="solid" tone="success">Success</a>
                          <a href="#" variant="outline" tone="warning">Warn</a>
                          <a href="#" variant="ghost" tone="danger">Danger</a>
                          <a href="#" variant="soft" tone="accent">Accent</a>
                        </row>
                      </column>
                    </column>
                  </if>
                  <if {components.stage.input()}>
                    <column gap="3" style(sampleMax)>
                      <column gap="2">
                        <p style(fieldHint)>Text field · sizes · input fill + foreground + border</p>
                        <input type="text" size="sm" placeholder="sm field" value={components.sampleText()} on:input={components.onSampleInput} label="Sample field" />
                        <input type="text" size="md" placeholder="md field" />
                        <input type="text" size="lg" placeholder="lg field" />
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Invalid · disabled</p>
                        <input type="email" size="sm" value="not-an-email" invalid />
                        <input type="text" size="sm" placeholder="Disabled" disabled />
                      </column>
                    </column>
                  </if>
                  <if {components.stage.textarea()}>
                    <column gap="3" style(sampleMax)>
                      <column gap="2">
                        <p style(fieldHint)>Sizes · shared input fill</p>
                        <textarea size="xs" rows="2" placeholder="xs" />
                        <textarea size="sm" rows="2" placeholder="sm" />
                        <textarea size={components.variant() || 'md'} rows="3" placeholder="md (chip or default)" />
                        <textarea size="lg" rows="2" placeholder="lg" />
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Invalid · disabled</p>
                        <textarea invalid rows="2" placeholder="invalid → danger border" />
                        <textarea disabled rows="2" placeholder="disabled" />
                      </column>
                    </column>
                  </if>
                  <if {components.stage.select()}>
                    <column gap="3" style(sampleMax)>
                      <column gap="2">
                        <p style(fieldHint)>Sizes · shared input fill + chevron</p>
                        <select size="xs">
                          <option>xs</option>
                          <option>Engineering</option>
                        </select>
                        <select size="sm">
                          <option>sm</option>
                          <option>Engineering</option>
                        </select>
                        <select size={components.variant() || 'md'}>
                          <option>md</option>
                          <option>Design</option>
                        </select>
                        <select size="lg">
                          <option>lg</option>
                          <option>Research</option>
                        </select>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Rest / invalid / disabled</p>
                        <select>
                          <option>Rest field tokens</option>
                          <option>Design</option>
                        </select>
                        <select invalid>
                          <option>Invalid border</option>
                          <option>Engineering</option>
                        </select>
                        <select disabled>
                          <option>Disabled</option>
                        </select>
                      </column>
                      <p style(fieldHint)>
                        Input fill + foreground + border · hover border-strong · not combobox
                      </p>
                    </column>
                  </if>
                  <if {components.stage.combobox()}>
                    <combobox placeholder="Pick a fruit" style(sampleMax)>
                      <option value="apple">Apple</option>
                      <option value="mango">Mango</option>
                      <option value="plum">Plum</option>
                    </combobox>
                  </if>
                  <if {components.stage.checkbox()}>
                    <column gap="2" style(sampleMax)>
                      <p style(fieldHint)>Rest · sizes · input face + primary checked</p>
                      <row gap="3" align="center">
                        <checkbox checked />
                        <checkbox />
                        <checkbox disabled />
                        <checkbox size="lg" checked />
                      </row>
                    </column>
                  </if>
                  <if {components.stage.radio()}>
                    <column gap="2" style(sampleMax)>
                      <radio value="a">Option A</radio>
                      <radio value="b" disabled>Disabled option</radio>
                      <p style(fieldHint)>Foreground ink · value + disabled only</p>
                    </column>
                  </if>
                  <if {components.stage.radioGroup()}>
                    <radio-group label="Plan" name="theme-l1-plan" value="monthly" style(sampleMax)>
                      <radio value="monthly">Monthly</radio>
                      <radio value="yearly">Yearly</radio>
                    </radio-group>
                  </if>
                  <if {components.stage.switchToggle()}>
                    <column gap="3" style(sampleMax)>
                      <column gap="2">
                        <p style(fieldHint)>Sizes · muted track + primary checked</p>
                        <row gap="3" align="center">
                          <switch size="sm" checked></switch>
                          <switch size={components.variant() || 'md'} checked></switch>
                          <switch size="lg" checked></switch>
                        </row>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Off / on / disabled (rest leaves)</p>
                        <row gap="3" align="center">
                          <switch></switch>
                          <switch checked={components.notify()}></switch>
                          <switch disabled></switch>
                          <switch checked disabled></switch>
                        </row>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Tone · checked track only; thumb stays input face</p>
                        <row gap="3" align="center">
                          <switch tone="primary" checked></switch>
                          <switch tone="success" checked></switch>
                          <switch tone="warning" checked></switch>
                          <switch tone="danger" checked></switch>
                        </row>
                      </column>
                    </column>
                  </if>
                  <if {components.stage.slider()}>
                    <column gap="3" style(sampleMax)>
                      <column gap="2">
                        <p style(fieldHint)>Sizes · muted track + primary fill</p>
                        <slider value="40" min="0" max="100" size="sm"></slider>
                        <slider value={components.load()} min="0" max="100" size={components.variant() || 'md'}></slider>
                        <slider value="80" min="0" max="100" size="lg"></slider>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Disabled · native freeze + opacity</p>
                        <slider value="55" min="0" max="100" disabled></slider>
                      </column>
                      <p style(fieldHint)>Thumb face = input · rim = primary fill · single thumb only</p>
                    </column>
                  </if>
                  <if {components.stage.toggle() || components.stage.toggleGroup()}>
                    <!-- standalone toggle died; pressed-button lives only in toggle-group -->
                    <toggle-group variant={components.variant() || 'outline'}>
                      <toggle value="left">Left</toggle>
                      <toggle value="center">Center</toggle>
                      <toggle value="right">Right</toggle>
                    </toggle-group>
                  </if>
                  <if {components.stage.field()}>
                    <field style(sampleMax)>
                      <label slot="label" for="theme-l1-field">Name</label>
                      <input id="theme-l1-field" size="sm" placeholder="Ada Lovelace" />
                      <p slot="hint">Hint under the control</p>
                    </field>
                  </if>
                  <if {components.stage.label()}>
                    <column gap="3" style(sampleMax)>
                      <column gap="2">
                        <p style(fieldHint)>Rest · sizes · foreground + label weight</p>
                        <label for="theme-l1-label-sm" size="sm">Email sm</label>
                        <label for="theme-l1-label-md" size={components.variant() || 'md'}>Email md</label>
                        <label for="theme-l1-label-lg" size="lg">Email lg</label>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Required · danger marker</p>
                        <label for="theme-l1-label" size={components.variant() || 'md'} required>Email</label>
                        <input id="theme-l1-label" size="sm" placeholder="ada@example.com" />
                      </column>
                      <column gap="1">
                        <p style(fieldHint)>Optional (no marker)</p>
                        <label for="theme-l1-label-opt" size="sm">Nickname</label>
                        <input id="theme-l1-label-opt" size="sm" placeholder="optional" />
                      </column>
                    </column>
                  </if>
                  <if {components.stage.fieldset()}>
                    <column gap="3" style(sampleMax)>
                      <column gap="2">
                        <p style(fieldHint)>Rest · border role</p>
                        <fieldset style(sampleMax)>
                          <legend>Account</legend>
                          <column gap="2">
                            <input size="sm" placeholder="Name" />
                            <input size="sm" placeholder="Email" />
                          </column>
                        </fieldset>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Disabled · native freeze + opacity</p>
                        <fieldset disabled style(sampleMax)>
                          <legend>Billing (locked)</legend>
                          <column gap="2">
                            <input size="sm" placeholder="Card" />
                            <button type="button" variant="solid" size="sm">Save</button>
                          </column>
                        </fieldset>
                      </column>
                    </column>
                  </if>
                  <if {components.stage.legend()}>
                    <column gap="3" style(sampleMax)>
                      <p style(fieldHint)>Rest · foreground caption</p>
                      <fieldset>
                        <legend>Account details</legend>
                        <column gap="2">
                          <input size="sm" placeholder="Name" />
                          <input size="sm" placeholder="Email" />
                        </column>
                      </fieldset>
                      <fieldset>
                        <legend>Billing options</legend>
                        <p style(fieldHint)>Caption only — nest inside fieldset.</p>
                      </fieldset>
                    </column>
                  </if>
                  <if {components.stage.themePicker()}>
                    <theme-picker></theme-picker>
                  </if>

                  <!-- ── Feedback ───────────────────────────────────── -->
                  <if {components.stage.alert()}>
                    <column gap="3" style(stageHost)>
                      <column gap="2">
                        <p style(fieldHint)>Rest · tone roles (info / success / warning / danger)</p>
                        <alert style(stageHost)>
                          <strong slot="title">Notice</strong>
                          Rest paint uses theme tone roles.
                        </alert>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Toned · stripe + icon brand (host fill stays component leaves)</p>
                        <column gap="2">
                          <alert tone="info" style(stageHost)>
                            <strong slot="title">Heads up</strong>
                            Info stripe under theme paint.
                          </alert>
                          <alert tone="success" style(stageHost)>
                            <strong slot="title">Saved</strong>
                            Success stripe under theme paint.
                          </alert>
                          <alert tone="warning" style(stageHost)>
                            <strong slot="title">Check input</strong>
                            Warning is assertive liveRegion.
                          </alert>
                          <alert tone="danger" style(stageHost)>
                            <strong slot="title">Failed</strong>
                            Destructive is assertive liveRegion.
                          </alert>
                          <alert tone="muted" style(stageHost)>
                            <strong slot="title">Note</strong>
                            Muted keeps alert.stripe honesty.
                          </alert>
                        </column>
                      </column>
                    </column>
                  </if>
                  <if {components.stage.progressBar()}>
                    <column gap="3" style(sampleMax)>
                      <column gap="2">
                        <p style(fieldHint)>Track/fill · muted + primary</p>
                        <progress value="30" max="100" size="sm"></progress>
                        <progress value={components.load()} max="100"></progress>
                        <progress value="90" max="100" size="lg"></progress>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Indeterminate · omit value (native + PROGRESS_MAP)</p>
                        <progress max="100"></progress>
                      </column>
                    </column>
                  </if>
                  <if {components.stage.meter()}>
                    <column gap="3" style(sampleMax)>
                      <column gap="2">
                        <p style(fieldHint)>Rest · sizes · muted track + semantic bands</p>
                        <meter value="0.55" size="sm"></meter>
                        <meter value="0.55"></meter>
                        <meter value="0.55" size="lg"></meter>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Bands · low=33 high=66 optimum=100</p>
                        <meter value="20" min="0" max="100" low="33" high="66" optimum="100"></meter>
                        <meter value="50" min="0" max="100" low="33" high="66" optimum="100"></meter>
                        <meter value="85" min="0" max="100" low="33" high="66" optimum="100"></meter>
                      </column>
                    </column>
                  </if>
                  <if {components.stage.spinner()}>
                    <column gap="3" style(stageHost)>
                      <column gap="2">
                        <p style(fieldHint)>Rest · muted arc + track</p>
                        <row gap="4" align="center" nowrap>
                          <spinner size="sm"></spinner>
                          <spinner></spinner>
                          <spinner size="lg"></spinner>
                        </row>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Toned · size ladder + host tones</p>
                        <row gap="4" align="center" nowrap>
                          <spinner size="xs" tone="primary"></spinner>
                          <spinner size="sm" tone="info"></spinner>
                          <spinner size={components.variant() || 'md'} tone="success"></spinner>
                          <spinner size="lg" tone="danger"></spinner>
                          <spinner tone="accent"></spinner>
                        </row>
                      </column>
                    </column>
                  </if>
                  <if {components.stage.skeleton()}>
                    <column gap="3" style(sampleMax)>
                      <column gap="2">
                        <p style(fieldHint)>Sheen · muted fill</p>
                        <column gap="2">
                          <skeleton shape="text" size="md"></skeleton>
                          <skeleton shape="text" size="sm"></skeleton>
                          <skeleton shape="line" size="sm"></skeleton>
                        </column>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Shapes · circle + block</p>
                        <row gap="3" align="center">
                          <skeleton shape="circle" size="md"></skeleton>
                          <skeleton shape="circle" size="sm"></skeleton>
                          <!-- no preset: size lock -->
                          <skeleton shape="block" style({ size({ w: 24, min-h: 16 }) })></skeleton>
                        </row>
                      </column>
                    </column>
                  </if>
                  <if {components.stage.emptyState()}>
                    <column gap="3" style(sampleMax)>
                      <column gap="2">
                        <p style(fieldHint)>Rest · muted surface + border + title</p>
                        <empty-state size="sm" style(stageHost)>
                          <icon slot="icon" name="inbox"></icon>
                          <strong slot="title">Empty inbox</strong>
                          Messages you receive land here.
                        </empty-state>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Inline · dashed chrome + tone on icon</p>
                        <empty-state variant="inline" size="sm" tone="primary" style(stageHost)>
                          <icon slot="icon" name="inbox"></icon>
                          <strong slot="title">No results</strong>
                          Try another filter.
                        </empty-state>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Connecting · bare + status live region</p>
                        <empty-state variant="connecting" size="sm" style(stageHost)>
                          <spinner slot="icon" size="sm"></spinner>
                          <strong slot="title">Connecting…</strong>
                          Establishing a secure session.
                        </empty-state>
                      </column>
                    </column>
                  </if>
                  <if {components.stage.errorState()}>
                    <column gap="4" style(stageHost)>
                      <column gap="2">
                        <p style(fieldHint)>Rest · muted surface + border</p>
                        <error-state code="ERR_THEME" size="sm">
                          <strong slot="title">Could not load preview</strong>
                          Rest chrome — muted surface + danger icon.
                          <button slot="action" type="button" variant="outline" size="sm">Retry</button>
                        </error-state>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Inline · brand-danger border</p>
                        <error-state variant="inline" size="sm" code="ERR_INLINE">
                          Inline recovery row on muted fill.
                          <button slot="action" type="button" variant="ghost" size="sm">Retry</button>
                        </error-state>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Details + code chip</p>
                        <error-state size="sm" code="ERR_STACK">
                          <strong slot="title">Upstream failed</strong>
                          Stack collapses behind Show details.
                          <p slot="details">at fetchData (api.ts:42)
  Error: gateway timeout</p>
                          <button slot="action" type="button" variant="solid" tone="danger" size="sm">Retry</button>
                        </error-state>
                      </column>
                    </column>
                  </if>
                  <if {components.stage.errorBoundary()}>
                    <error-boundary message="Panel failed to load.">
                      <p style(fieldHint)>Healthy boundary is invisible — this is the wrapped tree.</p>
                    </error-boundary>
                  </if>
                  <if {components.stage.toast()}>
                    <toast tone="success" duration="0" style(sampleMax)>
                      <strong>Saved.</strong> Sticky toast (duration 0).
                    </toast>
                  </if>
                  <if {components.stage.toastRegion()}>
                    <!-- no preset: size lock + position -->
                    <column gap="2" style(toastRegionHost)>
                      <p style(fieldHint)>Toast region hosts stacked toasts (fixed in product).</p>
                      <toast tone="info" duration="0">Region child toast</toast>
                    </column>
                  </if>

                  <!-- ── Display ────────────────────────────────────── -->
                  <if {components.stage.badge()}>
                    <column gap="3" style(stageHost)>
                      <column gap="2">
                        <p style(fieldHint)>Rest · primary / muted tone roles</p>
                        <row gap="2" align="center" nowrap>
                          <badge variant="soft" size="sm">soft</badge>
                          <badge variant="solid" size="sm">solid</badge>
                          <badge variant="outline" size="sm">outline</badge>
                        </row>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Toned · soft + solid + pill</p>
                        <row gap="2" align="center" nowrap>
                          <badge variant="soft" tone="primary" size="sm">primary</badge>
                          <badge variant="soft" tone="success" size="sm">success</badge>
                          <badge variant="solid" tone="warning" size="sm" pill>warn</badge>
                          <badge variant="outline" tone="danger" size="sm">err</badge>
                          <badge variant="soft" tone="accent" size="sm" pill>12</badge>
                        </row>
                      </column>
                    </column>
                  </if>
                  <if {components.stage.chip()}>
                    <column gap="3" style(stageHost)>
                      <column gap="2">
                        <p style(fieldHint)>Rest · primary / muted / border</p>
                        <row gap="2" align="center" nowrap>
                          <chip variant="solid" size="sm">solid</chip>
                          <chip variant="soft" size="sm">soft</chip>
                          <chip variant="outline" size="sm">outline</chip>
                          <chip variant="ghost" size="sm">ghost</chip>
                        </row>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Toned · selected · removable</p>
                        <row gap="2" align="center" nowrap>
                          <chip variant={components.variant() || 'solid'} tone="primary" size="sm" selected>Selected</chip>
                          <chip variant="soft" tone="success" size="sm">success</chip>
                          <chip variant="outline" tone="warning" size="sm">warn</chip>
                          <chip variant="solid" tone="danger" size="sm" removable>err</chip>
                          <chip variant="soft" tone="accent" size="sm">12</chip>
                        </row>
                      </column>
                    </column>
                  </if>
                  <if {components.stage.card()}>
                    <card variant={components.variant() || 'default'} padding="sm" style(sampleMax)>
                      <column gap="2">
                        <p style(doorTitle)>Card surface</p>
                        <p style(fieldHint)>Judges the radius role when overridden.</p>
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
                    <column gap="3" style(stageHost)>
                      <column gap="2">
                        <p style(fieldHint)>Size ladder · theme icon scale (xs–xl)</p>
                        <row gap="4" align="center" nowrap>
                          <icon name="sparkles" size="xs" label="xs"></icon>
                          <icon name="sparkles" size="sm" label="sm"></icon>
                          <icon name="sparkles" size={components.variant() || 'md'} label="md"></icon>
                          <icon name="sparkles" size="lg" label="lg"></icon>
                          <icon name="sparkles" size="xl" label="xl"></icon>
                        </row>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>size ladder · currentColor ink</p>
                        <row gap="4" align="center" nowrap>
                          <icon name="refresh" label="Refreshing"></icon>
                          <row nowrap align="center" style({ fg(primary) })>
                            <icon name="heart" size="lg" label="primary ink"></icon>
                          </row>
                          <row nowrap align="center" style({ fg(danger) })>
                            <icon name="heart" size="lg" label="danger ink"></icon>
                          </row>
                        </row>
                      </column>
                    </column>
                  </if>
                  <if {components.stage.separator()}>
                    <column gap="3" style(sampleMax)>
                      <column gap="2">
                        <p style(fieldHint)>Rest · border role</p>
                        <separator></separator>
                        <separator variant="dashed"></separator>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Vertical · same face between labels</p>
                        <!-- no preset: size lock -->
                        <row gap="3" align="center" style({ size({ h: 10 }) })>
                          <em style(fieldHint)>A</em>
                          <separator orientation="vertical"></separator>
                          <em style(fieldHint)>B</em>
                          <separator orientation="vertical" variant="dashed"></separator>
                          <em style(fieldHint)>C</em>
                        </row>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Toned · solid + dashed (free tones, not rest leaf)</p>
                        <separator tone="primary"></separator>
                        <separator variant="dashed" tone="danger"></separator>
                      </column>
                    </column>
                  </if>
                  <if {components.stage.accordion()}>
                    <accordion style(sampleMax)>
                      <strong slot="trigger">Show details</strong>
                      <p style(fieldHint)>Accordion body content.</p>
                    </accordion>
                  </if>
                  <if {components.stage.kbd()}>
                    <column gap="3" style(sampleMax)>
                      <column gap="2">
                        <p style(fieldHint)>Rest · muted fill + foreground + border</p>
                        <row gap="2" align="center">
                          <kbd>Ctrl</kbd>
                          <em style(fieldHint)>+</em>
                          <kbd>K</kbd>
                        </row>
                      </column>
                      <column gap="2">
                        <p style(fieldHint)>Modifiers · mono keycap chrome</p>
                        <row gap="2" align="center">
                          <kbd>⌘</kbd>
                          <kbd>⇧</kbd>
                          <kbd>⌥</kbd>
                          <kbd>⌃</kbd>
                          <kbd>Esc</kbd>
                          <kbd>F12</kbd>
                        </row>
                      </column>
                      <p style(fieldHint)>
                        Press <kbd>Ctrl</kbd> + <kbd>Shift</kbd> + <kbd>P</kbd> in docs.
                      </p>
                    </column>
                  </if>
                  <if {components.stage.prose()}>
                    <column gap="3" style(sampleMax)>
                      <p style(fieldHint)>Rest · foreground</p>
                      <prose>
                        <h2>Typography scope</h2>
                        <p>
                          Body copy with <strong>strong</strong>, <em>emphasis</em>,
                          <code>inline()</code>, and <mark>mark</mark>.
                        </p>
                        <blockquote>A quotation carries its own rule.</blockquote>
                        <ul>
                          <li>List item one</li>
                          <li>List item two</li>
                        </ul>
                      </prose>
                    </column>
                  </if>
                  <if {components.stage.codeBlock()}>
                    <column gap="3" style(sampleMax)>
                      <p style(fieldHint)>Rest · surface fill + foreground</p>
                      <code-block
                        code={'const n = signal(0)\nconst double = derived(() => n() * 2)\n// .tok-* ink via --syntax-*'}
                        lang="ts"
                      ></code-block>
                      <code-block
                        code={'<button variant="solid" tone="primary">\n  Save\n</button>'}
                        lang="sail"
                      ></code-block>
                    </column>
                  </if>

                  <!-- ── Overlay ────────────────────────────────────── -->
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
                        <row nowrap align="center" style({ pad(3).fg(muted) })>
                          <p style({ type(mono) })>
                            Live ticker · muted band ·&nbsp;
                          </p>
                        </row>
                      </ticker>
                      <ticker style(sampleMax, { overflow(hidden).pad(2).bg(primary).radius(md) })>
                        <row nowrap align="center" style({ pad(3).fg(primary-foreground) })>
                          <p style({ type(body-sm) })>
                            Brand paint ·&nbsp;
                          </p>
                        </row>
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
              </column>
            </scroll-area>
          </column>
        </card>
      </column>
    </column>

    <!-- RIGHT: override inspector -->
    <scroll-area components-editor components-inspector style(panelInspector)>
      <column gap="5" style(inspectorCol)>
        <column gap="2">
          <p style(eyebrow)>Overrides</p>
          <p style(inspectorTitle)>
            {components.selectedLabel()}
          </p>
          <p style(fieldHint)>
            Edit the theme roles this face spends. Shared color and type roles — not per-component leaves.
          </p>
          <if {session.catalogHint()}>
            <p catalog-hint style(fieldHint)>Add to project to edit.</p>
          </if>
        </column>

        <column section="component-tokens" gap="2">
          <row align="baseline" justify="between" gap="2" nowrap>
            <p style(sectionTitle)>Theme roles</p>
            <p style(fieldHint)>shared by the theme</p>
          </row>
          <if {components.hasComponentKnobs()}>
            <column gap="2" component-knob-list>
              <for {components.componentKnobs() as tok} key={tok.key}>
                <row token-path={tok.path} align="center" gap="2" nowrap style(tokenRow)>
                  <if {tok.isColor}>
                    <stack token-chip style(tokenChip)>
                      <sail-color-picker token-path={tok.path} value={tok.hex} disabled={!session.canEdit()} variant="swatch" showHex={false} on-change={components.onComponentColor}></sail-color-picker>
                    </stack>
                  </if>
                  <column gap="0" style(growMin)>
                    <p style(doorTitle)>{tok.label}</p>
                    <p style(pathMeta)>{tok.hexLabel}{tok.isOverridden ? '' : ' · theme default'}</p>
                  </column>
                  <if {tok.isOverridden}>
                    <badge tone="primary" size="sm" variant="soft" pill token-layer={tok.layer} style(shrink0)>set</badge>
                    <button type="button" size="xs" variant="ghost" tone="muted" token-path={tok.path} on:click={components.onClearComponent} disabled={!session.canEdit()} style(shrink0)>
                      Clear
                    </button>
                  </if>
                  <if {!tok.isOverridden}>
                    <badge tone="muted" size="sm" variant="outline" pill token-layer={tok.layer} style(shrink0)>default</badge>
                  </if>
                </row>
              </for>
            </column>
          </if>
          <if {!components.hasComponentKnobs()}>
            <p style(fieldHint)>This face has no dedicated theme roles to edit.</p>
          </if>
        </column>

        <column section="global-tokens" gap="2">
          <row align="baseline" justify="between" gap="2" nowrap>
            <p style(sectionTitle)>Theme-wide</p>
            <p style(fieldHint)>shared by many components</p>
          </row>
          <if {components.hasGlobalKnobs()}>
            <column gap="2" global-knob-list>
              <for {components.globalKnobs() as tok} key={tok.key}>
                <row token-path={tok.path} align="center" gap="2" nowrap style(tokenRow)>
                  <if {tok.isColor}>
                    <stack token-chip style(tokenChip)>
                      <sail-color-picker token-path={tok.path} value={tok.hex} disabled={!session.canEdit()} variant="swatch" showHex={false} on-change={components.onGlobalColor}></sail-color-picker>
                    </stack>
                  </if>
                  <column gap="0" style(growMin)>
                    <p style(doorTitle)>{tok.label}</p>
                    <p style(pathMeta)>{tok.hexLabel}</p>
                  </column>
                  <if {tok.isOverridden}>
                    <badge tone="primary" size="sm" variant="soft" pill style(shrink0)>set</badge>
                  </if>
                </row>
              </for>
            </column>
          </if>
          <if {!components.hasGlobalKnobs()}>
            <p style(fieldHint)>No related theme-wide knobs for this face.</p>
          </if>
        </column>
      </column>
    </scroll-area>
  </row>

  <toast-region position="bottom-end" components-toast-region>
    <for {session.toastItems() as t} key={t.key}>
      <toast tone={t.tone} components-toast on-dismiss={session.onToastDismiss}>
        {t.message}
      </toast>
    </for>
  </toast-region>
</column>

