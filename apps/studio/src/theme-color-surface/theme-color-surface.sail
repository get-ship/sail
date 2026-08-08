<!--
  Theme color facet — thin shell: chrome + roles editor + specimen.
  Controller composition: session + color domain.
-->
<scope>
  style sectionTitle = { text: fontSize(sm).fontWeight(semibold) }
  style fieldHint = { paint: fg(muted-foreground), text: fontSize(xs) }
  style monoMeta = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style pathMeta = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style doorTitle = { text: fontWeight(semibold) }
  style editorCol = { box: padX(6).padT(6).padB(10).maxW(panel-max-w-lg) }
  style brandCard = { box: overflow(hidden).minW(0), paint: radius(lg).borderWidth(1).border(border-subtle).bg(surface-1) }
  // position:relative sized box — sail-color-picker host is display:contents; swatch fills this box.
  style brandSwatch = { box: position(relative).h(28).minH(28).w(full).overflow(hidden), when: { stateEdit: { paint: cursor(pointer) }, stateLocked: { paint: cursor(default) } } }
  // Token-row chip: explicit size + relative so absolute-fill swatch has a real hit target.
  style tokenChip = { box: position(relative).w(9).h(9).minW(9).minH(9).overflow(hidden), paint: radius(md).borderWidth(1).border(border-subtle) }
  style heroMeta = { box: padX(3).padY(2) }
  style tokenRow = { box: padY(2).minW(0) }
  style seedDot = { box: w(4).h(4).minW(4).minH(4), paint: radius(full).borderWidth(1).border(border-subtle).bg(bind(swatchColor)) }
  style surfaceChip = { box: padX(2).padY(1), paint: radius(md).borderWidth(1).border(border-subtle).bg(surface-0) }
  style surfaceDot = { box: w(3).h(3).minW(3).minH(3), paint: radius(full).borderWidth(1).border(border-subtle).bg(bind(swatchColor)) }
  style bandStatus = { box: padX(4).padY(2) }
  style bandError = { box: padX(4).padY(2), paint: borderBWidth(1).borderB(border) }
  type ColorTokenRowProps = {
    token: string
    value: string
    label: string
    path: string
    disabled: boolean
    onChange: any
  }
</scope>

<partial color-token-row props: ColorTokenRowProps>
  <row token-row token={token} sem={token} align="center" gap="3" nowrap style(tokenRow)>
    <stack token-chip style(tokenChip, { box: shrink(0) })>
      <sail-color-picker
        token={token}
        value={value}
        disabled={disabled}
        variant="swatch"
        showHex={false}
        label={label}
        on-change={onChange}
      ></sail-color-picker>
    </stack>
    <column gap="0" style({ box: grow(1).minW(0) })>
      <p style(doorTitle)>{label}</p>
      <p style(pathMeta)>{path}</p>
    </column>
    <p style(monoMeta, { box: shrink(0) })>{value}</p>
  </row>
</partial>


<controller>
import { effect, mountAmbient, signal } from '@ship/sail-runtime'
import { createThemeSession } from '../compositions/theme-session/theme-session'
import { createColorEdit } from '../compositions/theme-color-surface/theme-color-edit'
// createThemeColorSurface() lives in theme-color-surface-vm for tests
export default controller(() => {
  const session = createThemeSession({
    facet: 'color',
    editGuardMessage: 'Add this theme to the project to edit color.',
    saveSuccessMessage: 'Color roles saved to DTCG source.',
  })
  const color = createColorEdit(session)
  const primaryHex = signal(color.primaryHex())
  const secondaryHex = signal(color.secondaryHex())
  const accentHex = signal(color.accentHex())
  const bgHex = signal(color.bgHex())
  const surface1Hex = signal(color.surface1Hex())
  const surface2Hex = signal(color.surface2Hex())
  const cardHex = signal(color.cardHex())
  const fgHex = signal(color.fgHex())
  const mutedHex = signal(color.mutedHex())
  const borderHex = signal(color.borderHex())
  const ringHex = signal(color.ringHex())
  const successHex = signal(color.successHex())
  const warningHex = signal(color.warningHex())
  const dangerHex = signal(color.dangerHex())
  const infoHex = signal(color.infoHex())
  mountAmbient(() =>
    effect(() => {
      primaryHex.set(color.primaryHex())
      secondaryHex.set(color.secondaryHex())
      accentHex.set(color.accentHex())
      bgHex.set(color.bgHex())
      surface1Hex.set(color.surface1Hex())
      surface2Hex.set(color.surface2Hex())
      cardHex.set(color.cardHex())
      fgHex.set(color.fgHex())
      mutedHex.set(color.mutedHex())
      borderHex.set(color.borderHex())
      ringHex.set(color.ringHex())
      successHex.set(color.successHex())
      warningHex.set(color.warningHex())
      dangerHex.set(color.dangerHex())
      infoHex.set(color.infoHex())
    }),
  )
  return {
    session,
    color,
    primaryHex,
    secondaryHex,
    accentHex,
    bgHex,
    surface1Hex,
    surface2Hex,
    cardHex,
    fgHex,
    mutedHex,
    borderHex,
    ringHex,
    successHex,
    warningHex,
    dangerHex,
    infoHex,
  }
})
</controller>

<column surface="themes" theme-color-surface color-chrome gap="0" style({  box: grow(1).minH(0).h(full).minW(0).overflow(hidden), paint: bg(background)  })>
  <theme-facet-chrome session={session}></theme-facet-chrome>

  <if {session.hasError()}>
    <row color-error align="center" gap="2" style(bandError, { box: shrink(0) })>
      <p style({ box: grow(1).minW(0), paint: fg(destructive), text: fontSize(sm).truncate() })>{session.errorText()}</p>
      <button type="button" size="sm" variant="ghost" color-retry on:click={session.onRetry}>
        Retry
      </button>
    </row>
  </if>

  <if {session.loading()}>
    <row color-loading align="center" gap="2" style(bandStatus, { box: shrink(0) })>
      <spinner size="sm"></spinner>
      <p style(fieldHint)>Loading…</p>
    </row>
  </if>

<row color-workbench gap="0" nowrap align="stretch" style({  box: grow(1).minH(0).basis(0).h(full).minW(0).overflow(hidden), paint: bg(background)  })>
    <scroll-area color-editor style({  box: grow(1).minW(0).minH(0).basis(0).h(full), paint: bg(background)  })>
      <column gap="8" style(editorCol)>
        <column color-header gap="2">
          <p style({ text: fontSize(xl).fontWeight(semibold).tracking(tight) })>Color</p>
          <p style(fieldHint)>Brand · surfaces · text · semantic</p>
        </column>

        <!-- Seed strip — Generate deferred (not built yet) -->
        <row seed-strip align="center" gap="3" nowrap style({ box: pad(3).minW(0), paint: radius(lg).borderWidth(1).border(border-subtle).bg(surface-0) })>
          <p style(fieldHint, { box: shrink(0) })>Seed</p>
          <row gap="1" align="center" nowrap hidden="true" style({ box: grow(1).minW(0) })>
            <stack use:swatch={primaryHex()} style(seedDot)></stack>
            <stack use:swatch={secondaryHex()} style(seedDot)></stack>
            <stack use:swatch={accentHex()} style(seedDot)></stack>
            <stack use:swatch={bgHex()} style(seedDot)></stack>
            <stack use:swatch={fgHex()} style(seedDot)></stack>
            <stack use:swatch={mutedHex()} style(seedDot)></stack>
            <stack use:swatch={borderHex()} style(seedDot)></stack>
          </row>
          <button type="button" size="sm" variant="outline" tone="muted" seed-generate disabled on:click={color.onSeedGenerate} title="Coming soon">
            Generate…
          </button>
        </row>

        <!-- 1. Brand -->
        <column section="brand" gap="3">
          <row align="baseline" justify="between" gap="3" nowrap>
            <p style(sectionTitle)>Brand</p>
            <p style(pathMeta)>color.primary · secondary · accent</p>
          </row>
          <row brand-grid gap="3" align="stretch">
            <column brand-card token="primary" gap="0" style(brandCard, { box: grow(1).minW(0) })>
              <stack token="primary" style(brandSwatch)>
                <sail-color-picker
                  token="primary"
                  value={primaryHex()}
                  disabled={!session.canEdit()}
                  variant="swatch"
                  showHex={false}
                  label="Primary"
                  on-change={color.onRoleColor} style({ box: w(full).h(full).minH(full) })></sail-color-picker>
              </stack>
              <row align="center" justify="between" style(heroMeta)>
                <column gap="0" style({ box: minW(0) })>
                  <p style(doorTitle)>Primary</p>
                  <p style(pathMeta)>color.primary</p>
                </column>
                <p style(monoMeta)>{primaryHex()}</p>
              </row>
            </column>
            <column brand-card token="secondary" gap="0" style(brandCard, { box: grow(1).minW(0) })>
              <stack token="secondary" style(brandSwatch)>
                <sail-color-picker
                  token="secondary"
                  value={secondaryHex()}
                  disabled={!session.canEdit()}
                  variant="swatch"
                  showHex={false}
                  label="Secondary"
                  on-change={color.onRoleColor} style({ box: w(full).h(full).minH(full) })></sail-color-picker>
              </stack>
              <row align="center" justify="between" style(heroMeta)>
                <column gap="0" style({ box: minW(0) })>
                  <p style(doorTitle)>Secondary</p>
                  <p style(pathMeta)>color.secondary</p>
                </column>
                <p style(monoMeta)>{secondaryHex()}</p>
              </row>
            </column>
            <column brand-card token="accent" gap="0" style(brandCard, { box: grow(1).minW(0) })>
              <stack token="accent" style(brandSwatch)>
                <sail-color-picker
                  token="accent"
                  value={accentHex()}
                  disabled={!session.canEdit()}
                  variant="swatch"
                  showHex={false}
                  label="Accent"
                  on-change={color.onRoleColor} style({ box: w(full).h(full).minH(full) })></sail-color-picker>
              </stack>
              <row align="center" justify="between" style(heroMeta)>
                <column gap="0" style({ box: minW(0) })>
                  <p style(doorTitle)>Accent</p>
                  <p style(pathMeta)>color.accent</p>
                </column>
                <p style(monoMeta)>{accentHex()}</p>
              </row>
            </column>
          </row>
          <if {session.catalogHint()}>
            <p catalog-hint style(fieldHint)>Add to project to edit.</p>
          </if>
        </column>

        <!-- 2. Surfaces -->
        <column section="surfaces" gap="3">
          <row align="baseline" justify="between" gap="3" nowrap>
            <p style(sectionTitle)>Surfaces</p>
            <p style(pathMeta)>color.bg · surface · card</p>
          </row>
          <column token-list gap="1">
            <color-token-row
              token="bg"
              value={bgHex()}
              label="Background"
              path="color.bg"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
            <color-token-row
              token="surface1"
              value={surface1Hex()}
              label="Surface 1"
              path="color.surface.1"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
            <color-token-row
              token="surface2"
              value={surface2Hex()}
              label="Surface 2"
              path="color.surface.2"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
            <color-token-row
              token="card"
              value={cardHex()}
              label="Card"
              path="color.surface.3"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
          </column>
          <row surface-chips gap="2" align="center" nowrap hidden="true">
            <row align="center" gap="1" style(surfaceChip)>
              <stack use:swatch={bgHex()} style(surfaceDot)></stack>
              <p style(pathMeta)>bg</p>
            </row>
            <row align="center" gap="1" style(surfaceChip)>
              <stack use:swatch={surface1Hex()} style(surfaceDot)></stack>
              <p style(pathMeta)>surface-1</p>
            </row>
            <row align="center" gap="1" style(surfaceChip)>
              <stack use:swatch={surface2Hex()} style(surfaceDot)></stack>
              <p style(pathMeta)>surface-2</p>
            </row>
            <row align="center" gap="1" style(surfaceChip)>
              <stack use:swatch={cardHex()} style(surfaceDot)></stack>
              <p style(pathMeta)>card</p>
            </row>
          </row>
        </column>

        <!-- 3. Text & borders -->
        <column section="text" gap="3">
          <row align="baseline" justify="between" gap="3" nowrap>
            <p style(sectionTitle)>Text &amp; borders</p>
            <p style(pathMeta)>color.fg · muted · border · ring</p>
          </row>
          <column token-list gap="1">
            <color-token-row
              token="fg"
              value={fgHex()}
              label="Foreground"
              path="color.foreground"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
            <color-token-row
              token="muted"
              value={mutedHex()}
              label="Muted"
              path="color.muted"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
            <color-token-row
              token="border"
              value={borderHex()}
              label="Border"
              path="color.border"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
            <color-token-row
              token="ring"
              value={ringHex()}
              label="Ring"
              path="color.ring"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
          </column>
        </column>

        <!-- 4. Semantic -->
        <column section="semantic" gap="3">
          <row align="baseline" justify="between" gap="3" nowrap>
            <p style(sectionTitle)>Semantic</p>
            <p style(pathMeta)>color.success · warning · danger · info</p>
          </row>
          <column token-list gap="1">
            <color-token-row
              token="success"
              value={successHex()}
              label="Success"
              path="color.success"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
            <color-token-row
              token="warning"
              value={warningHex()}
              label="Warning"
              path="color.warning"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
            <color-token-row
              token="danger"
              value={dangerHex()}
              label="Danger"
              path="color.danger"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
            <color-token-row
              token="info"
              value={infoHex()}
              label="Info"
              path="color.info"
              disabled={!session.canEdit()}
              on-change={color.onRoleColor}
            />
          </column>
        </column>

        <column color-footer gap="2" style({ box: padT(2), paint: borderTWidth(1).borderT(border-subtle) })>
          <p style(fieldHint)>
            
          </p>
        </column>
      </column>
    </scroll-area>

    <theme-color-specimen session={session}></theme-color-specimen>
  </row>

  <toast-region position="bottom-end" color-toast-region>
    <for {session.toastItems() as t} key={t.key}>
      <toast tone={t.tone} color-toast on-dismiss={session.onToastDismiss}>
        {t.message}
      </toast>
    </for>
  </toast-region>
</column>
