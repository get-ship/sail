<!--
  Theme overview — thin shell: chrome + identity editor + specimen.
  Controller composition: session + overview domain.
-->

<scope>
  style sectionTitle = { type(body-sm) }
  style fieldHint = { fg(muted).type(body-sm) }
  style monoMeta = { fg(muted).type(mono-sm) }
  style mutedIcon = { fg(muted) }
  // no preset: min-w lock
  style panelCard = { pad(4).size({ h: 100%, min-w: 0 }).radius(lg).border(subtle).bg(surface) }
  style doorCard = { pad(3).cursor(pointer), when: { hover: bg(hover-bg) } }
  // no preset: type weight
  style doorTitle = { type(body-semibold) }
  style doorHint = { fg(muted).type(body-sm) }
  // no preset: min-w lock
  style heroCard = { overflow(hidden).size({ min-w: 0 }).radius(lg).border(subtle).bg(surface) }
  // no preset: size lock + position
  style heroSwatch = { position(relative).size({ w: 100%, h: 28, min-h: 28 }).overflow(hidden), when: { stateEdit: cursor(pointer), stateLocked: cursor(default) } }
  // no preset: border width 0 + position + filter
  style heroColorInput = { position({ scheme: absolute, t: 0, l: 0 }).size(max).pad(0).border({ width: 0 }).cursor(pointer).filter({ opacity: 0 }), when: { disabled: cursor(not-allowed) } }
  style heroMeta = { pad(3) }
  // no preset: size lock
  style identityCol = { pad(6).size({ max-w: overlay-max-w-lg }) }
  // no preset: size lock
  style iconCell = { size({ w: 9, h: 9, min-w: 9, min-h: 9 }).radius(md).border(subtle).bg(app) }
  style bandStatus = { pad(4) }
  // no preset: edge-only border
  style bandError = { pad(4).border(default-b) }
  style fullBtn = { size(control-sm) }
  // no preset: size lock
  style specimenBtn = { size({ min-w: control-h-sm, min-h: 6 }) }
  style surfaceRoot = { size(grow).overflow(hidden).bg(app) }
  style workbench = { size(grow).overflow(hidden).bg(app) }
  style editorScroll = { size(grow).bg(app) }
  // no preset: edge-only border
  style specimenDock = { size(grow).overflow(hidden).border(subtle-l).bg(app) }
  style specimenPad = { size(grow).pad(4) }
  style specimenFrame = { size(max).overflow(hidden).elevation(lg) }
  // no preset: min-w lock + edge-only border
  style specimenChrome = { pad(3).size({ h: 8, min-w: 0, min-h: 8 }).border(subtle-b).bg(surface) }
  // no preset: size lock
  style trafficDots = { size({ min-w: 12 }) }
  // no preset: size lock
  style trafficDot = { size({ w: 2, h: 2, min-w: 2, min-h: 2 }).radius(full) }
  style chromeTitle = { size(grow).fg(muted).type(body-sm) }
  style errorText = { size(grow).fg(danger).type(body-sm) }
  style growMin = { size(grow) }
  // no preset: size lock
  style pickerHero = { size({ w: 100%, h: 20, min-h: 20 }) }
  // no preset: type wrap
  style chipLabel = { type({ wrap: truncate }) }
  type HeroColorCardProps = {
    token: string
    value: string
    label: string
    canEdit: boolean
    onChange: any
  }
  style shrink0 = { size(hug) }
  style scrollCanvas = { size(grow).bg(app) }
</scope>

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

<partial hero-color-card props: HeroColorCardProps>
  <column hero-card token={token} gap="0" style(heroCard, growMin)>
    <stack state={canEdit ? 'edit' : 'locked'} style(heroSwatch)>
      <svg aria-hidden="true" width="100%" height="100%" preserveAspectRatio="none">
        <rect width="100%" height="100%" fill={value} />
      </svg>
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

<column surface="themes" theme-overview-surface overview-chrome gap="0" style(surfaceRoot)>
  <theme-facet-chrome session={session}></theme-facet-chrome>

  <if {session.hasError()}>
    <row overview-error align="center" gap="2" style(bandError, shrink0)>
      <p style(errorText)>{session.errorText()}</p>
      <button type="button" size="sm" variant="ghost" overview-retry on:click={session.onRetry}>
        Retry
      </button>
    </row>
  </if>

  <if {session.loading()}>
    <row overview-loading align="center" gap="2" style(bandStatus, shrink0)>
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
              <grid cols="4" gap="2" icon-sample style({ pad(1) })>
                <for {overview.iconGlyphs() as g} key={g.key}>
                  <center title={g.name} style(iconCell, shrink0)>
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
                  <icon name={door.icon} size="sm" style(mutedIcon, shrink0)></icon>
                  <column gap="0" style(growMin)>
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
          <column gap="0" style(growMin)>
            <row preview-chrome align="center" gap="2" nowrap style(specimenChrome)>
              <row gap="2" align="center" nowrap hidden="true" style(trafficDots)>
                <stack style(trafficDot, { bg(danger) })></stack>
                <stack style(trafficDot, { bg(warning) })></stack>
                <stack style(trafficDot, { bg(success) })></stack>
              </row>
              <p style(chromeTitle)>Preview · {session.nameEdit()}</p>
              <p style(monoMeta, shrink0)>{session.paintMode()}</p>
            </row>
            <scroll-area style(scrollCanvas)>
              <column
                specimen-body gap={specimenGap()} style({ size(max).pad(bind(densityPad)) })>
                <row gap="2" align="center" nowrap specimen-buttons>
                  <button type="button" size={overview.controlSize()} variant="solid" tone="primary" style(specimenBtn)>Primary</button>
                  <button type="button" size={overview.controlSize()} variant="solid" tone="accent" style(specimenBtn)>Accent</button>
                  <button type="button" size={overview.controlSize()} variant="outline" tone="primary" style(specimenBtn)>Outline</button>
                </row>
                <row gap="2" align="center" nowrap specimen-badges>
                  <badge tone="primary" size={overview.badgeSize()} variant="solid" pill style(chipLabel)>Primary</badge>
                  <badge tone="success" size={overview.badgeSize()} variant="solid" pill style(chipLabel)>Success</badge>
                  <badge tone="warning" size={overview.badgeSize()} variant="solid" pill style(chipLabel)>Warning</badge>
                  <badge tone="danger" size={overview.badgeSize()} variant="solid" pill style(chipLabel)>Danger</badge>
                </row>
                <column type-samples gap={specimenStackGap()}>
                  <p style({ type(display-sm) })>
                    {overview.typeDisplay()}
                  </p>
                  <p style({ type(body-sm) })>
                    {overview.typeBody()} — product body copy.
                  </p>
                  <p style({ type(mono-sm) })>
                    {overview.typeMono()} — const ok = true
                  </p>
                </column>
                <if {overview.hasIconGlyphs()}>
                  <row specimen-icons gap="2" align="center" nowrap>
                    <for {overview.iconGlyphs() as g} key={g.key}>
                      <icon name={g.name} size={overview.iconSize()} label={g.name} style(shrink0)></icon>
                    </for>
                  </row>
                </if>
                <column gap={specimenStackGap()}>
                  <p style({ fg(muted).type(body-sm) })>Email</p>
                  <input type="email" size={overview.controlSize()} placeholder="you@company.com" label="Sample email" />
                </column>
                <card padding={overview.cardPadding()}>
                  <column gap={specimenStackGap()}>
                    <p style({ type(heading) })>{session.nameEdit()}</p>
                    <p style({ fg(muted).type(body-sm) })>
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
