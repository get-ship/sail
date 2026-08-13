<!--
  Themes → Components facet — component token overrides (not the Components entity page).
  Catalog rail | live stage | override inspector. Overrides write component.* leaves.
-->
<scope>
  style sectionTitle = { text: type({ size: sm, weight: semibold }) }
  style fieldHint = { paint: ink({ color: muted-foreground }), text: type({ size: xs }) }
  style pathMeta = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style doorTitle = { text: type({ weight: semibold }) }
  style bandStatus = { box: space({ pad-x: 4, pad-y: 2 }) }
  style bandError = { box: space({ pad-x: 4, pad-y: 2 }), paint: border({ edge: b, color: border, width: 1 }) }
  style railCol = { box: space({ pad-x: 3, pad-t: 4, pad-b: 6 }).size({ min-w: 0 }) }
  style inspectorCol = { box: space({ pad-x: 4, pad-t: 4, pad-b: 8 }).size({ min-w: 0 }) }
  style fullBtn = { box: size({ w: full }) }
  style chipCol = { box: size({ min-w: 0 }) }
  // Selected rail row: soft fill + subtle border.
  style listRow = { box: space({ pad-x: 2_5, pad-y: 2 }), paint: radius(md).cursor(pointer).border({ color: transparent, width: 1 }), when: { hover: { paint: fill({ color: hover-bg }) }, selected: { paint: fill({ color: surface-2 }).border({ color: border-subtle }) } } }
  style eyebrow = { paint: ink({ color: muted-foreground }), text: type({ size: xs, weight: semibold, tracking: wide, case: uppercase }) }
  style tokenStrip = { box: space({ pad-x: 2, pad-y: 1 }), paint: ink({ color: muted-foreground }).radius(sm).fill({ color: surface-0 }), text: type({ family: mono, size: xs }) }
  style tokenRow = { box: space({ pad-y: 1_5 }), paint: border({ edge: b, color: border-subtle, width: 1 }) }
  style surfaceRoot = { box: flow({ grow: 1 }).size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }), paint: fill({ color: background }) }
  style workbench = { box: flow({ grow: 1 }).size({h: full, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }), paint: fill({ color: background }) }
  style panelRail = { box: flow({ shrink: 0 }).size({ w: panel-max-w-sm, h: full, min-w: panel-min-w-sm, min-h: 0, max-w: panel-max-w-sm }), paint: border({ edge: r, color: border-subtle, width: 1 }).fill({ color: background }) }
  style panelInspector = { box: flow({ shrink: 0 }).size({ w: panel-max-w-sm, h: full, min-w: panel-min-w-sm, min-h: 0, max-w: panel-max-w-sm }), paint: border({ edge: l, color: border-subtle, width: 1 }).fill({ color: background }) }
  style specimenDock = { box: flow({ grow: 1 }).size({h: full, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }), paint: fill({ color: background }) }
  style specimenPad = { box: flow({ grow: 1 }).size({ h: full, min-h: 0 }).space({ pad: 4 }) }
  style specimenFrame = { box: size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }), paint: elevation(lg) }
  style specimenChrome = { box: flow({ shrink: 0 }).space({ pad-x: 3 }).size({ h: 8, min-w: 0, min-h: 8 }), paint: border({ edge: b, color: border-subtle, width: 1 }).fill({ color: surface-1 }) }
  style specimenVariants = { box: flow({ shrink: 0 }).space({ pad-x: 3, pad-y: 2 }).size({ min-w: 0 }), paint: border({ edge: b, color: border-subtle, width: 1 }).fill({ color: surface-1 }) }
  style trafficDots = { box: flow({ shrink: 0 }).size({ min-w: 12 }) }
  style trafficDot = { box: flow({ shrink: 0 }).size({ w: 2, h: 2, min-w: 2, min-h: 2 }), paint: radius(full) }
  style chromeTitle = { box: flow({ grow: 1 }).size({ min-w: 0 }), paint: ink({ color: muted-foreground }), text: type({ size: xs, wrap: truncate }) }
  style errorText = { box: flow({ grow: 1 }).size({ min-w: 0 }), paint: ink({ color: destructive }), text: type({ size: sm, wrap: truncate }) }
  style growMin = { box: flow({ grow: 1 }).size({ min-w: 0 }) }
  style shrink0 = { box: flow({ shrink: 0 }) }
  style tapFloor = { box: flow({ shrink: 0 }).size({ min-w: control-h-sm, min-h: control-h-sm }) }
  style iconBtn = { box: flow({ shrink: 0 }).size({ w: control-h-sm, h: control-h-sm, min-w: control-h-sm, min-h: control-h-sm }) }
  style stageHost = { box: size({ w: full }) }
  style sampleMax = { box: size({ w: full, max-w: panel-max-w-sm }) }
  style scrollGrow = { box: flow({ grow: 1 }).size({h: full, min-h: 0, basis: 0}), paint: fill({ color: background }) }
  style stageBody = { box: space({ pad: 5 }).size({ min-w: 0, min-h: full }) }
  style stageTitle = { text: type({ family: heading, size: md, weight: semibold }) }
  style stageBlurb = { paint: ink({ color: muted-foreground }), text: type({ family: body, size: sm, leading: relaxed }) }
  style inspectorTitle = { text: type({ family: heading, size: xl, weight: semibold, tracking: tight }) }
  style tokenChip = { box: flow({ shrink: 0 }).place({ position: relative }).size({ w: 9, h: 9, min-w: 9, min-h: 9 }).overflow({ all: hidden }), paint: radius(md).border({ color: border-subtle, width: 1 }).fill({ color: surface-0 }) }
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
    <row components-error align="center" gap="2" style(bandError, { box: flow({ shrink: 0 }) })>
      <p style(errorText)>{session.errorText()}</p>
      <button type="button" size="sm" variant="ghost" components-retry on:click={session.onRetry}>Retry</button>
    </row>
  </if>

  <if {session.loading()}>
    <row components-loading align="center" gap="2" style(bandStatus, { box: flow({ shrink: 0 }) })>
      <spinner size="sm"></spinner>
      <p style(fieldHint)>Loading…</p>
    </row>
  </if>

  <row components-workbench gap="0" nowrap align="stretch" style(workbench)>
    <scroll-area components-rail style(panelRail)>
      <column gap="3" style(railCol)>
        <column gap="2">
          <p style(eyebrow)>Components</p>
          <p style({ text: type({ size: md, weight: semibold, tracking: tight }) })>Components</p>
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
        <!-- Preview host: draft paint via applyScope (theme + mode). -->
        <card
          components-specimen
          preview-specimen
          padding="none" style(specimenFrame)>
          <column gap="0" style({  box: flow({ grow: 1 }).size({ h: full, min-h: 0 })  })>
            <row preview-chrome align="center" gap="2" nowrap style(specimenChrome)>
              <row gap="2" align="center" nowrap hidden="true" style(trafficDots)>
                <stack style(trafficDot, { paint: fill({ color: destructive }) })></stack>
                <stack style(trafficDot, { paint: fill({ color: warning }) })></stack>
                <stack style(trafficDot, { paint: fill({ color: success }) })></stack>
              </row>
              <p style(chromeTitle)>
                Preview · {session.nameEdit()} · {components.selectedLabel()}
              </p>
              <p style(pathMeta, { box: flow({ shrink: 0 }) })>{session.paintMode()}</p>
            </row>

            <if {components.hasVariants()}>
              <row specimen-variants align="center" gap="2" nowrap on:click={components.onVariantPick} style(specimenVariants)>
                <p style(pathMeta, { box: flow({ shrink: 0 }) })>{components.viewerVariantAttr()}:</p>
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

