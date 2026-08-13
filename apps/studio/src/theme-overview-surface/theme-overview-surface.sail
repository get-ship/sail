<!--
  Theme overview — thin shell: chrome + identity editor + specimen.
  Controller composition: session + overview domain.
-->
<scope>
  style sectionTitle = { text: type({ size: sm, weight: semibold }) }
  style fieldHint = { paint: ink({ color: muted-foreground }), text: type({ size: xs }) }
  style monoMeta = { paint: ink({ color: muted-foreground }), text: type({ family: mono, size: xs }) }
  style mutedIcon = { paint: ink({ color: muted-foreground }) }
  style panelCard = { box: space({ pad: 4 }).size({ h: full, min-w: 0 }), paint: radius(lg).border({ color: border-subtle, width: 1 }).fill({ color: surface-1 }) }
  style doorCard = { box: space({ pad: 3 }), paint: cursor(pointer), when: { hover: { paint: fill({ color: hover-bg }) } } }
  style doorTitle = { text: type({ weight: semibold }) }
  style doorHint = { paint: ink({ color: muted-foreground }), text: type({ size: xs }) }
  style heroCard = { box: overflow({ all: hidden }).size({ min-w: 0 }), paint: radius(lg).border({ color: border-subtle, width: 1 }).fill({ color: surface-1 }) }
  style heroSwatch = { box: place({ position: relative }).size({ w: full, h: 28, min-h: 28 }).overflow({ all: hidden }), paint: fill({ color: bind(swatchColor) }), when: { stateEdit: { paint: cursor(pointer) }, stateLocked: { paint: cursor(default) } } }
  style heroColorInput = { box: place({ position: absolute, top: 0, left: 0 }).size(full).space({ pad: 0 }), paint: border({ width: 0 }).cursor(pointer).filter({ opacity: 0 }), when: { disabled: { paint: cursor(not-allowed) } } }
  style heroMeta = { box: space({ pad-x: 3, pad-y: 2 }) }
  style identityCol = { box: space({ pad-x: 6, pad-t: 6, pad-b: 10 }).size({ max-w: panel-max-w-lg }) }
  style iconCell = { box: size({ w: 9, h: 9, min-w: 9, min-h: 9 }), paint: radius(md).border({ color: border-subtle, width: 1 }).fill({ color: surface-0 }) }
  style bandStatus = { box: space({ pad-x: 4, pad-y: 2 }) }
  style bandError = { box: space({ pad-x: 4, pad-y: 2 }), paint: border({ edge: b, color: border, width: 1 }) }
  style fullBtn = { box: size({ w: full, min-w: control-h-sm, min-h: control-h-sm }) }
  style specimenBtn = { box: flow({ shrink: 0 }).size({ min-w: control-h-sm, min-h: 6 }) }
  style surfaceRoot = { box: flow({ grow: 1 }).size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }), paint: fill({ color: background }) }
  style workbench = { box: flow({ grow: 1 }).size({h: full, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }), paint: fill({ color: background }) }
  style editorScroll = { box: flow({ grow: 1 }).size({h: full, min-w: 0, min-h: 0, basis: 0}), paint: fill({ color: background }) }
  style specimenDock = { box: flow({ grow: 1 }).size({h: full, min-w: panel-min-w-md, min-h: 0, max-w: panel-max-w-md, basis: 0}).overflow({ all: hidden }), paint: border({ edge: l, color: border-subtle, width: 1 }).fill({ color: background }) }
  style specimenPad = { box: flow({ grow: 1 }).size({ h: full, min-h: 0 }).space({ pad: 4 }) }
  style specimenFrame = { box: size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }), paint: elevation(lg) }
  style specimenChrome = { box: flow({ shrink: 0 }).space({ pad-x: 3 }).size({ h: 8, min-w: 0, min-h: 8 }), paint: border({ edge: b, color: border-subtle, width: 1 }).fill({ color: surface-1 }) }
  style trafficDots = { box: flow({ shrink: 0 }).size({ min-w: 12 }) }
  style trafficDot = { box: flow({ shrink: 0 }).size({ w: 2, h: 2, min-w: 2, min-h: 2 }), paint: radius(full) }
  style chromeTitle = { box: flow({ grow: 1 }).size({ min-w: 0 }), paint: ink({ color: muted-foreground }), text: type({ size: xs, wrap: truncate }) }
  style errorText = { box: flow({ grow: 1 }).size({ min-w: 0 }), paint: ink({ color: destructive }), text: type({ size: sm, wrap: truncate }) }
  style growMin = { box: flow({ grow: 1 }).size({ min-w: 0 }) }
  style pickerHero = { box: size({ w: full, h: 20, min-h: 20 }) }
  type HeroColorCardProps = {
    token: string
    value: string
    label: string
    canEdit: boolean
    onChange: any
  }
</scope>

<partial hero-color-card props: HeroColorCardProps>
  <column hero-card token={token} gap="0" style(heroCard, { box: flow({ grow: 1 }).size({ min-w: 0 }) })>
    <stack use:swatch={value} state={canEdit ? 'edit' : 'locked'} style(heroSwatch)>
      <sail-color-picker
        token={token}
        value={value}
        disabled={!canEdit}
        variant="swatch"
        showHex={false}
        label={label}
        on-change={onChange}
        style(pickerHero)
      ></sail-color-picker>
    </stack>
    <row align="center" justify="between" style(heroMeta)>
      <p style(doorTitle)>{label}</p>
      <p style(monoMeta)>{value}</p>
    </row>
  </column>
</partial>

<controller>
import { createThemeSession } from '../compositions/theme-session/theme-session'
import { watchPreviewHost } from '../compositions/theme-session/bind-preview-host'
import { createOverviewEdit } from '../compositions/theme-overview-surface/theme-overview-edit'
export default controller(() => {
  const session = createThemeSession({
    facet: 'overview',
    editGuardMessage: 'Add this theme to the project to edit identity.',
  })
  const overview = createOverviewEdit(session)
  // Style bind() only sees setup-local signals (not composition re-exports).
  const primaryHex = signal(overview.primaryHex())
  const secondaryHex = signal(overview.secondaryHex())
  const accentHex = signal(overview.accentHex())
  const densityPad = signal(overview.densityPad())
  const specimenGap = signal(overview.specimenGap())
  const specimenStackGap = signal(overview.specimenStackGap())
  mount(() =>
    effect(() => {
      primaryHex.set(overview.primaryHex())
      secondaryHex.set(overview.secondaryHex())
      accentHex.set(overview.accentHex())
      densityPad.set(overview.densityPad())
      specimenGap.set(overview.specimenGap())
      specimenStackGap.set(overview.specimenStackGap())
    }),
  )
  mount((host) => watchPreviewHost(host, (el) => session.attachPreviewHost(el)))
  return { session, overview, primaryHex, secondaryHex, accentHex, densityPad, specimenGap, specimenStackGap }
})
</controller>

<column surface="themes" theme-overview-surface overview-chrome gap="0" style(surfaceRoot)>
  <theme-facet-chrome session={session}></theme-facet-chrome>

  <if {session.hasError()}>
    <row overview-error align="center" gap="2" style(bandError, { box: flow({ shrink: 0 }) })>
      <p style(errorText)>{session.errorText()}</p>
      <button type="button" size="sm" variant="ghost" overview-retry on:click={session.onRetry}>
        Retry
      </button>
    </row>
  </if>

  <if {session.loading()}>
    <row overview-loading align="center" gap="2" style(bandStatus, { box: flow({ shrink: 0 }) })>
      <spinner size="sm"></spinner>
      <p style(fieldHint)>Loading…</p>
    </row>
  </if>

  <row overview-workbench editorial-split gap="0" nowrap align="stretch" style(workbench)>
    <scroll-area identity-column style(editorScroll)>
      <column gap="8" style(identityCol)>
        <column color-hero gap="3">
          <p style(sectionTitle)>Color</p>
          <row hero-colors gap="3" align="stretch">
            <hero-color-card
              token="primary"
              value={primaryHex()}
              label="Primary"
              canEdit={session.canEdit()}
              onChange={overview.onHeroColor}
            ></hero-color-card>
            <hero-color-card
              token="secondary"
              value={secondaryHex()}
              label="Secondary"
              canEdit={session.canEdit()}
              onChange={overview.onHeroColor}
            ></hero-color-card>
            <hero-color-card
              token="accent"
              value={accentHex()}
              label="Accent"
              canEdit={session.canEdit()}
              onChange={overview.onHeroColor}
            ></hero-color-card>
          </row>
          <if {session.catalogHint()}>
            <p catalog-hint style(fieldHint)>Add to project to edit.</p>
          </if>
        </column>

        <grid cols="3" gap="3" editorial-band>
          <column panel="type" gap="3" style(panelCard)>
            <p style(sectionTitle)>Type</p>
            <column gap="3">
              <column gap="1">
                <p style(fieldHint)>Display</p>
                <select type-token="display" size="sm" value={overview.typeDisplay()} disabled={!session.canEdit()} on:change={overview.onTypeSelect}>
                  <for {overview.displayFontOpts() as o} key={o.key}>
                    <option value={o.value}>{o.label}</option>
                  </for>
                </select>
              </column>
              <column gap="1">
                <p style(fieldHint)>Body</p>
                <select type-token="body" size="sm" value={overview.typeBody()} disabled={!session.canEdit()} on:change={overview.onTypeSelect}>
                  <for {overview.bodyFontOpts() as o} key={o.key}>
                    <option value={o.value}>{o.label}</option>
                  </for>
                </select>
              </column>
              <column gap="1">
                <p style(fieldHint)>Mono</p>
                <select type-token="mono" size="sm" value={overview.typeMono()} disabled={!session.canEdit()} on:change={overview.onTypeSelect}>
                  <for {overview.monoFontOpts() as o} key={o.key}>
                    <option value={o.value}>{o.label}</option>
                  </for>
                </select>
              </column>
            </column>
          </column>

          <column panel="icons" gap="3" style(panelCard)>
            <p style(sectionTitle)>Icons</p>
            <column gap="1">
              <p style(fieldHint)>Pack</p>
              <select icon-pack size="sm" value={overview.iconPack()} disabled={!session.canEdit()} on:change={overview.onIconPack}>
                <option value="lucide">Lucide</option>
                <option value="phosphor">Phosphor</option>
              </select>
            </column>
            <if {overview.hasIconGlyphs()}>
              <grid cols="4" gap="2" icon-sample style({ box: space({ pad-t: 1 }) })>
                <for {overview.iconGlyphs() as g} key={g.key}>
                  <center title={g.name} style(iconCell, { box: flow({ shrink: 0 }) })>
                    <icon name={g.name} size="sm" label={g.name}></icon>
                  </center>
                </for>
              </grid>
            </if>
          </column>

          <column panel="density" gap="3" style(panelCard)>
            <p style(sectionTitle)>Density</p>
            <column density-seg gap="2">
              <for {overview.densityOptions() as d} key={d.key}>
                <button
                  type="button"
                  size="sm"
                  density={d.id}
                  variant={d.on ? 'solid' : 'soft'}
                  tone={d.on ? 'primary' : 'muted'}
                  pressed={d.on ? 'true' : 'false'}
                  disabled={!session.canEdit()}
                  on:click={() => overview.onDensityClick(d.id)}
                  style(fullBtn)>
                  {d.label}
                </button>
              </for>
            </column>
          </column>
        </grid>

        <column doors-row gap="3">
          <p style(sectionTitle)>More detail</p>
          <grid cols="2" gap="3" facet-doors on:click={session.onFacet}>
            <for {overview.facetDoors() as door} key={door.key}>
              <card facet={door.facet} style(doorCard)>
                <row gap="2" align="center">
                  <icon name={door.icon} size="sm" style(mutedIcon, { box: flow({ shrink: 0 }) })></icon>
                  <column gap="0" style({ box: flow({ grow: 1 }).size({ min-w: 0 }) })>
                    <p style(doorTitle)>{door.label}</p>
                    <p style(doorHint)>{door.hint}</p>
                  </column>
                </row>
              </card>
            </for>
          </grid>
        </column>
      </column>
    </scroll-area>

    <column specimen-dock gap="0" style(specimenDock)>
      <column gap="0" style(specimenPad)>
        <card
          overview-specimen
          preview-specimen
          density={overview.specimenDensity()}
          padding="none" style(specimenFrame)>
          <column gap="0" style({  box: flow({ grow: 1 }).size({ h: full, min-h: 0 })  })>
            <row preview-chrome align="center" gap="2" nowrap style(specimenChrome)>
              <row gap="2" align="center" nowrap hidden="true" style(trafficDots)>
                <stack style(trafficDot, { paint: fill({ color: destructive }) })></stack>
                <stack style(trafficDot, { paint: fill({ color: warning }) })></stack>
                <stack style(trafficDot, { paint: fill({ color: success }) })></stack>
              </row>
              <p style(chromeTitle)>Preview · {session.nameEdit()}</p>
              <p style(monoMeta, { box: flow({ shrink: 0 }) })>{session.paintMode()}</p>
            </row>
            <scroll-area style({  box: flow({ grow: 1 }).size({h: full, min-h: 0, basis: 0}), paint: fill({ color: background })  })>
              <column
                specimen-body gap={specimenGap()} style({ box: size({ w: full, min-w: 0 }).space({ pad: bind(densityPad) }) })>
                <row gap="2" align="center" nowrap specimen-buttons>
                  <button type="button" size={overview.controlSize()} variant="solid" tone="primary" style(specimenBtn)>Primary</button>
                  <button type="button" size={overview.controlSize()} variant="solid" tone="accent" style(specimenBtn)>Accent</button>
                  <button type="button" size={overview.controlSize()} variant="outline" tone="primary" style(specimenBtn)>Outline</button>
                </row>
                <row gap="2" align="center" nowrap specimen-badges>
                  <badge tone="primary" size={overview.badgeSize()} variant="solid" pill>Primary</badge>
                  <badge tone="success" size={overview.badgeSize()} variant="solid" pill>Success</badge>
                  <badge tone="warning" size={overview.badgeSize()} variant="solid" pill>Warning</badge>
                  <badge tone="destructive" size={overview.badgeSize()} variant="solid" pill>Danger</badge>
                </row>
                <column type-samples gap={specimenStackGap()}>
                  <p style({ text: type({ family: display, size: xl, weight: semibold, tracking: tight }) })>
                    {overview.typeDisplay()}
                  </p>
                  <p style({ text: type({ family: body, size: sm, leading: relaxed }) })>
                    {overview.typeBody()} — product body copy.
                  </p>
                  <p style({ text: type({ family: mono, size: xs }) })>
                    {overview.typeMono()} — const ok = true
                  </p>
                </column>
                <if {overview.hasIconGlyphs()}>
                  <row specimen-icons gap="2" align="center" nowrap>
                    <for {overview.iconGlyphs() as g} key={g.key}>
                      <icon name={g.name} size={overview.iconSize()} label={g.name} style({ box: flow({ shrink: 0 }) })></icon>
                    </for>
                  </row>
                </if>
                <column gap={specimenStackGap()}>
                  <p style({ paint: ink({ color: muted-foreground }), text: type({ size: xs }) })>Email</p>
                  <input type="email" size={overview.controlSize()} placeholder="you@company.com" label="Sample email" />
                </column>
                <card padding={overview.cardPadding()}>
                  <column gap={specimenStackGap()}>
                    <p style({ text: type({ family: heading, size: md, weight: semibold }) })>{session.nameEdit()}</p>
                    <p style({ paint: ink({ color: muted-foreground }), text: type({ family: body, size: sm, leading: relaxed }) })>
                      Nested card in this theme.
                    </p>
                    <row gap="2" nowrap>
                      <button type="button" size={overview.controlSize()} variant="solid" tone="primary" style(specimenBtn)>Continue</button>
                      <button type="button" size={overview.controlSize()} variant="ghost" tone="muted" style(specimenBtn)>Cancel</button>
                    </row>
                  </column>
                </card>
              </column>
            </scroll-area>
          </column>
        </card>
      </column>
    </column>
  </row>

  <toast-region position="bottom-end" overview-toast-region>
    <for {session.toastItems() as t} key={t.key}>
      <toast tone={t.tone} overview-toast on-dismiss={session.onToastDismiss}>
        {t.message}
      </toast>
    </for>
  </toast-region>
</column>
