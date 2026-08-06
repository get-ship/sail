<!--
  <components-surface> — Library mode (project + packs).
  Composition: { session, library }. Inspect is a sibling route surface.
-->
<scope>
  const eyebrow = style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide) })
  const pathMeta = style({ paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) })
  const muted = style({ paint: fg(muted-foreground), text: fontSize(sm) })
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

>
  style({ box: minH(0).h(full).minW(0).overflow(hidden), paint: bg(background) })

  <row
    components-workbench
    grow
    gap="0"
    nowrap
    align="stretch"

  >
    style({ box: minH(0).basis(0).h(full).minW(0).overflow(hidden) })

    <column
      components-nav
      grow
      gap="0"

    >
      style({ box: minH(0).h(full).minW(panel-min-w-md).maxW(panel-max-w-md).basis(0).overflow(hidden), paint: borderRWidth(1).borderR(border-subtle).bg(surface-0) })

      <column components-nav-head shrink="0" gap="2">
        style({ box: padX(4).padT(4).padB(3), paint: borderBWidth(1).borderB(border-subtle) })

        <row align="center" justify="between" nowrap gap="2">
          <p>
            style(eyebrow)
Components</p>
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
        <row align="center" gap="2" shrink="0">
          style({ box: padX(4).padY(2) })

          <spinner size="sm"></spinner>
          <p>
            style(muted)
{library.loadingLabel()}</p>
        </row>
      </if>

      <scroll-area components-list grow on:click={library.onListClick}>
        style({ box: minH(0).basis(0).h(full) })

        <column gap="3">
          style({ box: pad(3) })

          <column list-group="project" gap="1">
            <row align="center" justify="between" nowrap>
              <p>
                style(eyebrow)
This project</p>
              <p>
                style(pathMeta)
{library.projectCountLabel()}</p>
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
              <p>
                style(eyebrow)
Installed packs</p>
              <p>
                style(pathMeta)
{library.packCountLabel()}</p>
            </row>
            <if {!library.hasPackRows()}>
              <p>
                style(muted)
No installed packs</p>
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

          <p l1-footnote>
            style({ box: padX(1), paint: fg(muted-foreground), text: fontSize(xs).leading(snug) })

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

    >
      style({ box: minH(0).h(full).minW(0).basis(0).overflow(hidden), paint: bg(background) })

      <column
        components-detail-head
        shrink="0"
        gap="3"

      >
        style({ box: padX(5).padY(4), paint: borderBWidth(1).borderB(border-subtle).bg(surface-0) })

        <row align="start" justify="between" gap="3" nowrap>
          <column grow min-w="0" gap="1">
            <p detail-title>
              style({ text: fontSize(lg).fontWeight(semibold).tracking(tight).truncate() })

              {library.detailTitle()}
            </p>
            <p detail-path>
              style(pathMeta)
{library.detailPath()}</p>
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

      <scroll-area components-detail-body grow>
        style({ box: minH(0).basis(0).h(full) })

        <column gap="4">
          style({ box: pad(5).minW(0) })

          <if {session.showNewForm()}>
            <card new-form padding="md">
              style({ box: minW(0) })

              <column gap="3">
                <p>
                  style({ text: fontSize(md).fontWeight(semibold) })
New component</p>
                <p>
                  style(muted)
Creates a draft brief only — does not write files yet.</p>
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

          <card components-preview preview-specimen padding="none">
            style({ box: minW(0).overflow(hidden), paint: shadow(lg) })

            <column gap="0">
              <row
                specimen-chrome
                align="center"
                gap="2"
                nowrap
                shrink="0"

              >
                style({ box: padX(3).h(8).minH(8).minW(0), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1) })

                <row gap="2" align="center" nowrap shrink="0" hidden="true">
                  style({ box: minW(12) })

                  <stack shrink="0">
                    style({ box: w(2).h(2).minW(2).minH(2), paint: radius(full).bg(destructive) })
</stack>
                  <stack shrink="0">
                    style({ box: w(2).h(2).minW(2).minH(2), paint: radius(full).bg(warning) })
</stack>
                  <stack shrink="0">
                    style({ box: w(2).h(2).minW(2).minH(2), paint: radius(full).bg(success) })
</stack>
                </row>
                <p grow min-w="0">
                  style({ paint: fg(muted-foreground), text: fontSize(xs).truncate() })

                  {library.previewTitle()}
                </p>
                <badge tone="muted" size="sm" variant="outline" pill shrink="0">preview</badge>
              </row>
              <column mode="dark" specimen-body gap="2">
                style({ box: pad(6).minH(32), paint: bg(background) })

                <p>
                  style({ text: fontSize(md).fontWeight(semibold).fontFamily(heading) })
{library.previewTitle()}</p>
                <p>
                  style({ paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) })
{library.previewHint()}</p>
              </column>
            </column>
          </card>

          <tabs components-tabs default-value="overview">
            <tab-list variant="line">
              <tab value="overview">Overview</tab>
              <tab value="contract">Contract</tab>
            </tab-list>
            <tab-panel value="overview">
              <column gap="3">
                style({ box: padY(3) })

                <column gap="1">
                  <p>
                    style(eyebrow)
Intent</p>
                  <p detail-intent>
                    style({ text: fontSize(sm).leading(relaxed) })
{library.detailIntent()}</p>
                </column>
                <column gap="2">
                  <p>
                    style(eyebrow)
Meta</p>
                  <for {library.kvRows()} as="kv" key={kv.key}>
                    <row kv-row gap="3" align="start" nowrap>
                      <p shrink="0">
                        style({ box: w(24), paint: fg(muted-foreground), text: fontSize(xs) })
{kv.label}</p>
                      <p grow min-w="0">
                        style({ text: fontSize(sm).fontFamily(mono).truncate() })
{kv.value}</p>
                    </row>
                  </for>
                </column>
                <column gap="1">
                  <p>
                    style(eyebrow)
Composition</p>
                  <p>
                    style(pathMeta)
{library.compositionLabel()}</p>
                  <p>
                    style(muted)
Slots: {library.slotsLabel()}</p>
                </column>
              </column>
            </tab-panel>
            <tab-panel value="contract">
              <column gap="3">
                style({ box: padY(3) })

                <if {library.schemaNote()}>
                  <p schema-note>
                    style(muted)
{library.schemaNote()}</p>
                </if>
                <if {!library.hasProps()}>
                  <studio-empty title="No props in catalog" size="sm">
                    Schema incomplete or undeclared attributes.
                  </studio-empty>
                </if>
                <if {library.hasProps()}>
                  <column props-table gap="1">
                    <row gap="3" nowrap>
                      style({ box: padY(1), paint: borderBWidth(1).borderB(border-subtle) })

                      <p grow>
                        style(eyebrow)
Name</p>
                      <p>
                        style(eyebrow)
Type</p>
                      <p>
                        style(eyebrow)
Default</p>
                    </row>
                    <for {library.propRows()} as="prop" key={prop.key}>
                      <row prop-row gap="3" align="start" nowrap>
                        style({ box: padY(2), paint: borderBWidth(1).borderB(border-subtle) })

                        <column grow min-w="0" gap="0">
                          <p>
                            style({ text: fontSize(sm).fontFamily(mono) })
{prop.name}</p>
                          <p>
                            style({ paint: fg(muted-foreground), text: fontSize(xs) })
{prop.requiredLabel}</p>
                        </column>
                        <p shrink="0">
                          style(pathMeta)
{prop.type}</p>
                        <p shrink="0">
                          style(pathMeta)
{prop.defaultValue}</p>
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
