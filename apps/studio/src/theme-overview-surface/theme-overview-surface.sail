<!--
  Theme overview — thin shell: chrome + identity editor + specimen.
  Controller composition: session + overview domain.
-->
<scope>
  const sectionTitle = style.fontSize(sm).fontWeight(semibold)
  const fieldHint = style.fontSize(xs).fg(muted-foreground)
  const monoMeta = style.fontSize(xs).fontFamily(mono).fg(muted-foreground)
  const mutedIcon = style.fg(muted-foreground)
  const panelCard = style.pad(4).radius(lg).borderWidth(1).border(border-subtle).bg(surface-1).minW(0).h(full)
  const doorCard = style.pad(3).cursor(pointer).when({ hover: style.bg(hover-bg) })
  const doorTitle = style.fontWeight(semibold)
  const doorHint = style.fontSize(xs).fg(muted-foreground)
  const heroCard = style.radius(lg).borderWidth(1).border(border-subtle).overflow(hidden).minW(0).bg(surface-1)
  const heroSwatch = style.position(relative).h(28).minH(28).w(full).overflow(hidden).bg(bind(swatchColor)).when({ stateEdit: style.cursor(pointer), stateLocked: style.cursor(default) })
  const heroColorInput = style.position(absolute).top(0).left(0).w(full).h(full).borderWidth(0).pad(0).cursor(pointer).opacity(0).when({ disabled: style.cursor(not-allowed) })
  const heroMeta = style.padX(3).padY(2)
  const identityCol = style.padX(6).padT(6).padB(10).maxW(panel-max-w-lg)
  const iconCell = style.w(9).h(9).minW(9).minH(9).radius(md).borderWidth(1).border(border-subtle).bg(surface-0)
  const bandStatus = style.padX(4).padY(2)
  const bandError = style.padX(4).padY(2).borderBWidth(1).borderB(border)
  const fullBtn = style.w(full)
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
  sx={style.minH(0).h(full).minW(0).overflow(hidden).bg(background)}
>
  <theme-facet-chrome session={session}></theme-facet-chrome>

  <if {session.hasError()}>
    <row overview-error align="center" gap="2" shrink="0" sx={bandError}>
      <p grow min-w="0" sx={style.fontSize(sm).fg(destructive).truncate()}>{session.errorText()}</p>
      <button type="button" size="sm" variant="ghost" overview-retry on:click={session.onRetry}>
        Retry
      </button>
    </row>
  </if>

  <if {session.loading()}>
    <row overview-loading align="center" gap="2" shrink="0" sx={bandStatus}>
      <spinner size="sm"></spinner>
      <p sx={fieldHint}>Loading…</p>
    </row>
  </if>

  <row
    overview-workbench
    editorial-split
    grow
    gap="0"
    nowrap
    align="stretch"
    sx={style.minH(0).basis(0).h(full).minW(0).overflow(hidden).bg(background)}
  >
    <scroll-area
      identity-column
      grow
      min-w="0"
      sx={style.minH(0).basis(0).h(full).minW(0).bg(background)}
    >
      <column gap="8" sx={identityCol}>
        <column color-hero gap="3">
          <p sx={sectionTitle}>Color</p>
          <row hero-colors gap="3" align="stretch">
            <column hero-card role="primary" grow gap="0" min-w="0" sx={heroCard}>
              <stack use:swatch={primaryHex()} state={session.canEdit() ? 'edit' : 'locked'} sx={heroSwatch}>
                <sail-color-picker role="primary" value={primaryHex()} disabled={!session.canEdit()} variant="swatch" showHex={false} aria-label="Primary" on:sail-change={overview.onHeroColor} sx={style.w(full).h(20).minH(20)}></sail-color-picker>
              </stack>
              <row align="center" justify="between" sx={heroMeta}>
                <p sx={doorTitle}>Primary</p>
                <p sx={monoMeta}>{primaryHex()}</p>
              </row>
            </column>
            <column hero-card role="secondary" grow gap="0" min-w="0" sx={heroCard}>
              <stack use:swatch={secondaryHex()} state={session.canEdit() ? 'edit' : 'locked'} sx={heroSwatch}>
                <sail-color-picker role="secondary" value={secondaryHex()} disabled={!session.canEdit()} variant="swatch" showHex={false} aria-label="Secondary" on:sail-change={overview.onHeroColor} sx={style.w(full).h(20).minH(20)}></sail-color-picker>
              </stack>
              <row align="center" justify="between" sx={heroMeta}>
                <p sx={doorTitle}>Secondary</p>
                <p sx={monoMeta}>{secondaryHex()}</p>
              </row>
            </column>
            <column hero-card role="accent" grow gap="0" min-w="0" sx={heroCard}>
              <stack use:swatch={accentHex()} state={session.canEdit() ? 'edit' : 'locked'} sx={heroSwatch}>
                <sail-color-picker role="accent" value={accentHex()} disabled={!session.canEdit()} variant="swatch" showHex={false} aria-label="Accent" on:sail-change={overview.onHeroColor} sx={style.w(full).h(20).minH(20)}></sail-color-picker>
              </stack>
              <row align="center" justify="between" sx={heroMeta}>
                <p sx={doorTitle}>Accent</p>
                <p sx={monoMeta}>{accentHex()}</p>
              </row>
            </column>
          </row>
          <if {session.catalogHint()}>
            <p catalog-hint sx={fieldHint}>Add to project to edit.</p>
          </if>
        </column>

        <grid cols="3" gap="3" editorial-band>
          <column panel="type" gap="3" sx={panelCard}>
            <p sx={sectionTitle}>Type</p>
            <column gap="3">
              <column gap="1">
                <p sx={fieldHint}>Display</p>
                <select type-role="display" size="sm" value={overview.typeDisplay()} disabled={!session.canEdit()} on:change={overview.onTypeSelect}>
                  <for {overview.displayFontOpts()} as="o" key={o.key}>
                    <option value={o.value}>{o.label}</option>
                  </for>
                </select>
              </column>
              <column gap="1">
                <p sx={fieldHint}>Body</p>
                <select type-role="body" size="sm" value={overview.typeBody()} disabled={!session.canEdit()} on:change={overview.onTypeSelect}>
                  <for {overview.bodyFontOpts()} as="o" key={o.key}>
                    <option value={o.value}>{o.label}</option>
                  </for>
                </select>
              </column>
              <column gap="1">
                <p sx={fieldHint}>Mono</p>
                <select type-role="mono" size="sm" value={overview.typeMono()} disabled={!session.canEdit()} on:change={overview.onTypeSelect}>
                  <for {overview.monoFontOpts()} as="o" key={o.key}>
                    <option value={o.value}>{o.label}</option>
                  </for>
                </select>
              </column>
            </column>
          </column>

          <column panel="icons" gap="3" sx={panelCard}>
            <p sx={sectionTitle}>Icons</p>
            <column gap="1">
              <p sx={fieldHint}>Pack</p>
              <select icon-pack size="sm" value={overview.iconPack()} disabled={!session.canEdit()} on:change={overview.onIconPack}>
                <option value="lucide">Lucide</option>
                <option value="phosphor">Phosphor</option>
              </select>
            </column>
            <if {overview.hasIconGlyphs()}>
              <grid cols="4" gap="2" icon-sample sx={style.padT(1)}>
                <for {overview.iconGlyphs()} as="g" key={g.key}>
                  <center shrink="0" sx={iconCell} title={g.name}>
                    <icon name={g.name} size="sm" label={g.name}></icon>
                  </center>
                </for>
              </grid>
            </if>
          </column>

          <column panel="density" gap="3" sx={panelCard}>
            <p sx={sectionTitle}>Density</p>
            <column density-seg gap="2" on:click={overview.onDensityClick}>
              <for {overview.densityOptions()} as="d" key={d.key}>
                <button
                  type="button"
                  size="sm"
                  density={d.id}
                  variant={d.on ? 'solid' : 'soft'}
                  tone={d.on ? 'primary' : 'muted'}
                  aria-pressed={d.on ? 'true' : 'false'}
                  disabled={!session.canEdit()}
                  sx={fullBtn}
                >
                  {d.label}
                </button>
              </for>
            </column>
          </column>
        </grid>

        <column doors-row gap="3">
          <p sx={sectionTitle}>More detail</p>
          <grid cols="2" gap="3" facet-doors on:click={session.onFacet}>
            <for {overview.facetDoors()} as="door" key={door.key}>
              <card facet={door.facet} sx={doorCard}>
                <row gap="2" align="center">
                  <icon name={door.icon} size="sm" shrink="0" sx={mutedIcon}></icon>
                  <column gap="0" grow min-w="0">
                    <p sx={doorTitle}>{door.label}</p>
                    <p sx={doorHint}>{door.hint}</p>
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
      sx={style.minW(panel-min-w-md).maxW(panel-max-w-md).basis(0).minH(0).h(full).borderLWidth(1).borderL(border-subtle).bg(background).overflow(hidden)}
    >
      <column grow gap="0" sx={style.minH(0).h(full).pad(4)}>
        <card
          overview-specimen
          preview-specimen
          mode={session.paintMode()}
          density={overview.specimenDensity()}
          padding="none"
          sx={style.minW(0).minH(0).h(full).overflow(hidden).shadow(lg)}
        >
          <column gap="0" grow sx={style.minH(0).h(full)}>
            <row
              specimen-chrome
              align="center"
              gap="2"
              nowrap
              shrink="0"
              sx={style.padX(3).h(8).minH(8).minW(0).borderBWidth(1).borderB(border-subtle).bg(surface-1)}
            >
              <row gap="2" align="center" nowrap shrink="0" aria-hidden="true" sx={style.minW(12)}>
                <stack shrink="0" sx={style.w(2).h(2).minW(2).minH(2).radius(full).bg(destructive)}></stack>
                <stack shrink="0" sx={style.w(2).h(2).minW(2).minH(2).radius(full).bg(warning)}></stack>
                <stack shrink="0" sx={style.w(2).h(2).minW(2).minH(2).radius(full).bg(success)}></stack>
              </row>
              <p grow min-w="0" sx={style.fontSize(xs).fg(muted-foreground).truncate()}>{session.nameEdit()}</p>
              <p sx={pathMeta} shrink="0">{session.paintMode()}</p>
            </row>
            <scroll-area grow sx={style.minH(0).basis(0).h(full).bg(background)}>
              <!--
                Plain host for density gap bind — layout column always ships sail-g-*
                which beats sx.gap(bind). stack rejects style.display (owns flex itself).
              -->
              <div
                specimen-body
                sx={style.display(flex).flexDir(column).w(full).minW(0).pad(bind(densityPad)).gap(bind(specimenGap))}
              >
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
                <div type-samples sx={style.display(flex).flexDir(column).gap(bind(specimenStackGap))}>
                  <p sx={style.fontFamily(display).fontSize(xl).fontWeight(semibold).tracking(tight)}>
                    {overview.typeDisplay()}
                  </p>
                  <p sx={style.fontFamily(body).fontSize(sm).leading(relaxed)}>
                    {overview.typeBody()} — product body copy.
                  </p>
                  <p sx={style.fontFamily(mono).fontSize(xs)}>
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
                <div sx={style.display(flex).flexDir(column).gap(bind(specimenStackGap))}>
                  <p sx={style.fontSize(xs).fg(muted-foreground)}>Email</p>
                  <input type="email" size={overview.controlSize()} placeholder="you@company.com" aria-label="Sample email" />
                </div>
                <card padding={overview.cardPadding()}>
                  <div sx={style.display(flex).flexDir(column).gap(bind(specimenStackGap))}>
                    <p sx={style.fontSize(md).fontWeight(semibold).fontFamily(heading)}>{session.nameEdit()}</p>
                    <p sx={style.fontSize(sm).fg(muted-foreground).leading(relaxed).fontFamily(body)}>
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
