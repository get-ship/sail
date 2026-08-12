<!--
  <components-surface> — Library mode (project + packs).
  Composition: { session, library }. Inspect is a sibling route surface.
-->
<scope>
  style eyebrow = { paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide) }
  style pathMeta = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style muted = { paint: fg(muted-foreground), text: fontSize(sm) }
  style surfaceRoot = { box: grow(1).minH(0).h(full).minW(0).overflow(hidden), paint: bg(background) }
  style workbench = { box: grow(1).minH(0).basis(0).h(full).minW(0).overflow(hidden) }
  style navPane = { box: grow(1).minH(0).h(full).minW(panel-min-w-md).maxW(panel-max-w-md).basis(0).overflow(hidden), paint: borderRWidth(1).borderR(border-subtle).bg(surface-0) }
  style navHead = { box: shrink(0).padX(4).padT(4).padB(3), paint: borderBWidth(1).borderB(border-subtle) }
  style detailPane = { box: grow(1).minH(0).h(full).minW(0).basis(0).overflow(hidden), paint: bg(background) }
  style detailHead = { box: shrink(0).padX(5).padY(4), paint: borderBWidth(1).borderB(border-subtle).bg(surface-0) }
  style scrollGrow = { box: grow(1).minH(0).basis(0).h(full) }
  style listPad = { box: pad(3) }
  style loadRow = { box: shrink(0).padX(4).padY(2) }
  style shrink0 = { box: shrink(0) }
  style growMin = { box: grow(1).minW(0) }
  style detailTitle = { text: fontSize(lg).fontWeight(semibold).tracking(tight).truncate() }
  style bodyPad = { box: pad(5).minW(0) }
  style specimenChrome = { box: shrink(0).padX(3).h(8).minH(8).minW(0), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1) }
  style trafficDots = { box: shrink(0).minW(12) }
  style trafficDot = { box: shrink(0).w(2).h(2).minW(2).minH(2), paint: radius(full) }
  style chromeTitle = { box: grow(1).minW(0), paint: fg(muted-foreground), text: fontSize(xs).truncate() }
  style previewFrame = { box: minW(0).overflow(hidden), paint: shadow(lg) }
  style previewBody = { box: pad(6).minH(32), paint: bg(background) }
  style previewTitle = { text: fontSize(md).fontWeight(semibold).fontFamily(heading) }
  style previewHint = { paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) }
  style propHead = { box: padY(1), paint: borderBWidth(1).borderB(border-subtle) }
  style propRow = { box: padY(2), paint: borderBWidth(1).borderB(border-subtle) }
  style kvLabel = { box: shrink(0).w(24), paint: fg(muted-foreground), text: fontSize(xs) }
  style kvValue = { box: grow(1).minW(0), text: fontSize(sm).fontFamily(mono).truncate() }
  style footnote = { box: padX(1), paint: fg(muted-foreground), text: fontSize(xs).leading(snug) }
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
        <row align="start" justify="between" gap="3" nowrap>
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
            <card new-form padding="md" style({ box: minW(0) })>
              <column gap="3">
                <p style({ text: fontSize(md).fontWeight(semibold) })>New component</p>
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
                  <stack style(trafficDot, { paint: bg(destructive) })></stack>
                  <stack style(trafficDot, { paint: bg(warning) })></stack>
                  <stack style(trafficDot, { paint: bg(success) })></stack>
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
              <column gap="3" style({ box: padY(3) })>
                <column gap="1">
                  <p style(eyebrow)>Intent</p>
                  <p detail-intent style({ text: fontSize(sm).leading(relaxed) })>{library.detailIntent()}</p>
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
              <column gap="3" style({ box: padY(3) })>
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
                      <p style(eyebrow, { box: grow(1) })>Name</p>
                      <p style(eyebrow)>Type</p>
                      <p style(eyebrow)>Default</p>
                    </row>
                    <for {library.propRows() as prop} key={prop.key}>
                      <row prop-row gap="3" align="start" nowrap style(propRow)>
                        <column gap="0" style(growMin)>
                          <p style({ text: fontSize(sm).fontFamily(mono) })>{prop.name}</p>
                          <p style({ paint: fg(muted-foreground), text: fontSize(xs) })>{prop.requiredLabel}</p>
                        </column>
                        <p style(pathMeta, { box: shrink(0) })>{prop.type}</p>
                        <p style(pathMeta, { box: shrink(0) })>{prop.defaultValue}</p>
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
