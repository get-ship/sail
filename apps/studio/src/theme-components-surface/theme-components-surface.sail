<!--
  Themes → Components facet — component token overrides (not the Components entity page).
  Catalog rail | live stage | override inspector. Overrides write component.* leaves.
-->
<scope>
  style sectionTitle = { text: fontSize(sm).fontWeight(semibold) }
  style fieldHint = { paint: fg(muted-foreground), text: fontSize(xs) }
  style pathMeta = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style doorTitle = { text: fontWeight(semibold) }
  style bandStatus = { box: padX(4).padY(2) }
  style bandError = { box: padX(4).padY(2), paint: borderBWidth(1).borderB(border) }
  style railCol = { box: padX(3).padT(4).padB(6).minW(0) }
  style inspectorCol = { box: padX(4).padT(4).padB(8).minW(0) }
  style fullBtn = { box: w(full) }
  style chipCol = { box: minW(0) }
  // Selected rail row: soft fill + subtle border.
  style listRow = { box: padX(2_5).padY(2), paint: radius(md).cursor(pointer).borderWidth(1).border(transparent), when: { hover: { paint: bg(hover-bg) }, selected: { paint: bg(surface-2).border(border-subtle) } } }
  style eyebrow = { paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).tracking(wide).textCase(uppercase) }
  style tokenStrip = { box: padX(2).padY(1), paint: fg(muted-foreground).radius(sm).bg(surface-0), text: fontSize(xs).fontFamily(mono) }
  style tokenRow = { box: padY(1_5), paint: borderBWidth(1).borderB(border-subtle) }
  style surfaceRoot = { box: grow(1).minH(0).h(full).minW(0).overflow(hidden), paint: bg(background) }
  style workbench = { box: grow(1).minH(0).basis(0).h(full).minW(0).overflow(hidden), paint: bg(background) }
  style panelRail = { box: shrink(0).minW(panel-min-w-sm).maxW(panel-max-w-sm).w(panel-max-w-sm).minH(0).h(full), paint: borderRWidth(1).borderR(border-subtle).bg(background) }
  style panelInspector = { box: shrink(0).minW(panel-min-w-sm).maxW(panel-max-w-sm).w(panel-max-w-sm).minH(0).h(full), paint: borderLWidth(1).borderL(border-subtle).bg(background) }
  style specimenDock = { box: grow(1).minW(0).basis(0).minH(0).h(full).overflow(hidden), paint: bg(background) }
  style specimenPad = { box: grow(1).minH(0).h(full).pad(4) }
  style specimenFrame = { box: minW(0).minH(0).h(full).overflow(hidden), paint: shadow(lg) }
  style specimenChrome = { box: shrink(0).padX(3).h(8).minH(8).minW(0), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1) }
  style specimenVariants = { box: shrink(0).padX(3).padY(2).minW(0), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1) }
  style trafficDots = { box: shrink(0).minW(12) }
  style trafficDot = { box: shrink(0).w(2).h(2).minW(2).minH(2), paint: radius(full) }
  style chromeTitle = { box: grow(1).minW(0), paint: fg(muted-foreground), text: fontSize(xs).truncate() }
  style errorText = { box: grow(1).minW(0), paint: fg(destructive), text: fontSize(sm).truncate() }
  style growMin = { box: grow(1).minW(0) }
  style shrink0 = { box: shrink(0) }
  style tapFloor = { box: shrink(0).minW(control-h-sm).minH(control-h-sm) }
  style iconBtn = { box: shrink(0).w(control-h-sm).h(control-h-sm).minW(control-h-sm).minH(control-h-sm) }
  style stageHost = { box: w(full) }
  style sampleMax = { box: w(full).maxW(panel-max-w-sm) }
  style scrollGrow = { box: grow(1).minH(0).basis(0).h(full), paint: bg(background) }
  style stageBody = { box: pad(5).minW(0).minH(full) }
  style stageTitle = { text: fontSize(md).fontWeight(semibold).fontFamily(heading) }
  style stageBlurb = { paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed).fontFamily(body) }
  style inspectorTitle = { text: fontSize(xl).fontWeight(semibold).tracking(tight).fontFamily(heading) }
  style tokenChip = { box: shrink(0).position(relative).w(9).h(9).minW(9).minH(9).overflow(hidden), paint: radius(md).borderWidth(1).border(border-subtle).bg(surface-0) }
  type ComponentRailRowProps = {
    id: string
    label: string
    hint: string
    selected: boolean
    hasOverride: boolean
  }
</scope>

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

<controller>
import { createThemeSession } from '../compositions/theme-session/theme-session'
import { watchPreviewHost } from '../compositions/theme-session/bind-preview-host'
import { createComponentsEdit } from '../compositions/theme-components-surface/theme-components-edit'
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

<column surface="themes" theme-components-surface components-chrome gap="0" style(surfaceRoot)>
  <theme-facet-chrome session={session}></theme-facet-chrome>

  <if {session.hasError()}>
    <row components-error align="center" gap="2" style(bandError, { box: shrink(0) })>
      <p style(errorText)>{session.errorText()}</p>
      <button type="button" size="sm" variant="ghost" components-retry on:click={session.onRetry}>Retry</button>
    </row>
  </if>

  <if {session.loading()}>
    <row components-loading align="center" gap="2" style(bandStatus, { box: shrink(0) })>
      <spinner size="sm"></spinner>
      <p style(fieldHint)>Loading…</p>
    </row>
  </if>

  <row components-workbench gap="0" nowrap align="stretch" style(workbench)>
    <scroll-area components-rail style(panelRail)>
      <column gap="3" style(railCol)>
        <column gap="2">
          <p style(eyebrow)>Components</p>
          <p style({ text: fontSize(md).fontWeight(semibold).tracking(tight) })>Components</p>
          <p l1-facet-blurb style(fieldHint)>
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
        <!-- Preview host: draft paint via applyScope (theme + data-mode). No free mode= attr. -->
        <card
          components-specimen
          preview-specimen
          padding="none" style(specimenFrame)>
          <column gap="0" style({  box: grow(1).minH(0).h(full)  })>
            <row preview-chrome align="center" gap="2" nowrap style(specimenChrome)>
              <row gap="2" align="center" nowrap hidden="true" style(trafficDots)>
                <stack style(trafficDot, { paint: bg(destructive) })></stack>
                <stack style(trafficDot, { paint: bg(warning) })></stack>
                <stack style(trafficDot, { paint: bg(success) })></stack>
              </row>
              <p style(chromeTitle)>
                Preview · {session.nameEdit()} · {components.selectedLabel()}
              </p>
              <p style(pathMeta, { box: shrink(0) })>{session.paintMode()}</p>
            </row>

            <if {components.hasVariants()}>
              <row specimen-variants align="center" gap="2" nowrap on:click={components.onVariantPick} style(specimenVariants)>
                <p style(pathMeta, { box: shrink(0) })>{components.viewerVariantAttr()}:</p>
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

                <column gap="4" align="start" l1-stage-host style(stageHost)>
                  <theme-l1-inputs components={components}></theme-l1-inputs>
                  <theme-l1-feedback components={components}></theme-l1-feedback>
                  <theme-l1-display components={components}></theme-l1-display>
                  <theme-l1-overlay components={components}></theme-l1-overlay>
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
            Edit colors for this face. Defaults come from the theme; “set” means a component.* exception.
          </p>
          <if {session.catalogHint()}>
            <p catalog-hint style(fieldHint)>Add to project to edit.</p>
          </if>
        </column>
        <column section="component-tokens" gap="2">
          <row align="baseline" justify="between" gap="2" nowrap>
            <p style(sectionTitle)>This component</p>
            <p style(fieldHint)>overrides theme defaults</p>
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
            <p style(fieldHint)>No component.* leaves for this face.</p>
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

