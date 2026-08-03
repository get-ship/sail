<!--
  Theme shape facet — specimen-first character dials (mock SSOT: shape.html).
  Controls ~panel | large stage. Outline radius chips; elev / border / glass.
  Live geometry via freeform px/shadow bind (not CSS-var product state).
-->
<scope>
  const sectionTitle = style.fontSize(sm).fontWeight(semibold)
  const fieldHint = style.fontSize(xs).fg(muted-foreground)
  const pathMeta = style.fontSize(xs).fontFamily(mono).fg(muted-foreground)
  const monoMeta = style.fontSize(xs).fontFamily(mono).fg(muted-foreground)
  const doorTitle = style.fontWeight(semibold)
  const eyebrow = style.fontSize(xs).fontWeight(semibold).tracking(wide).textCase(uppercase).fg(muted-foreground)
  const controlsCol = style.padX(4).padT(4).padB(8).minW(0)
  const bandStatus = style.padX(4).padY(2)
  const bandError = style.padX(4).padY(2).borderBWidth(1).borderB(border)
  const fullBtn = style.w(full)
  const chipCol = style.minW(0)
  const sliderRow = style.minW(0)
  const sectionPanel = style.pad(3).radius(lg).borderWidth(1).border(border-subtle).bg(surface-1)
  const tokenStrip = style.fontSize(xs).fontFamily(mono).fg(muted-foreground).padX(2).padY(1).radius(sm).bg(surface-0)
  // Outline chip shells — continuous radius via use:liveBox (token radius cannot drag continuously).
  const ladderChip = style.w(9).h(7).minW(9).minH(7).borderWidth(1).border(border).bg(transparent)
  const ladderChipXl = style.w(10).h(8).minW(10).minH(8).borderWidth(1).border(border).bg(transparent)
  const cornerChip = style.w(8).h(6).minW(8).minH(6).borderWidth(1).border(border).bg(transparent)
  // Elev chip marks — stepped vocabulary (none dashed → xl deep lift).
  const elevMarkNone = style.w(5).h(4).minW(5).minH(4).radius(sm).borderWidth(1).border(dashed).border(border-subtle).bg(background)
  const elevMarkXs = style.w(5).h(4).minW(5).minH(4).radius(sm).borderWidth(1).border(border-subtle).bg(background).shadow(xs)
  const elevMarkSm = style.w(5).h(4).minW(5).minH(4).radius(sm).borderWidth(1).border(border-subtle).bg(background).shadow(sm)
  const elevMarkMd = style.w(5).h(4).minW(5).minH(4).radius(sm).borderWidth(1).border(border-subtle).bg(background).shadow(md)
  const elevMarkLg = style.w(5).h(4).minW(5).minH(4).radius(sm).borderWidth(1).border(border-subtle).bg(background).shadow(lg)
  const elevMarkXl = style.w(5).h(4).minW(5).minH(4).radius(sm).borderWidth(1).border(border-subtle).bg(background).shadow(xl)
  const statCell = style.pad(2_5).radius(sm).borderWidth(1).border(border-subtle).minW(0)
  const stageSolid = style.minH(0).h(full).bg(background)
  // Primary specimen card: radius.lg + elev + border + freeform fill (frost α live).
  const liveCard = style.border(border).pad(5)
  const glassMini = style.maxW(panel-max-w-sm).w(full).pad(4).border(border)
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
  const packRadius = (r) => `border-radius:${r}`
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

<column
  data-surface="themes"
  data-theme-shape-surface
  data-shape-chrome
  gap="0"
  grow
  sx={style.minH(0).h(full).minW(0).overflow(hidden).bg(background)}
>
  <theme-facet-chrome session={session}></theme-facet-chrome>

  <if {session.hasError()}>
    <row data-shape-error align="center" gap="2" shrink="0" sx={bandError}>
      <p grow min-w="0" sx={style.fontSize(sm).fg(destructive).truncate()}>{session.errorText()}</p>
      <button type="button" size="sm" variant="ghost" data-shape-retry on:click={session.onRetry}>
        Retry
      </button>
    </row>
  </if>

  <if {session.loading()}>
    <row data-shape-loading align="center" gap="2" shrink="0" sx={bandStatus}>
      <spinner size="sm"></spinner>
      <p sx={fieldHint}>Loading…</p>
    </row>
  </if>

  <!-- Shape: left controls ~panel | right LARGE specimen (inverted vs Color). -->
  <row
    data-shape-workbench
    grow
    gap="0"
    nowrap
    align="stretch"
    sx={style.minH(0).basis(0).h(full).minW(0).overflow(hidden).bg(background)}
  >
    <scroll-area
      data-shape-controls
      shrink="0"
      sx={style.minW(panel-min-w-sm).maxW(panel-max-w-sm).w(panel-max-w-sm).minH(0).h(full).borderRWidth(1).borderR(border-subtle).bg(background)}
    >
      <column gap="4" sx={controlsCol}>
        <column data-shape-header gap="2">
          <p sx={eyebrow}>Globals · character</p>
          <p sx={style.fontSize(2xl).fontWeight(semibold).tracking(tight).fontFamily(heading)}>Shape</p>
          <p sx={fieldHint}>
            Corners, elevation, and material. Theme character — not per-component skins.
          </p>
          <row gap="2" align="center" nowrap>
            <badge tone="muted" size="sm" variant="outline" pill>{session.modesMeta()}</badge>
            <badge tone="muted" size="sm" variant="soft" pill>{shape.statRecipe()}</badge>
          </row>
          <if {session.catalogHint()}>
            <p data-catalog-hint sx={fieldHint}>Add to project to edit.</p>
          </if>
        </column>

        <!-- Quick patterns (recipes) -->
        <column data-section="recipes" gap="2" sx={sectionPanel}>
          <row align="baseline" justify="between" gap="2" nowrap>
            <p sx={sectionTitle}>Quick patterns</p>
            <p sx={pathMeta}>recipes</p>
          </row>
          <grid cols="3" gap="2" data-recipe-grid on:click={shape.onRecipeClick}>
            <for {shape.recipes()} as="r" key={r.key}>
              <button
                type="button"
                size="sm"
                data-recipe={r.id}
                variant={r.on ? 'solid' : 'outline'}
                tone={r.on ? 'primary' : 'muted'}
                aria-pressed={r.on ? 'true' : 'false'}
                disabled={!session.canEdit()}
                sx={fullBtn}
              >
                {r.label}
              </button>
            </for>
          </grid>
        </column>

        <!-- Radius scale — outline corner chips + master -->
        <column data-section="radius" gap="3" sx={sectionPanel}>
          <row align="baseline" justify="between" gap="2" nowrap>
            <p sx={sectionTitle}>Radius scale</p>
            <p sx={pathMeta}>{shape.radiusReadout()}</p>
          </row>
          <row data-radius-chips gap="2" align="stretch" nowrap on:click={shape.onRadiusChip} aria-label="Radius steps">
            <button
              type="button"
              size="sm"
              data-radius="sm"
              variant="outline"
              tone={shape.focusStep() === 'sm' ? 'primary' : 'muted'}
              aria-pressed={shape.focusStep() === 'sm' ? 'true' : 'false'}
              title="radius.sm"
              sx={chipCol}
            >
              <column gap="1" align="center">
                <stack aria-hidden="true" use:liveBox={liveCornerSmBox()} sx={cornerChip}></stack>
                <p sx={pathMeta}>sm</p>
                <p sx={monoMeta}>{shape.radiusSmLabel()}</p>
              </column>
            </button>
            <button
              type="button"
              size="sm"
              data-radius="md"
              variant="outline"
              tone={shape.focusStep() === 'md' ? 'primary' : 'muted'}
              aria-pressed={shape.focusStep() === 'md' ? 'true' : 'false'}
              title="radius.md"
              sx={chipCol}
            >
              <column gap="1" align="center">
                <stack aria-hidden="true" use:liveBox={liveCornerMdBox()} sx={cornerChip}></stack>
                <p sx={pathMeta}>md</p>
                <p sx={monoMeta}>{shape.radiusMdLabel()}</p>
              </column>
            </button>
            <button
              type="button"
              size="sm"
              data-radius="lg"
              variant="outline"
              tone={shape.focusStep() === 'lg' ? 'primary' : 'muted'}
              aria-pressed={shape.focusStep() === 'lg' ? 'true' : 'false'}
              title="radius.lg"
              sx={chipCol}
            >
              <column gap="1" align="center">
                <stack aria-hidden="true" use:liveBox={liveCornerLgBox()} sx={cornerChip}></stack>
                <p sx={pathMeta}>lg</p>
                <p sx={monoMeta}>{shape.radiusLgLabel()}</p>
              </column>
            </button>
            <button
              type="button"
              size="sm"
              data-radius="xl"
              variant="outline"
              tone={shape.focusStep() === 'xl' ? 'primary' : 'muted'}
              aria-pressed={shape.focusStep() === 'xl' ? 'true' : 'false'}
              title="radius.xl"
              sx={chipCol}
            >
              <column gap="1" align="center">
                <stack aria-hidden="true" use:liveBox={liveCornerXlBox()} sx={cornerChip}></stack>
                <p sx={pathMeta}>xl</p>
                <p sx={monoMeta}>{shape.radiusXlLabel()}</p>
              </column>
            </button>
          </row>
          <column data-radius-master gap="1" sx={sliderRow}>
            <row align="center" justify="between" gap="2" nowrap>
              <p sx={fieldHint}>Master · family</p>
              <p sx={monoMeta}>{shape.masterLabel()}</p>
            </row>
            <input
              type="range"
              min="0"
              max="24"
              step="1"
              value={shape.masterPx()}
              disabled={!session.canEdit()}
              data-radius-master
              aria-label="Radius master"
              on:input={shape.onMasterInput}
            />
          </column>
          <p sx={fieldHint}>Outline corner chips. Master sets mock ramp sm/md/lg/xl = 0.5/1/1.5/2 × md.</p>
        </column>

        <!-- Shadow / elevation -->
        <column data-section="shadow" gap="3" sx={sectionPanel}>
          <row align="baseline" justify="between" gap="2" nowrap>
            <p sx={sectionTitle}>Shadow steps</p>
            <p sx={pathMeta}>elevation · {shape.elevLabel()}</p>
          </row>
          <grid cols="3" gap="2" data-elev-chips on:click={shape.onElevClick} aria-label="Elevation steps">
            <button
              type="button"
              size="sm"
              data-elev="none"
              variant="outline"
              tone={shape.elevKey() === 'none' ? 'primary' : 'muted'}
              aria-pressed={shape.elevKey() === 'none' ? 'true' : 'false'}
              disabled={!session.canEdit()}
            >
              <row gap="2" align="center" nowrap>
                <stack aria-hidden="true" sx={elevMarkNone}></stack>
                <column gap="0" align="start" min-w="0">
                  <p sx={doorTitle}>none</p>
                  <p sx={pathMeta}>flat</p>
                </column>
              </row>
            </button>
            <button
              type="button"
              size="sm"
              data-elev="xs"
              variant="outline"
              tone={shape.elevKey() === 'xs' ? 'primary' : 'muted'}
              aria-pressed={shape.elevKey() === 'xs' ? 'true' : 'false'}
              disabled={!session.canEdit()}
            >
              <row gap="2" align="center" nowrap>
                <stack aria-hidden="true" sx={elevMarkXs}></stack>
                <column gap="0" align="start" min-w="0">
                  <p sx={doorTitle}>xs</p>
                  <p sx={pathMeta}>hair</p>
                </column>
              </row>
            </button>
            <button
              type="button"
              size="sm"
              data-elev="sm"
              variant="outline"
              tone={shape.elevKey() === 'sm' ? 'primary' : 'muted'}
              aria-pressed={shape.elevKey() === 'sm' ? 'true' : 'false'}
              disabled={!session.canEdit()}
            >
              <row gap="2" align="center" nowrap>
                <stack aria-hidden="true" sx={elevMarkSm}></stack>
                <column gap="0" align="start" min-w="0">
                  <p sx={doorTitle}>sm</p>
                  <p sx={pathMeta}>rest</p>
                </column>
              </row>
            </button>
            <button
              type="button"
              size="sm"
              data-elev="md"
              variant="outline"
              tone={shape.elevKey() === 'md' ? 'primary' : 'muted'}
              aria-pressed={shape.elevKey() === 'md' ? 'true' : 'false'}
              disabled={!session.canEdit()}
            >
              <row gap="2" align="center" nowrap>
                <stack aria-hidden="true" sx={elevMarkMd}></stack>
                <column gap="0" align="start" min-w="0">
                  <p sx={doorTitle}>md</p>
                  <p sx={pathMeta}>card</p>
                </column>
              </row>
            </button>
            <button
              type="button"
              size="sm"
              data-elev="lg"
              variant="outline"
              tone={shape.elevKey() === 'lg' ? 'primary' : 'muted'}
              aria-pressed={shape.elevKey() === 'lg' ? 'true' : 'false'}
              disabled={!session.canEdit()}
            >
              <row gap="2" align="center" nowrap>
                <stack aria-hidden="true" sx={elevMarkLg}></stack>
                <column gap="0" align="start" min-w="0">
                  <p sx={doorTitle}>lg</p>
                  <p sx={pathMeta}>float</p>
                </column>
              </row>
            </button>
            <button
              type="button"
              size="sm"
              data-elev="xl"
              variant="outline"
              tone={shape.elevKey() === 'xl' ? 'primary' : 'muted'}
              aria-pressed={shape.elevKey() === 'xl' ? 'true' : 'false'}
              disabled={!session.canEdit()}
            >
              <row gap="2" align="center" nowrap>
                <stack aria-hidden="true" sx={elevMarkXl}></stack>
                <column gap="0" align="start" min-w="0">
                  <p sx={doorTitle}>xl</p>
                  <p sx={pathMeta}>modal</p>
                </column>
              </row>
            </button>
          </grid>
          <p sx={fieldHint}>Maps to shadow.card-rest from the chosen step.</p>
        </column>

        <!-- Border strength -->
        <column data-section="border" gap="3" sx={sectionPanel}>
          <row align="baseline" justify="between" gap="2" nowrap>
            <p sx={sectionTitle}>Border strength</p>
            <p sx={pathMeta}>{shape.borderWLabel()}</p>
          </row>
          <column gap="1" sx={sliderRow}>
            <row align="center" justify="between" gap="2" nowrap>
              <p sx={fieldHint}>Width</p>
              <p sx={monoMeta}>{shape.borderWLabel()}</p>
            </row>
            <input
              type="range"
              min="0"
              max="3"
              step="0.5"
              value={shape.borderW()}
              disabled={!session.canEdit()}
              data-border-width
              aria-label="Border width"
              on:input={shape.onBorderWidth}
            />
          </column>
          <p sx={fieldHint}>Sets border thickness for cards and controls.</p>
        </column>

        <!-- Glass material — gated until Glass recipe / material dials -->
        <if {shape.showGlass()}>
          <column data-section="glass" gap="3" sx={sectionPanel}>
            <row align="baseline" justify="between" gap="2" nowrap>
              <p sx={sectionTitle}>Glass material</p>
              <p sx={pathMeta}>card fill α · blur</p>
            </row>
            <column gap="1" sx={sliderRow}>
              <row align="center" justify="between" gap="2" nowrap>
                <p sx={fieldHint}>Surface opacity</p>
                <p sx={monoMeta}>{shape.surfaceOpLabel()}</p>
              </row>
              <input
                type="range"
                min="10"
                max="100"
                step="5"
                value={shape.surfaceOpPct()}
                disabled={!session.canEdit()}
                data-surface-opacity
                aria-label="Surface opacity"
                on:input={shape.onSurfaceOp}
              />
            </column>
            <column gap="1" sx={sliderRow}>
              <row align="center" justify="between" gap="2" nowrap>
                <p sx={fieldHint}>Backdrop blur</p>
                <p sx={monoMeta}>{shape.surfaceBlurLabel()}</p>
              </row>
              <input
                type="range"
                min="0"
                max="28"
                step="1"
                value={shape.surfaceBlur()}
                disabled={!session.canEdit()}
                data-surface-blur
                aria-label="Backdrop blur"
                on:input={shape.onSurfaceBlur}
              />
            </column>
            <p sx={fieldHint}>Writes component.card.bg α + backdrop-filter. Frost fill previews freeform; blur is theme paint after Save.</p>
          </column>
        </if>

        <column data-shape-footer gap="2">
          <row gap="2" align="center" nowrap>
            <p sx={tokenStrip}>radius.*</p>
            <p sx={tokenStrip}>shadow.*</p>
            <p sx={tokenStrip}>border-width</p>
            <p sx={tokenStrip}>surface</p>
          </row>
          <p sx={fieldHint}>
            Theme-wide radius scale. Per-component exceptions live under Components.
          </p>
        </column>
      </column>
    </scroll-area>

    <!-- RIGHT: large specimen canvas (grow) -->
    <column
      data-specimen-dock
      data-shape-canvas
      gap="0"
      grow
      min-w="0"
      sx={style.minW(0).basis(0).minH(0).h(full).bg(background).overflow(hidden)}
    >
      <column grow gap="0" sx={style.minH(0).h(full).pad(4)}>
        <!--
          One themed host only. Frame chrome = theme name + mode.
          No external Specimen/Preview/light label stack.
        -->
        <card
          data-shape-specimen
          data-preview-specimen
          data-theme-preview={session.paintId()}
          data-mode={session.paintMode()}
          padding="none"
          sx={style.minW(0).minH(0).h(full).overflow(hidden).shadow(lg)}
        >
          <column gap="0" grow sx={style.minH(0).h(full)}>
            <row
              data-specimen-chrome
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
              <p grow min-w="0" sx={style.fontSize(xs).fg(muted-foreground).truncate()}>
                {session.nameEdit()} · Shape
              </p>
              <p sx={pathMeta} shrink="0">{session.paintMode()}</p>
            </row>

            <scroll-area grow sx={stageSolid}>
              <column
                data-specimen-body
                gap="5"
                align="center"
                sx={style.pad(6).minW(0).w(full)}
              >
                <column gap="1" sx={style.maxW(panel-max-w-sm).w(full)}>
                  <p sx={style.fontSize(md).fontWeight(semibold).fontFamily(heading)}>Studio surface</p>
                  <p sx={style.fontSize(sm).fg(muted-foreground).leading(relaxed).fontFamily(body)}>
                    Radius + elevation + border as character. Dial left — this card follows.
                  </p>
                </column>

                <!-- Single character card: freeform fill (frost α live), not solid nested plate. -->
                <stack data-spec-card use:liveBox={liveCardBox()} sx={liveCard}>
                  <column gap="4" sx={style.maxW(panel-max-w-sm).w(full)}>
                    <row gap="3" align="center" nowrap>
                      <avatar size="sm" initials="S" tone="primary"></avatar>
                      <column gap="0" grow min-w="0">
                        <p sx={style.fontSize(md).fontWeight(semibold).fontFamily(heading)}>{session.nameEdit()}</p>
                        <p sx={style.fontSize(xs).fg(muted-foreground)}>Radius + elevation + border as character</p>
                      </column>
                      <badge tone="primary" size="sm" variant="soft" pill>shape</badge>
                    </row>
                    <column gap="1">
                      <p sx={fieldHint}>Project name</p>
                      <input type="text" size="sm" value="aurora-shell" aria-label="Sample project name" spellcheck="false" />
                    </column>
                    <row gap="2" align="center" nowrap>
                      <button type="button" size="sm" variant="solid" tone="primary">Primary</button>
                      <button type="button" size="sm" variant="soft" tone="primary">Soft</button>
                      <button type="button" size="sm" variant="outline" tone="primary">Outline</button>
                    </row>
                    <row gap="2" align="stretch">
                      <column gap="0" grow min-w="0" sx={statCell}>
                        <p sx={pathMeta}>radius.md</p>
                        <p sx={style.fontSize(sm).fontFamily(mono).fontWeight(semibold)}>{shape.statRadius()}</p>
                      </column>
                      <column gap="0" grow min-w="0" sx={statCell}>
                        <p sx={pathMeta}>shadow</p>
                        <p sx={style.fontSize(sm).fontFamily(mono).fontWeight(semibold)}>{shape.statShadow()}</p>
                      </column>
                      <column gap="0" grow min-w="0" sx={statCell}>
                        <p sx={pathMeta}>border</p>
                        <p sx={style.fontSize(sm).fontFamily(mono).fontWeight(semibold)}>{shape.statBorder()}</p>
                      </column>
                    </row>
                    <row align="center" justify="between" gap="2" nowrap>
                      <p sx={pathMeta}>{shape.statRecipe()}</p>
                      <button type="button" size="sm" variant="ghost" tone="muted">Details</button>
                    </row>
                  </column>
                </stack>

                <if {shape.showGlass()}>
                  <column data-glass-mini gap="2" use:liveBox={liveGlassBox()} sx={glassMini}>
                    <p sx={pathMeta}>Glass material</p>
                    <p sx={style.fontSize(sm).fontWeight(semibold).fontFamily(heading)}>Frosted pane</p>
                    <p sx={style.fontSize(xs).fg(muted-foreground).leading(relaxed)}>
                      Translucent fill freeform live. Backdrop blur writes to theme; shows after Save.
                    </p>
                    <row gap="3" align="center" nowrap>
                      <p sx={pathMeta}>blur {shape.surfaceBlurLabel()}</p>
                      <p sx={pathMeta}>α {shape.surfaceOpLabel()}</p>
                    </row>
                  </column>
                </if>

                <column data-radius-ladder gap="2" align="center" aria-hidden="true">
                  <p sx={fieldHint}>Radius ladder</p>
                  <row gap="4" align="end" nowrap>
                    <column gap="1" align="center">
                      <stack use:liveBox={liveLadderSmBox()} sx={ladderChip}></stack>
                      <p sx={pathMeta}>sm</p>
                    </column>
                    <column gap="1" align="center">
                      <stack use:liveBox={liveLadderMdBox()} sx={ladderChip}></stack>
                      <p sx={pathMeta}>md</p>
                    </column>
                    <column gap="1" align="center">
                      <stack use:liveBox={liveLadderLgBox()} sx={ladderChip}></stack>
                      <p sx={pathMeta}>lg</p>
                    </column>
                    <column gap="1" align="center">
                      <stack use:liveBox={liveLadderXlBox()} sx={ladderChipXl}></stack>
                      <p sx={pathMeta}>xl</p>
                    </column>
                  </row>
                </column>

                <row align="center" justify="between" gap="2" nowrap sx={style.maxW(panel-max-w-md).w(full)}>
                  <p sx={fieldHint}>Globals only — dial once, surfaces follow.</p>
                  <p sx={pathMeta}>{shape.liveCssLine()}</p>
                </row>
              </column>
            </scroll-area>
          </column>
        </card>
      </column>
    </column>
  </row>

  <toast-region position="bottom-end" data-shape-toast-region>
    <for {session.toastItems()} as="t" key={t.key}>
      <toast tone={t.tone} data-shape-toast on:sail-toast-dismiss={session.onToastDismiss}>
        {t.message}
      </toast>
    </for>
  </toast-region>
</column>
