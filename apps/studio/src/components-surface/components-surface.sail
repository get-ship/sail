<!--
  <components-surface> — Library mode (project + packs).
  Composition: { session, library }. Inspect is a sibling route surface.
-->
<scope>
  const eyebrow = style.fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide).fg(muted-foreground)
  const pathMeta = style.fontSize(xs).fontFamily(mono).fg(muted-foreground)
  const muted = style.fontSize(sm).fg(muted-foreground)
</scope>

<controller>
import { createComponentsSession } from '../components-session/components-session'
import { createComponentsLibrary } from '../components-library/components-library'
export default controller(() => {
  const session = createComponentsSession({ mode: 'library' })
  const library = createComponentsLibrary(session)
  return { session, library }
})
</controller>

<column
  surface="components"
  components-surface
  components-library
  gap="0"
  grow
  sx={style.minH(0).h(full).minW(0).overflow(hidden).bg(background)}
>
  <row
    components-workbench
    grow
    gap="0"
    nowrap
    align="stretch"
    sx={style.minH(0).basis(0).h(full).minW(0).overflow(hidden)}
  >
    <column
      components-nav
      grow
      gap="0"
      sx={style.minH(0).h(full).minW(panel-min-w-md).maxW(panel-max-w-md).basis(0).borderRWidth(1).borderR(border-subtle).bg(surface-0).overflow(hidden)}
    >
      <column components-nav-head shrink="0" gap="2" sx={style.padX(4).padT(4).padB(3).borderBWidth(1).borderB(border-subtle)}>
        <row align="center" justify="between" nowrap gap="2">
          <p sx={eyebrow}>Components</p>
          <button type="button" size="sm" variant="solid" tone="primary" new-component on:click={session.onNewOpen} shrink="0">
            New…
          </button>
        </row>
        <row mode-toggle gap="1" align="center" nowrap on:click={session.onModeToggle}>
          <button type="button" size="sm" mode="library" variant="solid" tone="primary" pressed="true">
            Library
          </button>
          <button type="button" size="sm" mode="inspect" variant="ghost" tone="muted" pressed="false">
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
        <row align="center" gap="2" shrink="0" sx={style.padX(4).padY(2)}>
          <spinner size="sm"></spinner>
          <p sx={muted}>{library.loadingLabel()}</p>
        </row>
      </if>

      <scroll-area components-list grow on:click={library.onListClick} sx={style.minH(0).basis(0).h(full)}>
        <column gap="3" sx={style.pad(3)}>
          <column list-group="project" gap="1">
            <row align="center" justify="between" nowrap>
              <p sx={eyebrow}>This project</p>
              <p sx={pathMeta}>{library.projectCountLabel()}</p>
            </row>
            <if {library.emptyProject()}>
              <studio-empty title="No project components" size="sm">
                Author components under src/, or start a brief with New…
                <button slot="action" type="button" size="sm" variant="solid" tone="primary" new-component on:click={session.onNewOpen}>
                  New…
                </button>
              </studio-empty>
            </if>
            <for {library.projectRows()} as="row" key={row.key}>
              <studio-list-row
                component-row
                component-id={row.id}
                title={row.title}
                meta={row.meta}
                selected={row.selected}
              >
                <badge slot="trailing" tone={row.originTone} size="sm" variant="outline" pill shrink="0">
                  {row.originLabel}
                </badge>
              </studio-list-row>
            </for>
          </column>

          <column list-group="packs" gap="1">
            <row align="center" justify="between" nowrap>
              <p sx={eyebrow}>Installed packs</p>
              <p sx={pathMeta}>{library.packCountLabel()}</p>
            </row>
            <if {!library.hasPackRows()}>
              <p sx={muted}>No installed packs</p>
            </if>
            <for {library.packRows()} as="row" key={row.key}>
              <studio-list-row
                component-row
                component-id={row.id}
                title={row.title}
                meta={row.meta}
                selected={row.selected}
              >
                <badge slot="trailing" tone="muted" size="sm" variant="outline" pill shrink="0">
                  pack
                </badge>
              </studio-list-row>
            </for>
          </column>

          <p l1-footnote sx={style.fontSize(xs).fg(muted-foreground).leading(snug).padX(1)}>
            Element skins live under Themes → Components — not listed here.
            <button type="button" size="sm" variant="ghost" tone="muted" themes-components on:click={session.goThemesComponents}>
              Open Themes
            </button>
          </p>
        </column>
      </scroll-area>
    </column>

    <column
      components-detail
      grow
      gap="0"
      sx={style.minH(0).h(full).minW(0).basis(0).bg(background).overflow(hidden)}
    >
      <column
        components-detail-head
        shrink="0"
        gap="3"
        sx={style.padX(5).padY(4).borderBWidth(1).borderB(border-subtle).bg(surface-0)}
      >
        <row align="start" justify="between" gap="3" nowrap>
          <column grow min-w="0" gap="1">
            <p detail-title sx={style.fontSize(lg).fontWeight(semibold).tracking(tight).truncate()}>
              {library.detailTitle()}
            </p>
            <p detail-path sx={pathMeta}>{library.detailPath()}</p>
            <row gap="2" align="center" nowrap>
              <if {library.hasSelected()}>
                <badge tone={library.detailOriginTone()} size="sm" variant="soft" pill>{library.detailOriginLabel()}</badge>
                <badge tone="muted" size="sm" variant="outline" pill>{library.detailLayer()}</badge>
              </if>
            </row>
          </column>
          <row gap="2" align="center" shrink="0" nowrap>
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

      <scroll-area components-detail-body grow sx={style.minH(0).basis(0).h(full)}>
        <column gap="4" sx={style.pad(5).minW(0)}>
          <if {session.showNewForm()}>
            <card new-form padding="md" sx={style.minW(0)}>
              <column gap="3">
                <p sx={style.fontSize(md).fontWeight(semibold)}>New component</p>
                <p sx={muted}>Creates a draft brief only — does not write files yet.</p>
                <field label="Name">
                  <input type="text" size="sm" placeholder="my-widget" value={session.newName()} on:input={session.onNewNameInput} />
                </field>
                <field label="Intent">
                  <textarea size="sm" rows="3" placeholder="What should this component do?" value={session.newIntent()} on:input={session.onNewIntentInput}></textarea>
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

          <card components-preview preview-specimen padding="none" sx={style.minW(0).overflow(hidden).shadow(lg)}>
            <column gap="0">
              <row
                specimen-chrome
                align="center"
                gap="2"
                nowrap
                shrink="0"
                sx={style.padX(3).h(8).minH(8).minW(0).borderBWidth(1).borderB(border-subtle).bg(surface-1)}
              >
                <row gap="2" align="center" nowrap shrink="0" hidden="true" sx={style.minW(12)}>
                  <stack shrink="0" sx={style.w(2).h(2).minW(2).minH(2).radius(full).bg(destructive)}></stack>
                  <stack shrink="0" sx={style.w(2).h(2).minW(2).minH(2).radius(full).bg(warning)}></stack>
                  <stack shrink="0" sx={style.w(2).h(2).minW(2).minH(2).radius(full).bg(success)}></stack>
                </row>
                <p grow min-w="0" sx={style.fontSize(xs).fg(muted-foreground).truncate()}>
                  {library.previewTitle()}
                </p>
                <badge tone="muted" size="sm" variant="outline" pill shrink="0">preview</badge>
              </row>
              <column mode="dark" specimen-body gap="2" sx={style.pad(6).minH(32).bg(background)}>
                <p sx={style.fontSize(md).fontWeight(semibold).fontFamily(heading)}>{library.previewTitle()}</p>
                <p sx={style.fontSize(sm).fg(muted-foreground).leading(relaxed)}>{library.previewHint()}</p>
              </column>
            </column>
          </card>

          <tabs components-tabs default-value="overview">
            <tab-list variant="line">
              <tab value="overview">Overview</tab>
              <tab value="contract">Contract</tab>
            </tab-list>
            <tab-panel value="overview">
              <column gap="3" sx={style.padY(3)}>
                <column gap="1">
                  <p sx={eyebrow}>Intent</p>
                  <p detail-intent sx={style.fontSize(sm).leading(relaxed)}>{library.detailIntent()}</p>
                </column>
                <column gap="2">
                  <p sx={eyebrow}>Meta</p>
                  <for {library.kvRows()} as="kv" key={kv.key}>
                    <row kv-row gap="3" align="start" nowrap>
                      <p shrink="0" sx={style.fontSize(xs).fg(muted-foreground).w(24)}>{kv.label}</p>
                      <p grow min-w="0" sx={style.fontSize(sm).fontFamily(mono).truncate()}>{kv.value}</p>
                    </row>
                  </for>
                </column>
                <column gap="1">
                  <p sx={eyebrow}>Composition</p>
                  <p sx={pathMeta}>{library.compositionLabel()}</p>
                  <p sx={muted}>Slots: {library.slotsLabel()}</p>
                </column>
              </column>
            </tab-panel>
            <tab-panel value="contract">
              <column gap="3" sx={style.padY(3)}>
                <if {library.schemaNote()}>
                  <p schema-note sx={muted}>{library.schemaNote()}</p>
                </if>
                <if {!library.hasProps()}>
                  <studio-empty title="No props in catalog" size="sm">
                    Schema incomplete or undeclared attributes.
                  </studio-empty>
                </if>
                <if {library.hasProps()}>
                  <column props-table gap="1">
                    <row gap="3" nowrap sx={style.padY(1).borderBWidth(1).borderB(border-subtle)}>
                      <p grow sx={eyebrow}>Name</p>
                      <p sx={eyebrow}>Type</p>
                      <p sx={eyebrow}>Default</p>
                    </row>
                    <for {library.propRows()} as="prop" key={prop.key}>
                      <row prop-row gap="3" align="start" nowrap sx={style.padY(2).borderBWidth(1).borderB(border-subtle)}>
                        <column grow min-w="0" gap="0">
                          <p sx={style.fontSize(sm).fontFamily(mono)}>{prop.name}</p>
                          <p sx={style.fontSize(xs).fg(muted-foreground)}>{prop.requiredLabel}</p>
                        </column>
                        <p shrink="0" sx={pathMeta}>{prop.type}</p>
                        <p shrink="0" sx={pathMeta}>{prop.defaultValue}</p>
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
    <for {session.toastItems()} as="t" key={t.key}>
      <toast tone={t.tone} components-toast on:sail-toast-dismiss={session.onToastDismiss}>
        {t.message}
      </toast>
    </for>
  </toast-region>
</column>
