<!--
  Theme shape facet — specimen-first character dials (mock SSOT: shape.html).
  Controls ~panel | large stage. Outline radius chips; elev / border / glass.
  Live geometry via freeform px/shadow bind (not CSS-var product state).
-->
<scope>
  const sectionTitle = style({ text: fontSize(sm).fontWeight(semibold) })
  const fieldHint = style({ paint: fg(muted-foreground), text: fontSize(xs) })
  const pathMeta = style({ paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) })
  const monoMeta = style({ paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) })
  const doorTitle = style({ text: fontWeight(semibold) })
  const eyebrow = style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).tracking(wide).textCase(uppercase) })
  const controlsCol = style({ box: padX(4).padT(4).padB(8).minW(0) })
  const bandStatus = style({ box: padX(4).padY(2) })
  const bandError = style({ box: padX(4).padY(2), paint: borderBWidth(1).borderB(border) })
  const fullBtn = style({ box: w(full) })
  const chipCol = style({ box: minW(0) })
  const sliderRow = style({ box: minW(0) })
  const sectionPanel = style({ box: pad(3), paint: radius(lg).borderWidth(1).border(border-subtle).bg(surface-1) })
  const tokenStrip = style({ box: padX(2).padY(1), paint: fg(muted-foreground).radius(sm).bg(surface-0), text: fontSize(xs).fontFamily(mono) })
  // Outline chip shells — continuous radius via use:liveBox (token radius cannot drag continuously).
  const ladderChip = style({ box: w(9).h(7).minW(9).minH(7), paint: borderWidth(1).border(border).bg(transparent) })
  const ladderChipXl = style({ box: w(10).h(8).minW(10).minH(8), paint: borderWidth(1).border(border).bg(transparent) })
  const cornerChip = style({ box: w(8).h(6).minW(8).minH(6), paint: borderWidth(1).border(border).bg(transparent) })
  // Elev chip marks — stepped vocabulary (none dashed → xl deep lift).
  const elevMarkNone = style({ box: w(5).h(4).minW(5).minH(4), paint: radius(sm).borderWidth(1).border(dashed).border(border-subtle).bg(background) })
  const elevMarkXs = style({ box: w(5).h(4).minW(5).minH(4), paint: radius(sm).borderWidth(1).border(border-subtle).bg(background).shadow(xs) })
  const elevMarkSm = style({ box: w(5).h(4).minW(5).minH(4), paint: radius(sm).borderWidth(1).border(border-subtle).bg(background).shadow(sm) })
  const elevMarkMd = style({ box: w(5).h(4).minW(5).minH(4), paint: radius(sm).borderWidth(1).border(border-subtle).bg(background).shadow(md) })
  const elevMarkLg = style({ box: w(5).h(4).minW(5).minH(4), paint: radius(sm).borderWidth(1).border(border-subtle).bg(background).shadow(lg) })
  const elevMarkXl = style({ box: w(5).h(4).minW(5).minH(4), paint: radius(sm).borderWidth(1).border(border-subtle).bg(background).shadow(xl) })
  const statCell = style({ box: pad(2_5).minW(0), paint: radius(sm).borderWidth(1).border(border-subtle) })
  const stageSolid = style({ box: minH(0).h(full), paint: bg(background) })
  // Primary specimen card: radius.lg + elev + border + freeform fill (frost α live).
  const liveCard = style({ box: pad(5), paint: border(border) })
  const glassMini = style({ box: maxW(panel-max-w-sm).w(full).pad(4), paint: border(border) })
</scope>

<controller>
import { effect, mountAmbient, signal } from '@ship/sail-runtime'
import { createThemeSession } from '../theme-session/theme-session'
import { watchPreviewHost } from '../theme-session/bind-preview-host'
import { createShapeEdit } from './theme-shape-edit'
export default controller(() => {
  const session = createThemeSession({
    facet: 'shape',
    editGuardMessage: 'Add this theme to the project to edit shape.',
    saveSuccessMessage: 'Shape tokens saved to DTCG source.',
  })
  const shape = createShapeEdit(session)
  const liveCardRadius = signal(shape.liveCardRadius())
  const liveCardShadow = signal(shape.liveCardShadow())
  const liveBorderW = signal(shape.liveBorderW())
  const liveCardFill = signal(shape.liveCardFill())
  const liveLadderSm = signal(shape.liveLadderSm())
  const liveLadderMd = signal(shape.liveLadderMd())
  const liveLadderLg = signal(shape.liveLadderLg())
  const liveLadderXl = signal(shape.liveLadderXl())
  const liveCornerSm = signal(shape.liveCornerSm())
  const liveCornerMd = signal(shape.liveCornerMd())
  const liveCornerLg = signal(shape.liveCornerLg())
  const liveCornerXl = signal(shape.liveCornerXl())
  const liveCardBox = signal('')
  const liveGlassBox = signal('')
  const liveLadderSmBox = signal('')
  const liveLadderMdBox = signal('')
  const liveLadderLgBox = signal('')
  const liveLadderXlBox = signal('')
  const liveCornerSmBox = signal('')
  const liveCornerMdBox = signal('')
  const liveCornerLgBox = signal('')
  const liveCornerXlBox = signal('')
  const packCard = () => {
    const r = shape.liveCardRadius()
    const sh = shape.liveCardShadow()
    const bw = shape.liveBorderW()
    const fill = shape.liveCardFill()
    return `border-radius:${r};box-shadow:${sh};border-width:${bw};background:${fill}`
  }
  const packRadius = (r: string) => `border-radius:${r}`
  mountAmbient(() =>
    effect(() => {
      liveCardRadius.set(shape.liveCardRadius())
      liveCardShadow.set(shape.liveCardShadow())
      liveBorderW.set(shape.liveBorderW())
      liveCardFill.set(shape.liveCardFill())
      liveLadderSm.set(shape.liveLadderSm())
      liveLadderMd.set(shape.liveLadderMd())
      liveLadderLg.set(shape.liveLadderLg())
      liveLadderXl.set(shape.liveLadderXl())
      liveCornerSm.set(shape.liveCornerSm())
      liveCornerMd.set(shape.liveCornerMd())
      liveCornerLg.set(shape.liveCornerLg())
      liveCornerXl.set(shape.liveCornerXl())
      liveCardBox.set(packCard())
      liveGlassBox.set(packCard())
      liveLadderSmBox.set(packRadius(shape.liveLadderSm()))
      liveLadderMdBox.set(packRadius(shape.liveLadderMd()))
      liveLadderLgBox.set(packRadius(shape.liveLadderLg()))
      liveLadderXlBox.set(packRadius(shape.liveLadderXl()))
      liveCornerSmBox.set(packRadius(shape.liveCornerSm()))
      liveCornerMdBox.set(packRadius(shape.liveCornerMd()))
      liveCornerLgBox.set(packRadius(shape.liveCornerLg()))
      liveCornerXlBox.set(packRadius(shape.liveCornerXl()))
    }),
  )
  mount((host) => watchPreviewHost(host, (el) => session.attachPreviewHost(el)))
  return {
    session,
    shape,
    liveCardBox,
    liveGlassBox,
    liveLadderSmBox,
    liveLadderMdBox,
    liveLadderLgBox,
    liveLadderXlBox,
    liveCornerSmBox,
    liveCornerMdBox,
    liveCornerLgBox,
    liveCornerXlBox,
  }
})
</controller>

<column surface="themes" theme-shape-surface shape-chrome gap="0">
  style({  box: grow(1).minH(0).h(full).minW(0).overflow(hidden), paint: bg(background)  })

  <theme-facet-chrome session={session}></theme-facet-chrome>

  <if {session.hasError()}>
    <row shape-error align="center" gap="2">
      style([ style(bandError), style({ box: shrink(0) }) ])

      <p>
        style({ box: grow(1).minW(0), paint: fg(destructive), text: fontSize(sm).truncate() })
{session.errorText()}</p>
      <button type="button" size="sm" variant="ghost" shape-retry on:click={session.onRetry}>
        Retry
      </button>
    </row>
  </if>

  <if {session.loading()}>
    <row shape-loading align="center" gap="2">
      style([ style(bandStatus), style({ box: shrink(0) }) ])

      <spinner size="sm"></spinner>
      <p>
        style(fieldHint)
Loading…</p>
    </row>
  </if>

  <!-- Shape: left controls ~panel | right LARGE specimen (inverted vs Color). -->
  <row shape-workbench gap="0" nowrap align="stretch">
    style({  box: grow(1).minH(0).basis(0).h(full).minW(0).overflow(hidden), paint: bg(background)  })

    <scroll-area shape-controls>
      style({  box: shrink(0).minW(panel-min-w-sm).maxW(panel-max-w-sm).w(panel-max-w-sm).minH(0).h(full), paint: borderRWidth(1).borderR(border-subtle).bg(background)  })

      <column gap="4">
        style(controlsCol)

        <column shape-header gap="2">
          <p>
            style(eyebrow)
Globals · character</p>
          <p>
            style({ text: fontSize(2xl).fontWeight(semibold).tracking(tight).fontFamily(heading) })
Shape</p>
          <p>
            style(fieldHint)

            Corners, elevation, and material. Theme character — not per-component skins.
          </p>
          <row gap="2" align="center" nowrap>
            <badge tone="muted" size="sm" variant="outline" pill>{session.modesMeta()}</badge>
            <badge tone="muted" size="sm" variant="soft" pill>{shape.statRecipe()}</badge>
          </row>
          <if {session.catalogHint()}>
            <p catalog-hint>
              style(fieldHint)
Add to project to edit.</p>
          </if>
        </column>

        <!-- Quick patterns (recipes) -->
        <column section="recipes" gap="2">
          style(sectionPanel)

          <row align="baseline" justify="between" gap="2" nowrap>
            <p>
              style(sectionTitle)
Quick patterns</p>
            <p>
              style(pathMeta)
recipes</p>
          </row>
          <grid cols="3" gap="2" recipe-grid on:click={shape.onRecipeClick}>
            <for {shape.recipes() as r} key={r.key}>
              <button
                type="button"
                size="sm"
                recipe={r.id}
                variant={r.on ? 'solid' : 'outline'}
                tone={r.on ? 'primary' : 'muted'}
                pressed={r.on ? 'true' : 'false'}
                disabled={!session.canEdit()}

              >
                style(fullBtn)

                {r.label}
              </button>
            </for>
          </grid>
        </column>

        <!-- Radius scale — outline corner chips + master -->
        <column section="radius" gap="3">
          style(sectionPanel)

          <row align="baseline" justify="between" gap="2" nowrap>
            <p>
              style(sectionTitle)
Radius scale</p>
            <p>
              style(pathMeta)
{shape.radiusReadout()}</p>
          </row>
          <row radius-chips gap="2" align="stretch" nowrap on:click={shape.onRadiusChip} label="Radius steps">
            <button
              type="button"
              size="sm"
              radius="sm"
              variant="outline"
              tone={shape.focusStep() === 'sm' ? 'primary' : 'muted'}
              pressed={shape.focusStep() === 'sm' ? 'true' : 'false'}
              title="radius.sm"

            >
              style(chipCol)

              <column gap="1" align="center">
                <stack hidden="true" use:liveBox={liveCornerSmBox()}>
                  style(cornerChip)
</stack>
                <p>
                  style(pathMeta)
sm</p>
                <p>
                  style(monoMeta)
{shape.radiusSmLabel()}</p>
              </column>
            </button>
            <button
              type="button"
              size="sm"
              radius="md"
              variant="outline"
              tone={shape.focusStep() === 'md' ? 'primary' : 'muted'}
              pressed={shape.focusStep() === 'md' ? 'true' : 'false'}
              title="radius.md"

            >
              style(chipCol)

              <column gap="1" align="center">
                <stack hidden="true" use:liveBox={liveCornerMdBox()}>
                  style(cornerChip)
</stack>
                <p>
                  style(pathMeta)
md</p>
                <p>
                  style(monoMeta)
{shape.radiusMdLabel()}</p>
              </column>
            </button>
            <button
              type="button"
              size="sm"
              radius="lg"
              variant="outline"
              tone={shape.focusStep() === 'lg' ? 'primary' : 'muted'}
              pressed={shape.focusStep() === 'lg' ? 'true' : 'false'}
              title="radius.lg"

            >
              style(chipCol)

              <column gap="1" align="center">
                <stack hidden="true" use:liveBox={liveCornerLgBox()}>
                  style(cornerChip)
</stack>
                <p>
                  style(pathMeta)
lg</p>
                <p>
                  style(monoMeta)
{shape.radiusLgLabel()}</p>
              </column>
            </button>
            <button
              type="button"
              size="sm"
              radius="xl"
              variant="outline"
              tone={shape.focusStep() === 'xl' ? 'primary' : 'muted'}
              pressed={shape.focusStep() === 'xl' ? 'true' : 'false'}
              title="radius.xl"

            >
              style(chipCol)

              <column gap="1" align="center">
                <stack hidden="true" use:liveBox={liveCornerXlBox()}>
                  style(cornerChip)
</stack>
                <p>
                  style(pathMeta)
xl</p>
                <p>
                  style(monoMeta)
{shape.radiusXlLabel()}</p>
              </column>
            </button>
          </row>
          <column radius-master gap="1">
            style(sliderRow)

            <row align="center" justify="between" gap="2" nowrap>
              <p>
                style(fieldHint)
Master · family</p>
              <p>
                style(monoMeta)
{shape.masterLabel()}</p>
            </row>
            <input
              type="range"
              min="0"
              max="24"
              step="1"
              value={shape.masterPx()}
              disabled={!session.canEdit()}
              radius-master
              label="Radius master"
              on:input={shape.onMasterInput}
            />
          </column>
          <p>
            style(fieldHint)
Outline corner chips. Master sets mock ramp sm/md/lg/xl = 0.5/1/1.5/2 × md.</p>
        </column>

        <!-- Shadow / elevation -->
        <column section="shadow" gap="3">
          style(sectionPanel)

          <row align="baseline" justify="between" gap="2" nowrap>
            <p>
              style(sectionTitle)
Shadow steps</p>
            <p>
              style(pathMeta)
elevation · {shape.elevLabel()}</p>
          </row>
          <grid cols="3" gap="2" elev-chips on:click={shape.onElevClick} label="Elevation steps">
            <button
              type="button"
              size="sm"
              elev="none"
              variant="outline"
              tone={shape.elevKey() === 'none' ? 'primary' : 'muted'}
              pressed={shape.elevKey() === 'none' ? 'true' : 'false'}
              disabled={!session.canEdit()}
            >
              <row gap="2" align="center" nowrap>
                <stack hidden="true">
                  style(elevMarkNone)
</stack>
                <column gap="0" align="start">
                  style({ box: minW(0) })

                  <p>
                    style(doorTitle)
none</p>
                  <p>
                    style(pathMeta)
flat</p>
                </column>
              </row>
            </button>
            <button
              type="button"
              size="sm"
              elev="xs"
              variant="outline"
              tone={shape.elevKey() === 'xs' ? 'primary' : 'muted'}
              pressed={shape.elevKey() === 'xs' ? 'true' : 'false'}
              disabled={!session.canEdit()}
            >
              <row gap="2" align="center" nowrap>
                <stack hidden="true">
                  style(elevMarkXs)
</stack>
                <column gap="0" align="start">
                  style({ box: minW(0) })

                  <p>
                    style(doorTitle)
xs</p>
                  <p>
                    style(pathMeta)
hair</p>
                </column>
              </row>
            </button>
            <button
              type="button"
              size="sm"
              elev="sm"
              variant="outline"
              tone={shape.elevKey() === 'sm' ? 'primary' : 'muted'}
              pressed={shape.elevKey() === 'sm' ? 'true' : 'false'}
              disabled={!session.canEdit()}
            >
              <row gap="2" align="center" nowrap>
                <stack hidden="true">
                  style(elevMarkSm)
</stack>
                <column gap="0" align="start">
                  style({ box: minW(0) })

                  <p>
                    style(doorTitle)
sm</p>
                  <p>
                    style(pathMeta)
rest</p>
                </column>
              </row>
            </button>
            <button
              type="button"
              size="sm"
              elev="md"
              variant="outline"
              tone={shape.elevKey() === 'md' ? 'primary' : 'muted'}
              pressed={shape.elevKey() === 'md' ? 'true' : 'false'}
              disabled={!session.canEdit()}
            >
              <row gap="2" align="center" nowrap>
                <stack hidden="true">
                  style(elevMarkMd)
</stack>
                <column gap="0" align="start">
                  style({ box: minW(0) })

                  <p>
                    style(doorTitle)
md</p>
                  <p>
                    style(pathMeta)
card</p>
                </column>
              </row>
            </button>
            <button
              type="button"
              size="sm"
              elev="lg"
              variant="outline"
              tone={shape.elevKey() === 'lg' ? 'primary' : 'muted'}
              pressed={shape.elevKey() === 'lg' ? 'true' : 'false'}
              disabled={!session.canEdit()}
            >
              <row gap="2" align="center" nowrap>
                <stack hidden="true">
                  style(elevMarkLg)
</stack>
                <column gap="0" align="start">
                  style({ box: minW(0) })

                  <p>
                    style(doorTitle)
lg</p>
                  <p>
                    style(pathMeta)
float</p>
                </column>
              </row>
            </button>
            <button
              type="button"
              size="sm"
              elev="xl"
              variant="outline"
              tone={shape.elevKey() === 'xl' ? 'primary' : 'muted'}
              pressed={shape.elevKey() === 'xl' ? 'true' : 'false'}
              disabled={!session.canEdit()}
            >
              <row gap="2" align="center" nowrap>
                <stack hidden="true">
                  style(elevMarkXl)
</stack>
                <column gap="0" align="start">
                  style({ box: minW(0) })

                  <p>
                    style(doorTitle)
xl</p>
                  <p>
                    style(pathMeta)
modal</p>
                </column>
              </row>
            </button>
          </grid>
          <p>
            style(fieldHint)
Maps to shadow.card-rest from the chosen step.</p>
        </column>

        <!-- Border strength -->
        <column section="border" gap="3">
          style(sectionPanel)

          <row align="baseline" justify="between" gap="2" nowrap>
            <p>
              style(sectionTitle)
Border strength</p>
            <p>
              style(pathMeta)
{shape.borderWLabel()}</p>
          </row>
          <column gap="1">
            style(sliderRow)

            <row align="center" justify="between" gap="2" nowrap>
              <p>
                style(fieldHint)
Width</p>
              <p>
                style(monoMeta)
{shape.borderWLabel()}</p>
            </row>
            <input
              type="range"
              min="0"
              max="3"
              step="0.5"
              value={shape.borderW()}
              disabled={!session.canEdit()}
              border-width
              label="Border width"
              on:input={shape.onBorderWidth}
            />
          </column>
          <p>
            style(fieldHint)
Sets border thickness for cards and controls.</p>
        </column>

        <!-- Glass material — gated until Glass recipe / material dials -->
        <if {shape.showGlass()}>
          <column section="glass" gap="3">
            style(sectionPanel)

            <row align="baseline" justify="between" gap="2" nowrap>
              <p>
                style(sectionTitle)
Glass material</p>
              <p>
                style(pathMeta)
card fill α · blur</p>
            </row>
            <column gap="1">
              style(sliderRow)

              <row align="center" justify="between" gap="2" nowrap>
                <p>
                  style(fieldHint)
Surface opacity</p>
                <p>
                  style(monoMeta)
{shape.surfaceOpLabel()}</p>
              </row>
              <input
                type="range"
                min="10"
                max="100"
                step="5"
                value={shape.surfaceOpPct()}
                disabled={!session.canEdit()}
                surface-opacity
                label="Surface opacity"
                on:input={shape.onSurfaceOp}
              />
            </column>
            <column gap="1">
              style(sliderRow)

              <row align="center" justify="between" gap="2" nowrap>
                <p>
                  style(fieldHint)
Backdrop blur</p>
                <p>
                  style(monoMeta)
{shape.surfaceBlurLabel()}</p>
              </row>
              <input
                type="range"
                min="0"
                max="28"
                step="1"
                value={shape.surfaceBlur()}
                disabled={!session.canEdit()}
                surface-blur
                label="Backdrop blur"
                on:input={shape.onSurfaceBlur}
              />
            </column>
            <p>
              style(fieldHint)
Writes component.card.bg α + backdrop-filter. Frost fill previews freeform; blur is theme paint after Save.</p>
          </column>
        </if>

        <column shape-footer gap="2">
          <row gap="2" align="center" nowrap>
            <p>
              style(tokenStrip)
radius.*</p>
            <p>
              style(tokenStrip)
shadow.*</p>
            <p>
              style(tokenStrip)
border-width</p>
            <p>
              style(tokenStrip)
surface</p>
          </row>
          <p>
            style(fieldHint)

            Theme-wide radius scale. Per-component exceptions live under Components.
          </p>
        </column>
      </column>
    </scroll-area>

    <!-- RIGHT: large specimen canvas (grow) -->
    <column specimen-dock shape-canvas gap="0">
      style({  box: grow(1).minW(0).basis(0).minH(0).h(full).overflow(hidden), paint: bg(background)  })

      <column gap="0">
        style({  box: grow(1).minH(0).h(full).pad(4)  })

        <!--
          One themed host only. Frame chrome = theme name + mode.
          No external Specimen/Preview/light label stack.
        -->
        <card
          shape-specimen
          preview-specimen
          mode={session.paintMode()}
          padding="none"

        >
          style({ box: minW(0).minH(0).h(full).overflow(hidden), paint: shadow(lg) })

          <column gap="0">
            style({  box: grow(1).minH(0).h(full)  })

            <row specimen-chrome align="center" gap="2" nowrap>
              style({  box: shrink(0).padX(3).h(8).minH(8).minW(0), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1)  })

              <row gap="2" align="center" nowrap hidden="true">
                style({  box: shrink(0).minW(12)  })

                <stack>
                  style({  box: shrink(0).w(2).h(2).minW(2).minH(2), paint: radius(full).bg(destructive)  })
</stack>
                <stack>
                  style({  box: shrink(0).w(2).h(2).minW(2).minH(2), paint: radius(full).bg(warning)  })
</stack>
                <stack>
                  style({  box: shrink(0).w(2).h(2).minW(2).minH(2), paint: radius(full).bg(success)  })
</stack>
              </row>
              <p>
                style({ box: grow(1).minW(0), paint: fg(muted-foreground), text: fontSize(xs).truncate() })

                {session.nameEdit()} · Shape
              </p>
              <p>
                style([ style(pathMeta), style({ box: shrink(0) }) ])
{session.paintMode()}</p>
            </row>

            <scroll-area>
              style([ style(stageSolid), style({ box: grow(1) }) ])

              <column
                specimen-body
                gap="5"
                align="center"

              >
                style({ box: pad(6).minW(0).w(full) })

                <column gap="1">
                  style({ box: maxW(panel-max-w-sm).w(full) })

                  <p>
                    style({ text: fontSize(md).fontWeight(semibold).fontFamily(heading) })
Studio surface</p>
                  <p>
                    style({ paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed).fontFamily(body) })

                    Radius + elevation + border as character. Dial left — this card follows.
                  </p>
                </column>

                <!-- Single character card: freeform fill (frost α live), not solid nested plate. -->
                <stack spec-card use:liveBox={liveCardBox()}>
                  style(liveCard)

                  <column gap="4">
                    style({ box: maxW(panel-max-w-sm).w(full) })

                    <row gap="3" align="center" nowrap>
                      <avatar size="sm" initials="S" tone="primary"></avatar>
                      <column gap="0">
                        style({ box: grow(1).minW(0) })

                        <p>
                          style({ text: fontSize(md).fontWeight(semibold).fontFamily(heading) })
{session.nameEdit()}</p>
                        <p>
                          style({ paint: fg(muted-foreground), text: fontSize(xs) })
Radius + elevation + border as character</p>
                      </column>
                      <badge tone="primary" size="sm" variant="soft" pill>shape</badge>
                    </row>
                    <column gap="1">
                      <p>
                        style(fieldHint)
Project name</p>
                      <input type="text" size="sm" value="aurora-shell" label="Sample project name" spellcheck="false" />
                    </column>
                    <row gap="2" align="center" nowrap>
                      <button type="button" size="sm" variant="solid" tone="primary">Primary</button>
                      <button type="button" size="sm" variant="soft" tone="primary">Soft</button>
                      <button type="button" size="sm" variant="outline" tone="primary">Outline</button>
                    </row>
                    <row gap="2" align="stretch">
                      <column gap="0">
                        style([ style(statCell), style({ box: grow(1).minW(0) }) ])

                        <p>
                          style(pathMeta)
radius.md</p>
                        <p>
                          style({ text: fontSize(sm).fontFamily(mono).fontWeight(semibold) })
{shape.statRadius()}</p>
                      </column>
                      <column gap="0">
                        style([ style(statCell), style({ box: grow(1).minW(0) }) ])

                        <p>
                          style(pathMeta)
shadow</p>
                        <p>
                          style({ text: fontSize(sm).fontFamily(mono).fontWeight(semibold) })
{shape.statShadow()}</p>
                      </column>
                      <column gap="0">
                        style([ style(statCell), style({ box: grow(1).minW(0) }) ])

                        <p>
                          style(pathMeta)
border</p>
                        <p>
                          style({ text: fontSize(sm).fontFamily(mono).fontWeight(semibold) })
{shape.statBorder()}</p>
                      </column>
                    </row>
                    <row align="center" justify="between" gap="2" nowrap>
                      <p>
                        style(pathMeta)
{shape.statRecipe()}</p>
                      <button type="button" size="sm" variant="ghost" tone="muted">Details</button>
                    </row>
                  </column>
                </stack>

                <if {shape.showGlass()}>
                  <column glass-mini gap="2" use:liveBox={liveGlassBox()}>
                    style(glassMini)

                    <p>
                      style(pathMeta)
Glass material</p>
                    <p>
                      style({ text: fontSize(sm).fontWeight(semibold).fontFamily(heading) })
Frosted pane</p>
                    <p>
                      style({ paint: fg(muted-foreground), text: fontSize(xs).leading(relaxed) })

                      Translucent fill freeform live. Backdrop blur writes to theme; shows after Save.
                    </p>
                    <row gap="3" align="center" nowrap>
                      <p>
                        style(pathMeta)
blur {shape.surfaceBlurLabel()}</p>
                      <p>
                        style(pathMeta)
α {shape.surfaceOpLabel()}</p>
                    </row>
                  </column>
                </if>

                <column radius-ladder gap="2" align="center" hidden="true">
                  <p>
                    style(fieldHint)
Radius ladder</p>
                  <row gap="4" align="end" nowrap>
                    <column gap="1" align="center">
                      <stack use:liveBox={liveLadderSmBox()}>
                        style(ladderChip)
</stack>
                      <p>
                        style(pathMeta)
sm</p>
                    </column>
                    <column gap="1" align="center">
                      <stack use:liveBox={liveLadderMdBox()}>
                        style(ladderChip)
</stack>
                      <p>
                        style(pathMeta)
md</p>
                    </column>
                    <column gap="1" align="center">
                      <stack use:liveBox={liveLadderLgBox()}>
                        style(ladderChip)
</stack>
                      <p>
                        style(pathMeta)
lg</p>
                    </column>
                    <column gap="1" align="center">
                      <stack use:liveBox={liveLadderXlBox()}>
                        style(ladderChipXl)
</stack>
                      <p>
                        style(pathMeta)
xl</p>
                    </column>
                  </row>
                </column>

                <row align="center" justify="between" gap="2" nowrap>
                  style({ box: maxW(panel-max-w-md).w(full) })

                  <p>
                    style(fieldHint)
Globals only — dial once, surfaces follow.</p>
                  <p>
                    style(pathMeta)
{shape.liveCssLine()}</p>
                </row>
              </column>
            </scroll-area>
          </column>
        </card>
      </column>
    </column>
  </row>

  <toast-region position="bottom-end" shape-toast-region>
    <for {session.toastItems() as t} key={t.key}>
      <toast tone={t.tone} shape-toast on-dismiss={session.onToastDismiss}>
        {t.message}
      </toast>
    </for>
  </toast-region>
</column>
