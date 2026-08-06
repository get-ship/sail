<!--
  Theme color facet — thin shell: chrome + roles editor + specimen.
  Controller composition: session + color domain.
-->
<scope>
  const sectionTitle = style({ text: fontSize(sm).fontWeight(semibold) })
  const fieldHint = style({ paint: fg(muted-foreground), text: fontSize(xs) })
  const monoMeta = style({ paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) })
  const pathMeta = style({ paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) })
  const doorTitle = style({ text: fontWeight(semibold) })
  const editorCol = style({ box: padX(6).padT(6).padB(10).maxW(panel-max-w-lg) })
  const brandCard = style({ box: overflow(hidden).minW(0), paint: radius(lg).borderWidth(1).border(border-subtle).bg(surface-1) })
  // position:relative sized box — sail-color-picker host is display:contents; swatch fills this box.
  const brandSwatch = style({ box: position(relative).h(28).minH(28).w(full).overflow(hidden) }).when(stateEdit, { paint: cursor(pointer) }).when(stateLocked, { paint: cursor(default) })
  // Token-row chip: explicit size + relative so absolute-fill swatch has a real hit target.
  const tokenChip = style({ box: position(relative).w(9).h(9).minW(9).minH(9).overflow(hidden), paint: radius(md).borderWidth(1).border(border-subtle) })
  const heroMeta = style({ box: padX(3).padY(2) })
  const tokenRow = style({ box: padY(2).minW(0) })
  const seedDot = style({ box: w(4).h(4).minW(4).minH(4), paint: radius(full).borderWidth(1).border(border-subtle).bg(bind(swatchColor)) })
  const surfaceChip = style({ box: padX(2).padY(1), paint: radius(md).borderWidth(1).border(border-subtle).bg(surface-0) })
  const surfaceDot = style({ box: w(3).h(3).minW(3).minH(3), paint: radius(full).borderWidth(1).border(border-subtle).bg(bind(swatchColor)) })
  const bandStatus = style({ box: padX(4).padY(2) })
  const bandError = style({ box: padX(4).padY(2), paint: borderBWidth(1).borderB(border) })
</scope>

<controller>
import { effect, mountAmbient, signal } from '@ship/sail-runtime'
import { createThemeSession } from '../theme-session/theme-session'
import { createColorEdit } from './theme-color-edit'
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

<column
  surface="themes"
  theme-color-surface
  color-chrome
  gap="0"
  grow

>
  style({ box: minH(0).h(full).minW(0).overflow(hidden), paint: bg(background) })

  <theme-facet-chrome session={session}></theme-facet-chrome>

  <if {session.hasError()}>
    <row color-error align="center" gap="2" shrink="0">
      style(bandError)

      <p grow min-w="0">
        style({ paint: fg(destructive), text: fontSize(sm).truncate() })
{session.errorText()}</p>
      <button type="button" size="sm" variant="ghost" color-retry on:click={session.onRetry}>
        Retry
      </button>
    </row>
  </if>

  <if {session.loading()}>
    <row color-loading align="center" gap="2" shrink="0">
      style(bandStatus)

      <spinner size="sm"></spinner>
      <p>
        style(fieldHint)
Loading…</p>
    </row>
  </if>

<row
    color-workbench
    grow
    gap="0"
    nowrap
    align="stretch"

  >
  style({ box: minH(0).basis(0).h(full).minW(0).overflow(hidden), paint: bg(background) })

    <scroll-area
      color-editor
      grow
      min-w="0"

    >
      style({ box: minH(0).basis(0).h(full).minW(0), paint: bg(background) })

      <column gap="8">
        style(editorCol)

        <column color-header gap="2">
          <p>
            style({ text: fontSize(xl).fontWeight(semibold).tracking(tight) })
Color</p>
          <p>
            style(fieldHint)
Brand · surfaces · text · semantic</p>
        </column>

        <!-- Seed strip — Generate deferred (not built yet) -->
        <row seed-strip align="center" gap="3" nowrap>
          style({ box: pad(3).minW(0), paint: radius(lg).borderWidth(1).border(border-subtle).bg(surface-0) })

          <p shrink="0">
            style(fieldHint)
Seed</p>
          <row gap="1" align="center" nowrap grow min-w="0" hidden="true">
            <stack use:swatch={primaryHex()}>
              style(seedDot)
</stack>
            <stack use:swatch={secondaryHex()}>
              style(seedDot)
</stack>
            <stack use:swatch={accentHex()}>
              style(seedDot)
</stack>
            <stack use:swatch={bgHex()}>
              style(seedDot)
</stack>
            <stack use:swatch={fgHex()}>
              style(seedDot)
</stack>
            <stack use:swatch={mutedHex()}>
              style(seedDot)
</stack>
            <stack use:swatch={borderHex()}>
              style(seedDot)
</stack>
          </row>
          <button type="button" size="sm" variant="outline" tone="muted" seed-generate disabled on:click={color.onSeedGenerate} title="Coming soon">
            Generate…
          </button>
        </row>

        <!-- 1. Brand -->
        <column section="brand" gap="3">
          <row align="baseline" justify="between" gap="3" nowrap>
            <p>
              style(sectionTitle)
Brand</p>
            <p>
              style(pathMeta)
color.primary · secondary · accent</p>
          </row>
          <row brand-grid gap="3" align="stretch">
            <column brand-card token="primary" grow gap="0" min-w="0">
              style(brandCard)

              <stack token="primary">
                style(brandSwatch)

                <sail-color-picker
                  token="primary"
                  value={primaryHex()}
                  disabled={!session.canEdit()}
                  variant="swatch"
                  showHex={false}
                  label="Primary"
                  on:sail-change={color.onRoleColor}

                >
                  style({ box: w(full).h(full).minH(full) })
</sail-color-picker>
              </stack>
              <row align="center" justify="between">
                style(heroMeta)

                <column gap="0" min-w="0">
                  <p>
                    style(doorTitle)
Primary</p>
                  <p>
                    style(pathMeta)
color.primary</p>
                </column>
                <p>
                  style(monoMeta)
{primaryHex()}</p>
              </row>
            </column>
            <column brand-card token="secondary" grow gap="0" min-w="0">
              style(brandCard)

              <stack token="secondary">
                style(brandSwatch)

                <sail-color-picker
                  token="secondary"
                  value={secondaryHex()}
                  disabled={!session.canEdit()}
                  variant="swatch"
                  showHex={false}
                  label="Secondary"
                  on:sail-change={color.onRoleColor}

                >
                  style({ box: w(full).h(full).minH(full) })
</sail-color-picker>
              </stack>
              <row align="center" justify="between">
                style(heroMeta)

                <column gap="0" min-w="0">
                  <p>
                    style(doorTitle)
Secondary</p>
                  <p>
                    style(pathMeta)
color.secondary</p>
                </column>
                <p>
                  style(monoMeta)
{secondaryHex()}</p>
              </row>
            </column>
            <column brand-card token="accent" grow gap="0" min-w="0">
              style(brandCard)

              <stack token="accent">
                style(brandSwatch)

                <sail-color-picker
                  token="accent"
                  value={accentHex()}
                  disabled={!session.canEdit()}
                  variant="swatch"
                  showHex={false}
                  label="Accent"
                  on:sail-change={color.onRoleColor}

                >
                  style({ box: w(full).h(full).minH(full) })
</sail-color-picker>
              </stack>
              <row align="center" justify="between">
                style(heroMeta)

                <column gap="0" min-w="0">
                  <p>
                    style(doorTitle)
Accent</p>
                  <p>
                    style(pathMeta)
color.accent</p>
                </column>
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

        <!-- 2. Surfaces -->
        <column section="surfaces" gap="3">
          <row align="baseline" justify="between" gap="3" nowrap>
            <p>
              style(sectionTitle)
Surfaces</p>
            <p>
              style(pathMeta)
color.bg · surface · card</p>
          </row>
          <column token-list gap="1">
            <row token-row token="bg" align="center" gap="3" nowrap>
              style(tokenRow)

              <stack token-chip shrink="0">
                style(tokenChip)

              <sail-color-picker
                token="bg"
                value={bgHex()}
                disabled={!session.canEdit()}
                variant="swatch"
                showHex={false}
                label="Background"
                on:sail-change={color.onRoleColor}
              ></sail-color-picker>
              </stack>
              <column gap="0" grow min-w="0">
                <p>
                  style(doorTitle)
Background</p>
                <p>
                  style(pathMeta)
color.bg</p>
              </column>
              <p shrink="0">
                style(monoMeta)
{bgHex()}</p>
            </row>
            <row token-row token="surface1" align="center" gap="3" nowrap>
              style(tokenRow)

              <stack token-chip shrink="0">
                style(tokenChip)

              <sail-color-picker
                token="surface1"
                value={surface1Hex()}
                disabled={!session.canEdit()}
                variant="swatch"
                showHex={false}
                label="Surface 1"
                on:sail-change={color.onRoleColor}
              ></sail-color-picker>
              </stack>
              <column gap="0" grow min-w="0">
                <p>
                  style(doorTitle)
Surface 1</p>
                <p>
                  style(pathMeta)
color.surface.1</p>
              </column>
              <p shrink="0">
                style(monoMeta)
{surface1Hex()}</p>
            </row>
            <row token-row token="surface2" align="center" gap="3" nowrap>
              style(tokenRow)

              <stack token-chip shrink="0">
                style(tokenChip)

              <sail-color-picker
                token="surface2"
                value={surface2Hex()}
                disabled={!session.canEdit()}
                variant="swatch"
                showHex={false}
                label="Surface 2"
                on:sail-change={color.onRoleColor}
              ></sail-color-picker>
              </stack>
              <column gap="0" grow min-w="0">
                <p>
                  style(doorTitle)
Surface 2</p>
                <p>
                  style(pathMeta)
color.surface.2</p>
              </column>
              <p shrink="0">
                style(monoMeta)
{surface2Hex()}</p>
            </row>
            <row token-row token="card" align="center" gap="3" nowrap>
              style(tokenRow)

              <stack token-chip shrink="0">
                style(tokenChip)

              <sail-color-picker
                token="card"
                value={cardHex()}
                disabled={!session.canEdit()}
                variant="swatch"
                showHex={false}
                label="Card"
                on:sail-change={color.onRoleColor}
              ></sail-color-picker>
              </stack>
              <column gap="0" grow min-w="0">
                <p>
                  style(doorTitle)
Card</p>
                <p>
                  style(pathMeta)
color.surface.3</p>
              </column>
              <p shrink="0">
                style(monoMeta)
{cardHex()}</p>
            </row>
          </column>
          <row surface-chips gap="2" align="center" nowrap hidden="true">
            <row align="center" gap="1">
              style(surfaceChip)

              <stack use:swatch={bgHex()}>
                style(surfaceDot)
</stack>
              <p>
                style(pathMeta)
bg</p>
            </row>
            <row align="center" gap="1">
              style(surfaceChip)

              <stack use:swatch={surface1Hex()}>
                style(surfaceDot)
</stack>
              <p>
                style(pathMeta)
surface-1</p>
            </row>
            <row align="center" gap="1">
              style(surfaceChip)

              <stack use:swatch={surface2Hex()}>
                style(surfaceDot)
</stack>
              <p>
                style(pathMeta)
surface-2</p>
            </row>
            <row align="center" gap="1">
              style(surfaceChip)

              <stack use:swatch={cardHex()}>
                style(surfaceDot)
</stack>
              <p>
                style(pathMeta)
card</p>
            </row>
          </row>
        </column>

        <!-- 3. Text & borders -->
        <column section="text" gap="3">
          <row align="baseline" justify="between" gap="3" nowrap>
            <p>
              style(sectionTitle)
Text &amp; borders</p>
            <p>
              style(pathMeta)
color.fg · muted · border · ring</p>
          </row>
          <column token-list gap="1">
            <row token-row token="fg" align="center" gap="3" nowrap>
              style(tokenRow)

              <stack token-chip shrink="0">
                style(tokenChip)

              <sail-color-picker
                token="fg"
                value={fgHex()}
                disabled={!session.canEdit()}
                variant="swatch"
                showHex={false}
                label="Foreground"
                on:sail-change={color.onRoleColor}
              ></sail-color-picker>
              </stack>
              <column gap="0" grow min-w="0">
                <p>
                  style(doorTitle)
Foreground</p>
                <p>
                  style(pathMeta)
color.foreground</p>
              </column>
              <p shrink="0">
                style(monoMeta)
{fgHex()}</p>
            </row>
            <row token-row token="muted" align="center" gap="3" nowrap>
              style(tokenRow)

              <stack token-chip shrink="0">
                style(tokenChip)

              <sail-color-picker
                token="muted"
                value={mutedHex()}
                disabled={!session.canEdit()}
                variant="swatch"
                showHex={false}
                label="Muted"
                on:sail-change={color.onRoleColor}
              ></sail-color-picker>
              </stack>
              <column gap="0" grow min-w="0">
                <p>
                  style(doorTitle)
Muted</p>
                <p>
                  style(pathMeta)
color.muted</p>
              </column>
              <p shrink="0">
                style(monoMeta)
{mutedHex()}</p>
            </row>
            <row token-row token="border" align="center" gap="3" nowrap>
              style(tokenRow)

              <stack token-chip shrink="0">
                style(tokenChip)

              <sail-color-picker
                token="border"
                value={borderHex()}
                disabled={!session.canEdit()}
                variant="swatch"
                showHex={false}
                label="Border"
                on:sail-change={color.onRoleColor}
              ></sail-color-picker>
              </stack>
              <column gap="0" grow min-w="0">
                <p>
                  style(doorTitle)
Border</p>
                <p>
                  style(pathMeta)
color.border</p>
              </column>
              <p shrink="0">
                style(monoMeta)
{borderHex()}</p>
            </row>
            <row token-row token="ring" align="center" gap="3" nowrap>
              style(tokenRow)

              <stack token-chip shrink="0">
                style(tokenChip)

              <sail-color-picker
                token="ring"
                value={ringHex()}
                disabled={!session.canEdit()}
                variant="swatch"
                showHex={false}
                label="Ring"
                on:sail-change={color.onRoleColor}
              ></sail-color-picker>
              </stack>
              <column gap="0" grow min-w="0">
                <p>
                  style(doorTitle)
Ring</p>
                <p>
                  style(pathMeta)
color.ring</p>
              </column>
              <p shrink="0">
                style(monoMeta)
{ringHex()}</p>
            </row>
          </column>
        </column>

        <!-- 4. Semantic -->
        <column section="semantic" gap="3">
          <row align="baseline" justify="between" gap="3" nowrap>
            <p>
              style(sectionTitle)
Semantic</p>
            <p>
              style(pathMeta)
color.success · warning · danger · info</p>
          </row>
          <column token-list gap="1">
            <row token-row token="success" sem="success" align="center" gap="3" nowrap>
              style(tokenRow)

              <stack token-chip shrink="0">
                style(tokenChip)

              <sail-color-picker
                token="success"
                value={successHex()}
                disabled={!session.canEdit()}
                variant="swatch"
                showHex={false}
                label="Success"
                on:sail-change={color.onRoleColor}
              ></sail-color-picker>
              </stack>
              <column gap="0" grow min-w="0">
                <p>
                  style(doorTitle)
Success</p>
                <p>
                  style(pathMeta)
color.success</p>
              </column>
              <p shrink="0">
                style(monoMeta)
{successHex()}</p>
            </row>
            <row token-row token="warning" sem="warning" align="center" gap="3" nowrap>
              style(tokenRow)

              <stack token-chip shrink="0">
                style(tokenChip)

              <sail-color-picker
                token="warning"
                value={warningHex()}
                disabled={!session.canEdit()}
                variant="swatch"
                showHex={false}
                label="Warning"
                on:sail-change={color.onRoleColor}
              ></sail-color-picker>
              </stack>
              <column gap="0" grow min-w="0">
                <p>
                  style(doorTitle)
Warning</p>
                <p>
                  style(pathMeta)
color.warning</p>
              </column>
              <p shrink="0">
                style(monoMeta)
{warningHex()}</p>
            </row>
            <row token-row token="danger" sem="danger" align="center" gap="3" nowrap>
              style(tokenRow)

              <stack token-chip shrink="0">
                style(tokenChip)

              <sail-color-picker
                token="danger"
                value={dangerHex()}
                disabled={!session.canEdit()}
                variant="swatch"
                showHex={false}
                label="Danger"
                on:sail-change={color.onRoleColor}
              ></sail-color-picker>
              </stack>
              <column gap="0" grow min-w="0">
                <p>
                  style(doorTitle)
Danger</p>
                <p>
                  style(pathMeta)
color.danger</p>
              </column>
              <p shrink="0">
                style(monoMeta)
{dangerHex()}</p>
            </row>
            <row token-row token="info" sem="info" align="center" gap="3" nowrap>
              style(tokenRow)

              <stack token-chip shrink="0">
                style(tokenChip)

              <sail-color-picker
                token="info"
                value={infoHex()}
                disabled={!session.canEdit()}
                variant="swatch"
                showHex={false}
                label="Info"
                on:sail-change={color.onRoleColor}
              ></sail-color-picker>
              </stack>
              <column gap="0" grow min-w="0">
                <p>
                  style(doorTitle)
Info</p>
                <p>
                  style(pathMeta)
color.info</p>
              </column>
              <p shrink="0">
                style(monoMeta)
{infoHex()}</p>
            </row>
          </column>
        </column>

        <column color-footer gap="2">
          style({ box: padT(2), paint: borderTWidth(1).borderT(border-subtle) })

          <p>
            style(fieldHint)

            
          </p>
        </column>
      </column>
    </scroll-area>

    <theme-color-specimen session={session}></theme-color-specimen>
  </row>

  <toast-region position="bottom-end" color-toast-region>
    <for {session.toastItems()} as="t" key={t.key}>
      <toast tone={t.tone} color-toast on:sail-toast-dismiss={session.onToastDismiss}>
        {t.message}
      </toast>
    </for>
  </toast-region>
</column>
