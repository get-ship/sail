<!--
  Theme overview — thin shell: chrome + identity editor + specimen.
  Controller composition: session + overview domain.
-->
<scope>
  const sectionTitle = style({ text: fontSize(sm).fontWeight(semibold) })
  const fieldHint = style({ paint: fg(muted-foreground), text: fontSize(xs) })
  const monoMeta = style({ paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) })
  const mutedIcon = style({ paint: fg(muted-foreground) })
  const panelCard = style({ box: pad(4).minW(0).h(full), paint: radius(lg).borderWidth(1).border(border-subtle).bg(surface-1) })
  const doorCard = style({ box: pad(3), paint: cursor(pointer) }).when(hover, { paint: bg(hover-bg) })
  const doorTitle = style({ text: fontWeight(semibold) })
  const doorHint = style({ paint: fg(muted-foreground), text: fontSize(xs) })
  const heroCard = style({ box: overflow(hidden).minW(0), paint: radius(lg).borderWidth(1).border(border-subtle).bg(surface-1) })
  const heroSwatch = style({ box: position(relative).h(28).minH(28).w(full).overflow(hidden), paint: bg(bind(swatchColor)) }).when(stateEdit, { paint: cursor(pointer) }).when(stateLocked, { paint: cursor(default) })
  const heroColorInput = style({ box: position(absolute).top(0).left(0).w(full).h(full).pad(0), paint: borderWidth(0).cursor(pointer).opacity(0) }).when(disabled, { paint: cursor(not-allowed) })
  const heroMeta = style({ box: padX(3).padY(2) })
  const identityCol = style({ box: padX(6).padT(6).padB(10).maxW(panel-max-w-lg) })
  const iconCell = style({ box: w(9).h(9).minW(9).minH(9), paint: radius(md).borderWidth(1).border(border-subtle).bg(surface-0) })
  const bandStatus = style({ box: padX(4).padY(2) })
  const bandError = style({ box: padX(4).padY(2), paint: borderBWidth(1).borderB(border) })
  const fullBtn = style({ box: w(full) })
</scope>

<controller>
import { effect, mountAmbient, signal } from '@ship/sail-runtime'
import { createThemeSession } from '../theme-session/theme-session'
import { watchPreviewHost } from '../theme-session/bind-preview-host'
import { createOverviewEdit } from './theme-overview-edit'
export default controller(() => {
  const session = createThemeSession({
    facet: 'overview',
    editGuardMessage: 'Add this theme to the project to edit identity.',
  })
  const overview = createOverviewEdit(session)
  // sx bind() only sees setup-local signals (not factory re-exports).
  const primaryHex = signal(overview.primaryHex())
  const secondaryHex = signal(overview.secondaryHex())
  const accentHex = signal(overview.accentHex())
  const densityPad = signal(overview.densityPad())
  const specimenGap = signal(overview.specimenGap())
  const specimenStackGap = signal(overview.specimenStackGap())
  mountAmbient(() =>
    effect(() => {
      primaryHex.set(overview.primaryHex())
      secondaryHex.set(overview.secondaryHex())
      accentHex.set(overview.accentHex())
      densityPad.set(overview.densityPad())
      specimenGap.set(overview.specimenGap())
      specimenStackGap.set(overview.specimenStackGap())
    }),
  )
  // Draft paint host — applyScope when T1 applier is set.
  mount((host) => watchPreviewHost(host, (el) => session.attachPreviewHost(el)))
  return { session, overview, primaryHex, secondaryHex, accentHex, densityPad, specimenGap, specimenStackGap }
})
</controller>

<column
  surface="themes"
  theme-overview-surface
  overview-chrome
  gap="0"
  grow

>
  style({ box: minH(0).h(full).minW(0).overflow(hidden), paint: bg(background) })

  <theme-facet-chrome session={session}></theme-facet-chrome>

  <if {session.hasError()}>
    <row overview-error align="center" gap="2" shrink="0">
      style(bandError)

      <p grow min-w="0">
        style({ paint: fg(destructive), text: fontSize(sm).truncate() })
{session.errorText()}</p>
      <button type="button" size="sm" variant="ghost" overview-retry on:click={session.onRetry}>
        Retry
      </button>
    </row>
  </if>

  <if {session.loading()}>
    <row overview-loading align="center" gap="2" shrink="0">
      style(bandStatus)

      <spinner size="sm"></spinner>
      <p>
        style(fieldHint)
Loading…</p>
    </row>
  </if>

  <row
    overview-workbench
    editorial-split
    grow
    gap="0"
    nowrap
    align="stretch"

  >
    style({ box: minH(0).basis(0).h(full).minW(0).overflow(hidden), paint: bg(background) })

    <scroll-area
      identity-column
      grow
      min-w="0"

    >
      style({ box: minH(0).basis(0).h(full).minW(0), paint: bg(background) })

      <column gap="8">
        style(identityCol)

        <column color-hero gap="3">
          <p>
            style(sectionTitle)
Color</p>
          <row hero-colors gap="3" align="stretch">
            <column hero-card token="primary" grow gap="0" min-w="0">
              style(heroCard)

              <stack use:swatch={primaryHex()} state={session.canEdit() ? 'edit' : 'locked'}>
                style(heroSwatch)

                <sail-color-picker token="primary" value={primaryHex()} disabled={!session.canEdit()} variant="swatch" showHex={false} label="Primary" on:sail-change={overview.onHeroColor}>
                  style({ box: w(full).h(20).minH(20) })
</sail-color-picker>
              </stack>
              <row align="center" justify="between">
                style(heroMeta)

                <p>
                  style(doorTitle)
Primary</p>
                <p>
                  style(monoMeta)
{primaryHex()}</p>
              </row>
            </column>
            <column hero-card token="secondary" grow gap="0" min-w="0">
              style(heroCard)

              <stack use:swatch={secondaryHex()} state={session.canEdit() ? 'edit' : 'locked'}>
                style(heroSwatch)

                <sail-color-picker token="secondary" value={secondaryHex()} disabled={!session.canEdit()} variant="swatch" showHex={false} label="Secondary" on:sail-change={overview.onHeroColor}>
                  style({ box: w(full).h(20).minH(20) })
</sail-color-picker>
              </stack>
              <row align="center" justify="between">
                style(heroMeta)

                <p>
                  style(doorTitle)
Secondary</p>
                <p>
                  style(monoMeta)
{secondaryHex()}</p>
              </row>
            </column>
            <column hero-card token="accent" grow gap="0" min-w="0">
              style(heroCard)

              <stack use:swatch={accentHex()} state={session.canEdit() ? 'edit' : 'locked'}>
                style(heroSwatch)

                <sail-color-picker token="accent" value={accentHex()} disabled={!session.canEdit()} variant="swatch" showHex={false} label="Accent" on:sail-change={overview.onHeroColor}>
                  style({ box: w(full).h(20).minH(20) })
</sail-color-picker>
              </stack>
              <row align="center" justify="between">
                style(heroMeta)

                <p>
                  style(doorTitle)
Accent</p>
                <p>
                  style(monoMeta)
{accentHex()}</p>
              </row>
            </column>
          </row>
          <if {session.catalogHint()}>
            <p catalog-hint>
              style(fieldHint)
Add to project to edit.</p>
          </if>
        </column>

        <grid cols="3" gap="3" editorial-band>
          <column panel="type" gap="3">
            style(panelCard)

            <p>
              style(sectionTitle)
Type</p>
            <column gap="3">
              <column gap="1">
                <p>
                  style(fieldHint)
Display</p>
                <select type-token="display" size="sm" value={overview.typeDisplay()} disabled={!session.canEdit()} on:change={overview.onTypeSelect}>
                  <for {overview.displayFontOpts()} as="o" key={o.key}>
                    <option value={o.value}>{o.label}</option>
                  </for>
                </select>
              </column>
              <column gap="1">
                <p>
                  style(fieldHint)
Body</p>
                <select type-token="body" size="sm" value={overview.typeBody()} disabled={!session.canEdit()} on:change={overview.onTypeSelect}>
                  <for {overview.bodyFontOpts()} as="o" key={o.key}>
                    <option value={o.value}>{o.label}</option>
                  </for>
                </select>
              </column>
              <column gap="1">
                <p>
                  style(fieldHint)
Mono</p>
                <select type-token="mono" size="sm" value={overview.typeMono()} disabled={!session.canEdit()} on:change={overview.onTypeSelect}>
                  <for {overview.monoFontOpts()} as="o" key={o.key}>
                    <option value={o.value}>{o.label}</option>
                  </for>
                </select>
              </column>
            </column>
          </column>

          <column panel="icons" gap="3">
            style(panelCard)

            <p>
              style(sectionTitle)
Icons</p>
            <column gap="1">
              <p>
                style(fieldHint)
Pack</p>
              <select icon-pack size="sm" value={overview.iconPack()} disabled={!session.canEdit()} on:change={overview.onIconPack}>
                <option value="lucide">Lucide</option>
                <option value="phosphor">Phosphor</option>
              </select>
            </column>
            <if {overview.hasIconGlyphs()}>
              <grid cols="4" gap="2" icon-sample>
                style({ box: padT(1) })

                <for {overview.iconGlyphs()} as="g" key={g.key}>
                  <center shrink="0" title={g.name}>
                    style(iconCell)

                    <icon name={g.name} size="sm" label={g.name}></icon>
                  </center>
                </for>
              </grid>
            </if>
          </column>

          <column panel="density" gap="3">
            style(panelCard)

            <p>
              style(sectionTitle)
Density</p>
            <column density-seg gap="2" on:click={overview.onDensityClick}>
              <for {overview.densityOptions()} as="d" key={d.key}>
                <button
                  type="button"
                  size="sm"
                  density={d.id}
                  variant={d.on ? 'solid' : 'soft'}
                  tone={d.on ? 'primary' : 'muted'}
                  pressed={d.on ? 'true' : 'false'}
                  disabled={!session.canEdit()}

                >
                  style(fullBtn)

                  {d.label}
                </button>
              </for>
            </column>
          </column>
        </grid>

        <column doors-row gap="3">
          <p>
            style(sectionTitle)
More detail</p>
          <grid cols="2" gap="3" facet-doors on:click={session.onFacet}>
            <for {overview.facetDoors()} as="door" key={door.key}>
              <card facet={door.facet}>
                style(doorCard)

                <row gap="2" align="center">
                  <icon name={door.icon} size="sm" shrink="0">
                    style(mutedIcon)
</icon>
                  <column gap="0" grow min-w="0">
                    <p>
                      style(doorTitle)
{door.label}</p>
                    <p>
                      style(doorHint)
{door.hint}</p>
                  </column>
                </row>
              </card>
            </for>
          </grid>
        </column>
      </column>
    </scroll-area>

    <column
      specimen-dock
      gap="0"
      grow
      min-w="0"

    >
      style({ box: minW(panel-min-w-md).maxW(panel-max-w-md).basis(0).minH(0).h(full).overflow(hidden), paint: borderLWidth(1).borderL(border-subtle).bg(background) })

      <column grow gap="0">
        style({ box: minH(0).h(full).pad(4) })

        <card
          overview-specimen
          preview-specimen
          mode={session.paintMode()}
          density={overview.specimenDensity()}
          padding="none"

        >
          style({ box: minW(0).minH(0).h(full).overflow(hidden), paint: shadow(lg) })

          <column gap="0" grow>
            style({ box: minH(0).h(full) })

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
{session.nameEdit()}</p>
              <p shrink="0">
                style(pathMeta)
{session.paintMode()}</p>
            </row>
            <scroll-area grow>
              style({ box: minH(0).basis(0).h(full), paint: bg(background) })

              <!--
                Plain host for density gap bind — layout column always ships sail-g-*
                which beats sx.gap(bind). stack rejects style({ box: display(owns flex itself) }).
              -->
              <div
                specimen-body

              >
                style({ box: display(flex).flexDir(column).w(full).minW(0).pad(bind(densityPad)).gap(bind(specimenGap)) })

                <row gap="2" align="center" nowrap specimen-buttons>
                  <button type="button" size={overview.controlSize()} variant="solid" tone="primary">Primary</button>
                  <button type="button" size={overview.controlSize()} variant="solid" tone="accent">Accent</button>
                  <button type="button" size={overview.controlSize()} variant="outline" tone="primary">Outline</button>
                </row>
                <row gap="2" align="center" nowrap specimen-badges>
                  <badge tone="primary" size={overview.badgeSize()} variant="solid" pill>Primary</badge>
                  <badge tone="success" size={overview.badgeSize()} variant="solid" pill>Success</badge>
                  <badge tone="warning" size={overview.badgeSize()} variant="solid" pill>Warning</badge>
                  <badge tone="destructive" size={overview.badgeSize()} variant="solid" pill>Danger</badge>
                </row>
                <div type-samples>
                  style({ box: display(flex).flexDir(column).gap(bind(specimenStackGap)) })

                  <p>
                    style({ text: fontFamily(display).fontSize(xl).fontWeight(semibold).tracking(tight) })

                    {overview.typeDisplay()}
                  </p>
                  <p>
                    style({ text: fontFamily(body).fontSize(sm).leading(relaxed) })

                    {overview.typeBody()} — product body copy.
                  </p>
                  <p>
                    style({ text: fontFamily(mono).fontSize(xs) })

                    {overview.typeMono()} — const ok = true
                  </p>
                </div>
                <if {overview.hasIconGlyphs()}>
                  <row specimen-icons gap="2" align="center" nowrap>
                    <for {overview.iconGlyphs()} as="g" key={g.key}>
                      <icon name={g.name} size={overview.iconSize()} label={g.name} shrink="0"></icon>
                    </for>
                  </row>
                </if>
                <div>
                  style({ box: display(flex).flexDir(column).gap(bind(specimenStackGap)) })

                  <p>
                    style({ paint: fg(muted-foreground), text: fontSize(xs) })
Email</p>
                  <input type="email" size={overview.controlSize()} placeholder="you@company.com" label="Sample email" />
                </div>
                <card padding={overview.cardPadding()}>
                  <div>
                    style({ box: display(flex).flexDir(column).gap(bind(specimenStackGap)) })

                    <p>
                      style({ text: fontSize(md).fontWeight(semibold).fontFamily(heading) })
{session.nameEdit()}</p>
                    <p>
                      style({ paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed).fontFamily(body) })

                      Nested card in this theme.
                    </p>
                    <row gap="2" nowrap>
                      <button type="button" size={overview.controlSize()} variant="solid" tone="primary">Continue</button>
                      <button type="button" size={overview.controlSize()} variant="ghost" tone="muted">Cancel</button>
                    </row>
                  </div>
                </card>
              </div>
            </scroll-area>
          </column>
        </card>
      </column>
    </column>
  </row>

  <toast-region position="bottom-end" overview-toast-region>
    <for {session.toastItems()} as="t" key={t.key}>
      <toast tone={t.tone} overview-toast on:sail-toast-dismiss={session.onToastDismiss}>
        {t.message}
      </toast>
    </for>
  </toast-region>
</column>
