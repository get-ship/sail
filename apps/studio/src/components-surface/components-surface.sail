<!--
  <components-surface> — Library mode (project + packs).
  Composition: { session, library }. Inspect is a sibling route surface.
-->
<scope>
  style eyebrow = { fg(muted-foreground).type({ size: xs, weight: semibold, tracking: wide, case: uppercase }) }
  style pathMeta = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style muted = { fg(muted-foreground).type({ size: sm }) }
  style surfaceRoot = { grow(1).size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }).bg(background) }
  style workbench = { grow(1).size({h: full, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }) }
  style navPane = { grow(1).size({h: full, min-w: panel-min-w-md, min-h: 0, max-w: panel-max-w-md, basis: 0}).overflow({ all: hidden }).border({ edge: r, color: border-subtle, width: 1 }).bg(surface-0) }
  style navHead = { shrink(0).pad({ x: 4, t: 4, b: 3 }).border({ edge: b, color: border-subtle, width: 1 }) }
  style detailPane = { grow(1).size({h: full, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }).bg(background) }
  style detailHead = { shrink(0).pad({ x: 5, y: 4 }).border({ edge: b, color: border-subtle, width: 1 }).bg(surface-0) }
  style detailHeadMain = { size({ min-w: 0 }).overflow({ x: auto }) }
  style scrollGrow = { grow(1).size({h: full, min-h: 0, basis: 0}) }
  style listPad = { pad(3) }
  style loadRow = { shrink(0).pad({ x: 4, y: 2 }) }
  style shrink0 = { shrink(0) }
  style growMin = { grow(1).size({ min-w: 0 }) }
  style detailTitle = { type({ size: lg, weight: semibold, tracking: tight, wrap: truncate }) }
  style bodyPad = { pad(5).size({ min-w: 0 }) }
  style specimenChrome = { shrink(0).pad({ x: 3 }).size({ h: 8, min-w: 0, min-h: 8 }).border({ edge: b, color: border-subtle, width: 1 }).bg(surface-1) }
  style trafficDots = { shrink(0).size({ min-w: 12 }) }
  style trafficDot = { shrink(0).size({ w: 2, h: 2, min-w: 2, min-h: 2 }).radius(full) }
  style chromeTitle = { grow(1).size({ min-w: 0 }).fg(muted-foreground).type({ size: xs, wrap: truncate }) }
  style previewFrame = { size({ min-w: 0 }).overflow({ all: hidden }).elevation(lg) }
  style previewBody = { pad(6).size({ min-h: 32 }).bg(background) }
  style previewTitle = { type({ family: heading, size: md, weight: semibold }) }
  style previewHint = { fg(muted-foreground).type({ size: sm, leading: relaxed }) }
  style propHead = { pad({ y: 1 }).border({ edge: b, color: border-subtle, width: 1 }) }
  style propRow = { pad({ y: 2 }).border({ edge: b, color: border-subtle, width: 1 }) }
  style kvLabel = { shrink(0).size({ w: 24 }).fg(muted-foreground).type({ size: xs }) }
  style kvValue = { grow(1).size({ min-w: 0 }).type({ family: mono, size: sm, wrap: truncate }) }
  style footnote = { pad({ x: 1 }).fg(muted-foreground).type({ size: xs, leading: snug }) }
</scope>

<controller>
import { createComponentsSession } from '../compositions/components-session/components-session'
import { createComponentsLibrary } from '../compositions/components-library/components-library'
export default controller(() => {
  const session = createComponentsSession({ mode: 'library' })
  const library = createComponentsLibrary(session)
  return { session, library }
})
</controller>

<column surface="components" components-surface components-library gap="0" style(surfaceRoot)>
  <row components-workbench gap="0" nowrap align="stretch" style(workbench)>
    <column components-nav gap="0" style(navPane)>
      <column components-nav-head gap="2" style(navHead)>
        <row align="center" justify="between" nowrap gap="2">
          <p style(eyebrow)>Components</p>
          <button type="button" size="sm" variant="solid" tone="primary" new-component on:click={session.onNewOpen} style(shrink0)>
            New…
          </button>
        </row>
        <row mode-toggle gap="1" align="center" nowrap>
          <button type="button" size="sm" mode="library" variant="solid" tone="primary" pressed="true" on:click={() => session.onModeToggle('library')}>
            Library
          </button>
          <button type="button" size="sm" mode="inspect" variant="ghost" tone="muted" pressed="false" on:click={() => session.onModeToggle('inspect')}>
            Inspect
          </button>
        </row>
        <input
          type="search"
          size="sm"
          placeholder="Filter name, pack, path…"
          components-filter
          value={session.filterQuery()}
          on:input={session.onFilterInput}
        />
      </column>

      <if {library.isLoading()}>
        <row align="center" gap="2" style(loadRow)>
          <spinner size="sm"></spinner>
          <p style(muted)>{library.loadingLabel()}</p>
        </row>
      </if>

      <scroll-area components-list style(scrollGrow)>
        <column gap="3" style(listPad)>
          <column list-group="project" gap="1">
            <row align="center" justify="between" nowrap>
              <p style(eyebrow)>This project</p>
              <p style(pathMeta)>{library.projectCountLabel()}</p>
            </row>
            <if {library.emptyProject()}>
              <studio-empty title="No project components" size="sm">
                Author components under src/, or start a brief with New…
                <button slot="action" type="button" size="sm" variant="solid" tone="primary" new-component on:click={session.onNewOpen}>
                  New…
                </button>
              </studio-empty>
            </if>
            <for {library.projectRows() as row} key={row.key}>
              <studio-list-row
                component-row
                title={row.title}
                meta={row.meta}
                selected={row.selected}
                on:click={() => library.onListClick(row.id)}
              >
                <badge slot="trailing" tone={row.originTone} size="sm" variant="outline" pill style(shrink0)>
                  {row.originLabel}
                </badge>
              </studio-list-row>
            </for>
          </column>

          <column list-group="packs" gap="1">
            <row align="center" justify="between" nowrap>
              <p style(eyebrow)>Installed packs</p>
              <p style(pathMeta)>{library.packCountLabel()}</p>
            </row>
            <if {!library.hasPackRows()}>
              <p style(muted)>No installed packs</p>
            </if>
            <for {library.packRows() as row} key={row.key}>
              <studio-list-row
                component-row
                title={row.title}
                meta={row.meta}
                selected={row.selected}
                on:click={() => library.onListClick(row.id)}
              >
                <badge slot="trailing" tone="muted" size="sm" variant="outline" pill style(shrink0)>
                  pack
                </badge>
              </studio-list-row>
            </for>
          </column>

          <p l1-footnote style(footnote)>
            Element skins live under Themes → Components — not listed here.
            <button type="button" size="sm" variant="ghost" tone="muted" themes-components on:click={session.goThemesComponents}>
              Open Themes
            </button>
          </p>
        </column>
      </scroll-area>
    </column>

    <column components-detail gap="0" style(detailPane)>
      <column components-detail-head gap="3" style(detailHead)>
        <row align="start" justify="between" gap="3" nowrap style(detailHeadMain)>
          <column gap="1" style(growMin)>
            <p detail-title style(detailTitle)>
              {library.detailTitle()}
            </p>
            <p detail-path style(pathMeta)>{library.detailPath()}</p>
            <row gap="2" align="center" nowrap>
              <if {library.hasSelected()}>
                <badge tone={library.detailOriginTone()} size="sm" variant="soft" pill>{library.detailOriginLabel()}</badge>
                <badge tone="muted" size="sm" variant="outline" pill>{library.detailLayer()}</badge>
              </if>
            </row>
          </column>
          <row gap="2" align="center" nowrap style(shrink0)>
            <button
              type="button"
              size="sm"
              variant="solid"
              tone="primary"
              open-edit
              disabled={!library.canOpenInEdit()}
              title={library.canOpenInEdit() ? 'Open this component source in Edit' : 'No project path to open in Edit'}
              on:click={library.onOpenInEdit}
            >
              Open in Edit
            </button>
            <button
              type="button"
              size="sm"
              variant="default"
              start-thread
              disabled={!library.hasSelected()}
              title={library.hasSelected() ? 'Start a Work thread for this component' : 'Select a component first'}
              on:click={library.onStartThread}
            >
              Start thread
            </button>
            <button
              type="button"
              size="sm"
              variant="ghost"
              copy-brief
              disabled={!library.hasSelected()}
              title={library.hasSelected() ? 'Copy a structured brief for an agent' : 'Select a component first'}
              on:click={library.onCopyBrief}
            >
              Copy agent brief
            </button>
          </row>
        </row>
      </column>

      <scroll-area components-detail-body style(scrollGrow)>
        <column gap="4" style(bodyPad)>
          <if {session.showNewForm()}>
            <card new-form padding="md" style({ size({ min-w: 0 }) })>
              <column gap="3">
                <p style({ type({ size: md, weight: semibold }) })>New component</p>
                <p style(muted)>Creates a draft brief only — does not write files yet.</p>
                <field label="Name">
                  <input type="text" size="sm" placeholder="my-widget" value={session.newName()} on:input={session.onNewNameInput} />
                </field>
                <field label="Intent">
                  <textarea size="sm" rows="3" placeholder="What should this component do?" value={session.newIntent()} on:input={session.onNewIntentInput} />
                </field>
                <field label="Layer">
                  <select size="sm" value={session.newLayer()} on:change={session.onNewLayerChange}>
                    <option value="app">app</option>
                    <option value="L2">L2 pack</option>
                  </select>
                </field>
                <row gap="2" nowrap>
                  <button type="button" size="sm" variant="solid" tone="primary" create-brief on:click={session.onCreateBrief}>
                    Create brief
                  </button>
                  <button type="button" size="sm" variant="default" on:click={session.onStartThreadFromNew}>
                    Start thread
                  </button>
                </row>
              </column>
            </card>
          </if>

          <card components-preview preview-specimen padding="none" style(previewFrame)>
            <column gap="0">
              <row specimen-chrome align="center" gap="2" nowrap style(specimenChrome)>
                <row gap="2" align="center" nowrap hidden="true" style(trafficDots)>
                  <stack style(trafficDot, { bg(destructive) })></stack>
                  <stack style(trafficDot, { bg(warning) })></stack>
                  <stack style(trafficDot, { bg(success) })></stack>
                </row>
                <p style(chromeTitle)>
                  {library.previewTitle()}
                </p>
                <badge tone="muted" size="sm" variant="outline" pill style(shrink0)>preview</badge>
              </row>
              <column mode="dark" specimen-body gap="2" style(previewBody)>
                <p style(previewTitle)>{library.previewTitle()}</p>
                <p style(previewHint)>{library.previewHint()}</p>
              </column>
            </column>
          </card>

          <tabs components-tabs default-value="overview">
            <tab-list variant="line">
              <tab value="overview">Overview</tab>
              <tab value="contract">Contract</tab>
            </tab-list>
            <tab-panel value="overview">
              <column gap="3" style({ pad({ y: 3 }) })>
                <column gap="1">
                  <p style(eyebrow)>Intent</p>
                  <p detail-intent style({ type({ size: sm, leading: relaxed }) })>{library.detailIntent()}</p>
                </column>
                <column gap="2">
                  <p style(eyebrow)>Meta</p>
                  <for {library.kvRows() as kv} key={kv.key}>
                    <row kv-row gap="3" align="start" nowrap>
                      <p style(kvLabel)>{kv.label}</p>
                      <p style(kvValue)>{kv.value}</p>
                    </row>
                  </for>
                </column>
                <column gap="1">
                  <p style(eyebrow)>Composition</p>
                  <p style(pathMeta)>{library.compositionLabel()}</p>
                  <p style(muted)>Slots: {library.slotsLabel()}</p>
                </column>
              </column>
            </tab-panel>
            <tab-panel value="contract">
              <column gap="3" style({ pad({ y: 3 }) })>
                <if {library.schemaNote()}>
                  <p schema-note style(muted)>{library.schemaNote()}</p>
                </if>
                <if {!library.hasProps()}>
                  <studio-empty title="No props in catalog" size="sm">
                    Schema incomplete or undeclared attributes.
                  </studio-empty>
                </if>
                <if {library.hasProps()}>
                  <column props-table gap="1">
                    <row gap="3" nowrap style(propHead)>
                      <p style(eyebrow, { grow(1) })>Name</p>
                      <p style(eyebrow)>Type</p>
                      <p style(eyebrow)>Default</p>
                    </row>
                    <for {library.propRows() as prop} key={prop.key}>
                      <row prop-row gap="3" align="start" nowrap style(propRow)>
                        <column gap="0" style(growMin)>
                          <p style({ type({ family: mono, size: sm }) })>{prop.name}</p>
                          <p style({ fg(muted-foreground).type({ size: xs }) })>{prop.requiredLabel}</p>
                        </column>
                        <p style(pathMeta, { shrink(0) })>{prop.type}</p>
                        <p style(pathMeta, { shrink(0) })>{prop.defaultValue}</p>
                      </row>
                    </for>
                  </column>
                </if>
              </column>
            </tab-panel>
          </tabs>
        </column>
      </scroll-area>
    </column>
  </row>

  <toast-region position="bottom-end" components-toast-region>
    <for {session.toastItems() as t} key={t.key}>
      <toast tone={t.tone} components-toast on-dismiss={session.onToastDismiss}>
        {t.message}
      </toast>
    </for>
  </toast-region>
</column>
