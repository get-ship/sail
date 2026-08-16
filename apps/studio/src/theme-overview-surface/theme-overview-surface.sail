<!--
  Theme overview — thin shell: chrome + identity editor + specimen.
  Controller composition: session + overview domain.
-->
<scope>
  style sectionTitle = { type({ size: sm, weight: semibold }) }
  style fieldHint = { fg(muted-foreground).type({ size: xs }) }
  style monoMeta = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style mutedIcon = { fg(muted-foreground) }
  style panelCard = { pad(4).size({ h: full, min-w: 0 }).radius(lg).border({ color: border-subtle, width: 1 }).bg(surface-1) }
  style doorCard = { pad(3).cursor(pointer), when: { hover: bg(hover-bg) } }
  style doorTitle = { type({ weight: semibold }) }
  style doorHint = { fg(muted-foreground).type({ size: xs }) }
  style heroCard = { overflow({ all: hidden }).size({ min-w: 0 }).radius(lg).border({ color: border-subtle, width: 1 }).bg(surface-1) }
  style heroSwatch = { position(relative).size({ w: full, h: 28, min-h: 28 }).overflow({ all: hidden }).bg(bind(swatchColor)), when: { stateEdit: cursor(pointer), stateLocked: cursor(default) } }
  style heroColorInput = { position({ scheme: absolute, t: 0, l: 0 }).size(full).pad(0).border({ width: 0 }).cursor(pointer).filter({ opacity: 0 }), when: { disabled: cursor(not-allowed) } }
  style heroMeta = { pad({ x: 3, y: 2 }) }
  style identityCol = { pad({ x: 6, t: 6, b: 10 }).size({ max-w: panel-max-w-lg }) }
  style iconCell = { size({ w: 9, h: 9, min-w: 9, min-h: 9 }).radius(md).border({ color: border-subtle, width: 1 }).bg(surface-0) }
  style bandStatus = { pad({ x: 4, y: 2 }) }
  style bandError = { pad({ x: 4, y: 2 }).border({ edge: b, color: border, width: 1 }) }
  style fullBtn = { size({ w: full, min-w: control-h-sm, min-h: control-h-sm }) }
  style specimenBtn = { shrink(0).size({ min-w: control-h-sm, min-h: 6 }) }
  style surfaceRoot = { grow(1).size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }).bg(background) }
  style workbench = { grow(1).size({h: full, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }).bg(background) }
  style editorScroll = { grow(1).size({h: full, min-w: 0, min-h: 0, basis: 0}).bg(background) }
  style specimenDock = { grow(1).size({h: full, min-w: panel-min-w-md, min-h: 0, max-w: panel-max-w-md, basis: 0}).overflow({ all: hidden }).border({ edge: l, color: border-subtle, width: 1 }).bg(background) }
  style specimenPad = { grow(1).size({ h: full, min-h: 0 }).pad(4) }
  style specimenFrame = { size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }).elevation(lg) }
  style specimenChrome = { shrink(0).pad({ x: 3 }).size({ h: 8, min-w: 0, min-h: 8 }).border({ edge: b, color: border-subtle, width: 1 }).bg(surface-1) }
  style trafficDots = { shrink(0).size({ min-w: 12 }) }
  style trafficDot = { shrink(0).size({ w: 2, h: 2, min-w: 2, min-h: 2 }).radius(full) }
  style chromeTitle = { grow(1).size({ min-w: 0 }).fg(muted-foreground).type({ size: xs, wrap: truncate }) }
  style errorText = { grow(1).size({ min-w: 0 }).fg(destructive).type({ size: sm, wrap: truncate }) }
  style growMin = { grow(1).size({ min-w: 0 }) }
  style pickerHero = { size({ w: full, h: 20, min-h: 20 }) }
  style chipLabel = { type({ wrap: truncate }) }
  type HeroColorCardProps = {
    token: string
    value: string
    label: string
    canEdit: boolean
    onChange: any
  }
</scope>

<partial hero-color-card props: HeroColorCardProps>
  <column hero-card token={token} gap="0" style(heroCard, { grow(1).size({ min-w: 0 }) })>
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
    <row overview-error align="center" gap="2" style(bandError, { shrink(0) })>
      <p style(errorText)>{session.errorText()}</p>
      <button type="button" size="sm" variant="ghost" overview-retry on:click={session.onRetry}>
        Retry
      </button>
    </row>
  </if>

  <if {session.loading()}>
    <row overview-loading align="center" gap="2" style(bandStatus, { shrink(0) })>
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
              <grid cols="4" gap="2" icon-sample style({ pad({ t: 1 }) })>
                <for {overview.iconGlyphs() as g} key={g.key}>
                  <center title={g.name} style(iconCell, { shrink(0) })>
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
                  <icon name={door.icon} size="sm" style(mutedIcon, { shrink(0) })></icon>
                  <column gap="0" style({ grow(1).size({ min-w: 0 }) })>
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
          <column gap="0" style({ grow(1).size({ h: full, min-h: 0 }) })>
            <row preview-chrome align="center" gap="2" nowrap style(specimenChrome)>
              <row gap="2" align="center" nowrap hidden="true" style(trafficDots)>
                <stack style(trafficDot, { bg(destructive) })></stack>
                <stack style(trafficDot, { bg(warning) })></stack>
                <stack style(trafficDot, { bg(success) })></stack>
              </row>
              <p style(chromeTitle)>Preview · {session.nameEdit()}</p>
              <p style(monoMeta, { shrink(0) })>{session.paintMode()}</p>
            </row>
            <scroll-area style({ grow(1).size({h: full, min-h: 0, basis: 0}).bg(background) })>
              <column
                specimen-body gap={specimenGap()} style({ size({ w: full, min-w: 0 }).pad(bind(densityPad)) })>
                <row gap="2" align="center" nowrap specimen-buttons>
                  <button type="button" size={overview.controlSize()} variant="solid" tone="primary" style(specimenBtn)>Primary</button>
                  <button type="button" size={overview.controlSize()} variant="solid" tone="accent" style(specimenBtn)>Accent</button>
                  <button type="button" size={overview.controlSize()} variant="outline" tone="primary" style(specimenBtn)>Outline</button>
                </row>
                <row gap="2" align="center" nowrap specimen-badges>
                  <badge tone="primary" size={overview.badgeSize()} variant="solid" pill style(chipLabel)>Primary</badge>
                  <badge tone="success" size={overview.badgeSize()} variant="solid" pill style(chipLabel)>Success</badge>
                  <badge tone="warning" size={overview.badgeSize()} variant="solid" pill style(chipLabel)>Warning</badge>
                  <badge tone="destructive" size={overview.badgeSize()} variant="solid" pill style(chipLabel)>Danger</badge>
                </row>
                <column type-samples gap={specimenStackGap()}>
                  <p style({ type({ family: display, size: xl, weight: semibold, tracking: tight }) })>
                    {overview.typeDisplay()}
                  </p>
                  <p style({ type({ family: body, size: sm, leading: relaxed }) })>
                    {overview.typeBody()} — product body copy.
                  </p>
                  <p style({ type({ family: mono, size: xs }) })>
                    {overview.typeMono()} — const ok = true
                  </p>
                </column>
                <if {overview.hasIconGlyphs()}>
                  <row specimen-icons gap="2" align="center" nowrap>
                    <for {overview.iconGlyphs() as g} key={g.key}>
                      <icon name={g.name} size={overview.iconSize()} label={g.name} style({ shrink(0) })></icon>
                    </for>
                  </row>
                </if>
                <column gap={specimenStackGap()}>
                  <p style({ fg(muted-foreground).type({ size: xs }) })>Email</p>
                  <input type="email" size={overview.controlSize()} placeholder="you@company.com" label="Sample email" />
                </column>
                <card padding={overview.cardPadding()}>
                  <column gap={specimenStackGap()}>
                    <p style({ type({ family: heading, size: md, weight: semibold }) })>{session.nameEdit()}</p>
                    <p style({ fg(muted-foreground).type({ family: body, size: sm, leading: relaxed }) })>
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
