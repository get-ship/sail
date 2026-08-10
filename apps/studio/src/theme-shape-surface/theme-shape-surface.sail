<!--
  Theme shape facet — specimen-first character dials (mock SSOT: shape.html).
  Controls ~panel | large stage. Outline radius chips; elev / border / glass.
  Live geometry via freeform px/shadow bind (not CSS-var product state).
-->
<scope>
  style sectionTitle = { text: fontSize(sm).fontWeight(semibold) }
  style fieldHint = { paint: fg(muted-foreground), text: fontSize(xs) }
  style pathMeta = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style monoMeta = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style doorTitle = { text: fontWeight(semibold) }
  style eyebrow = { paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).tracking(wide).textCase(uppercase) }
  style controlsCol = { box: padX(4).padT(4).padB(8).minW(0) }
  style bandStatus = { box: padX(4).padY(2) }
  style bandError = { box: padX(4).padY(2), paint: borderBWidth(1).borderB(border) }
  style fullBtn = { box: w(full) }
  style chipCol = { box: minW(0) }
  style sliderRow = { box: minW(0) }
  style sectionPanel = { box: pad(3), paint: radius(lg).borderWidth(1).border(border-subtle).bg(surface-1) }
  style tokenStrip = { box: padX(2).padY(1), paint: fg(muted-foreground).radius(sm).bg(surface-0), text: fontSize(xs).fontFamily(mono) }
  // Outline chip shells — continuous radius via use:liveBox (token radius cannot drag continuously).
  style ladderChip = { box: w(9).h(7).minW(9).minH(7), paint: borderWidth(1).border(border).bg(transparent) }
  style ladderChipXl = { box: w(10).h(8).minW(10).minH(8), paint: borderWidth(1).border(border).bg(transparent) }
  style cornerChip = { box: w(8).h(6).minW(8).minH(6), paint: borderWidth(1).border(border).bg(transparent) }
  // Elev chip marks — stepped vocabulary (none dashed → xl deep lift).
  style elevMarkNone = { box: w(5).h(4).minW(5).minH(4), paint: radius(sm).borderWidth(1).border(dashed).border(border-subtle).bg(background) }
  style elevMarkXs = { box: w(5).h(4).minW(5).minH(4), paint: radius(sm).borderWidth(1).border(border-subtle).bg(background).shadow(xs) }
  style elevMarkSm = { box: w(5).h(4).minW(5).minH(4), paint: radius(sm).borderWidth(1).border(border-subtle).bg(background).shadow(sm) }
  style elevMarkMd = { box: w(5).h(4).minW(5).minH(4), paint: radius(sm).borderWidth(1).border(border-subtle).bg(background).shadow(md) }
  style elevMarkLg = { box: w(5).h(4).minW(5).minH(4), paint: radius(sm).borderWidth(1).border(border-subtle).bg(background).shadow(lg) }
  style elevMarkXl = { box: w(5).h(4).minW(5).minH(4), paint: radius(sm).borderWidth(1).border(border-subtle).bg(background).shadow(xl) }
  style statCell = { box: pad(2_5).minW(0), paint: radius(sm).borderWidth(1).border(border-subtle) }
  style stageSolid = { box: minH(0).h(full), paint: bg(background) }
  // Primary specimen card: radius.lg + elev + border + freeform fill (frost α live).
  style liveCard = { box: pad(5), paint: border(border) }
  style glassMini = { box: maxW(panel-max-w-sm).w(full).pad(4), paint: border(border) }
  style surfaceRoot = { box: grow(1).minH(0).h(full).minW(0).overflow(hidden), paint: bg(background) }
  style workbench = { box: grow(1).minH(0).basis(0).h(full).minW(0).overflow(hidden), paint: bg(background) }
  style panelRail = { box: shrink(0).minW(panel-min-w-sm).maxW(panel-max-w-sm).w(panel-max-w-sm).minH(0).h(full), paint: borderRWidth(1).borderR(border-subtle).bg(background) }
  style specimenDock = { box: grow(1).minW(0).basis(0).minH(0).h(full).overflow(hidden), paint: bg(background) }
  style specimenPad = { box: grow(1).minH(0).h(full).pad(4) }
  style specimenFrame = { box: minW(0).minH(0).h(full).overflow(hidden), paint: shadow(lg) }
  style specimenChrome = { box: shrink(0).padX(3).h(8).minH(8).minW(0), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1) }
  style trafficDots = { box: shrink(0).minW(12) }
  style trafficDot = { box: shrink(0).w(2).h(2).minW(2).minH(2), paint: radius(full) }
  style chromeTitle = { box: grow(1).minW(0), paint: fg(muted-foreground), text: fontSize(xs).truncate() }
  style errorText = { box: grow(1).minW(0), paint: fg(destructive), text: fontSize(sm).truncate() }
  style pageTitle = { text: fontSize(2xl).fontWeight(semibold).tracking(tight).fontFamily(heading) }
  style growMin = { box: grow(1).minW(0) }
  style elevHintCol = { box: minW(0) }
  type ElevChipProps = {
    elev: string
    label: string
    hint: string
    on: boolean
    canEdit: boolean
  }
</scope>

<partial elev-chip props: ElevChipProps>
  <button
    type="button"
    size="sm"
    elev={elev}
    variant="outline"
    tone={on ? 'primary' : 'muted'}
    pressed={on ? 'true' : 'false'}
    disabled={!canEdit}
  >
    <row gap="2" align="center" nowrap>
      <if {elev === 'none'}><stack hidden="true" style(elevMarkNone)></stack></if>
      <if {elev === 'xs'}><stack hidden="true" style(elevMarkXs)></stack></if>
      <if {elev === 'sm'}><stack hidden="true" style(elevMarkSm)></stack></if>
      <if {elev === 'md'}><stack hidden="true" style(elevMarkMd)></stack></if>
      <if {elev === 'lg'}><stack hidden="true" style(elevMarkLg)></stack></if>
      <if {elev === 'xl'}><stack hidden="true" style(elevMarkXl)></stack></if>
      <column gap="0" align="start" style(elevHintCol)>
        <p style(doorTitle)>{label}</p>
        <p style(pathMeta)>{hint}</p>
      </column>
    </row>
  </button>
</partial>

<controller>
import { createThemeSession } from '../compositions/theme-session/theme-session'
import { watchPreviewHost } from '../compositions/theme-session/bind-preview-host'
import { createShapeEdit } from '../compositions/theme-shape-surface/theme-shape-edit'
export default controller(() => {
  const session = createThemeSession({
    facet: 'shape',
    editGuardMessage: 'Add this theme to the project to edit shape.',
    saveSuccessMessage: 'Shape tokens saved to DTCG source.',
  })
  const shape = createShapeEdit(session)
  mount((host) => watchPreviewHost(host, (el) => session.attachPreviewHost(el)))
  return {
    session,
    shape,
    liveCardBox: shape.liveCardBox,
    liveGlassBox: shape.liveGlassBox,
    liveLadderSmBox: shape.liveLadderSmBox,
    liveLadderMdBox: shape.liveLadderMdBox,
    liveLadderLgBox: shape.liveLadderLgBox,
    liveLadderXlBox: shape.liveLadderXlBox,
    liveCornerSmBox: shape.liveCornerSmBox,
    liveCornerMdBox: shape.liveCornerMdBox,
    liveCornerLgBox: shape.liveCornerLgBox,
    liveCornerXlBox: shape.liveCornerXlBox,
  }
})
</controller>

<column surface="themes" theme-shape-surface shape-chrome gap="0" style(surfaceRoot)>
  <theme-facet-chrome session={session}></theme-facet-chrome>

  <if {session.hasError()}>
    <row shape-error align="center" gap="2" style(bandError, { box: shrink(0) })>
      <p style(errorText)>{session.errorText()}</p>
      <button type="button" size="sm" variant="ghost" shape-retry on:click={session.onRetry}>
        Retry
      </button>
    </row>
  </if>

  <if {session.loading()}>
    <row shape-loading align="center" gap="2" style(bandStatus, { box: shrink(0) })>
      <spinner size="sm"></spinner>
      <p style(fieldHint)>Loading…</p>
    </row>
  </if>

  <row shape-workbench gap="0" nowrap align="stretch" style(workbench)>
    <scroll-area shape-controls style(panelRail)>
      <column gap="4" style(controlsCol)>
        <column shape-header gap="2">
          <p style(eyebrow)>Globals · character</p>
          <p style(pageTitle)>Shape</p>
          <p style(fieldHint)>
            Corners, elevation, and material. Theme character — not per-component skins.
          </p>
          <row gap="2" align="center" nowrap>
            <badge tone="muted" size="sm" variant="outline" pill>{session.modesMeta()}</badge>
            <badge tone="muted" size="sm" variant="soft" pill>{shape.statRecipe()}</badge>
          </row>
          <if {session.catalogHint()}>
            <p catalog-hint style(fieldHint)>Add to project to edit.</p>
          </if>
        </column>

        <!-- Quick patterns (recipes) -->
        <column section="recipes" gap="2" style(sectionPanel)>
          <row align="baseline" justify="between" gap="2" nowrap>
            <p style(sectionTitle)>Quick patterns</p>
            <p style(pathMeta)>recipes</p>
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
                disabled={!session.canEdit()} style(fullBtn)>
                {r.label}
              </button>
            </for>
          </grid>
        </column>

        <!-- Radius scale — outline corner chips + master -->
        <column section="radius" gap="3" style(sectionPanel)>
          <row align="baseline" justify="between" gap="2" nowrap>
            <p style(sectionTitle)>Radius scale</p>
            <p style(pathMeta)>{shape.radiusReadout()}</p>
          </row>
          <row radius-chips gap="2" align="stretch" nowrap on:click={shape.onRadiusChip} label="Radius steps">
            <button
              type="button"
              size="sm"
              radius="sm"
              variant="outline"
              tone={shape.focusStep() === 'sm' ? 'primary' : 'muted'}
              pressed={shape.focusStep() === 'sm' ? 'true' : 'false'}
              title="radius.sm" style(chipCol)>
              <column gap="1" align="center">
                <stack hidden="true" use:liveBox={liveCornerSmBox()} style(cornerChip)></stack>
                <p style(pathMeta)>sm</p>
                <p style(monoMeta)>{shape.radiusSmLabel()}</p>
              </column>
            </button>
            <button
              type="button"
              size="sm"
              radius="md"
              variant="outline"
              tone={shape.focusStep() === 'md' ? 'primary' : 'muted'}
              pressed={shape.focusStep() === 'md' ? 'true' : 'false'}
              title="radius.md" style(chipCol)>
              <column gap="1" align="center">
                <stack hidden="true" use:liveBox={liveCornerMdBox()} style(cornerChip)></stack>
                <p style(pathMeta)>md</p>
                <p style(monoMeta)>{shape.radiusMdLabel()}</p>
              </column>
            </button>
            <button
              type="button"
              size="sm"
              radius="lg"
              variant="outline"
              tone={shape.focusStep() === 'lg' ? 'primary' : 'muted'}
              pressed={shape.focusStep() === 'lg' ? 'true' : 'false'}
              title="radius.lg" style(chipCol)>
              <column gap="1" align="center">
                <stack hidden="true" use:liveBox={liveCornerLgBox()} style(cornerChip)></stack>
                <p style(pathMeta)>lg</p>
                <p style(monoMeta)>{shape.radiusLgLabel()}</p>
              </column>
            </button>
            <button
              type="button"
              size="sm"
              radius="xl"
              variant="outline"
              tone={shape.focusStep() === 'xl' ? 'primary' : 'muted'}
              pressed={shape.focusStep() === 'xl' ? 'true' : 'false'}
              title="radius.xl" style(chipCol)>
              <column gap="1" align="center">
                <stack hidden="true" use:liveBox={liveCornerXlBox()} style(cornerChip)></stack>
                <p style(pathMeta)>xl</p>
                <p style(monoMeta)>{shape.radiusXlLabel()}</p>
              </column>
            </button>
          </row>
          <column radius-master gap="1" style(sliderRow)>
            <row align="center" justify="between" gap="2" nowrap>
              <p style(fieldHint)>Master · family</p>
              <p style(monoMeta)>{shape.masterLabel()}</p>
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
          <p style(fieldHint)>Outline corner chips. Master sets mock ramp sm/md/lg/xl = 0.5/1/1.5/2 × md.</p>
        </column>

        <!-- Shadow / elevation -->
        <column section="shadow" gap="3" style(sectionPanel)>
          <row align="baseline" justify="between" gap="2" nowrap>
            <p style(sectionTitle)>Shadow steps</p>
            <p style(pathMeta)>elevation · {shape.elevLabel()}</p>
          </row>
          <grid cols="3" gap="2" elev-chips on:click={shape.onElevClick} label="Elevation steps">
            <for {shape.elevChips() as c} key={c.key}>
              <elev-chip
                elev={c.elev}
                label={c.label}
                hint={c.hint}
                on={c.on}
                canEdit={session.canEdit()}
              ></elev-chip>
            </for>
          </grid>
          <p style(fieldHint)>Maps to shadow.card-rest from the chosen step.</p>
        </column>

        <!-- Border strength -->
        <column section="border" gap="3" style(sectionPanel)>
          <row align="baseline" justify="between" gap="2" nowrap>
            <p style(sectionTitle)>Border strength</p>
            <p style(pathMeta)>{shape.borderWLabel()}</p>
          </row>
          <column gap="1" style(sliderRow)>
            <row align="center" justify="between" gap="2" nowrap>
              <p style(fieldHint)>Width</p>
              <p style(monoMeta)>{shape.borderWLabel()}</p>
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
          <p style(fieldHint)>Sets border thickness for cards and controls.</p>
        </column>

        <!-- Glass material — gated until Glass recipe / material dials -->
        <if {shape.showGlass()}>
          <column section="glass" gap="3" style(sectionPanel)>
            <row align="baseline" justify="between" gap="2" nowrap>
              <p style(sectionTitle)>Glass material</p>
              <p style(pathMeta)>card fill α · blur</p>
            </row>
            <column gap="1" style(sliderRow)>
              <row align="center" justify="between" gap="2" nowrap>
                <p style(fieldHint)>Surface opacity</p>
                <p style(monoMeta)>{shape.surfaceOpLabel()}</p>
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
            <column gap="1" style(sliderRow)>
              <row align="center" justify="between" gap="2" nowrap>
                <p style(fieldHint)>Backdrop blur</p>
                <p style(monoMeta)>{shape.surfaceBlurLabel()}</p>
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
            <p style(fieldHint)>Writes component.card.bg α + backdrop-filter. Frost fill previews freeform; blur is theme paint after Save.</p>
          </column>
        </if>

        <column shape-footer gap="2">
          <row gap="2" align="center" nowrap>
            <p style(tokenStrip)>radius.*</p>
            <p style(tokenStrip)>shadow.*</p>
            <p style(tokenStrip)>border-width</p>
            <p style(tokenStrip)>surface</p>
          </row>
          <p style(fieldHint)>
            Theme-wide radius scale. Per-component exceptions live under Components.
          </p>
        </column>
      </column>
    </scroll-area>

    <column specimen-dock shape-canvas gap="0" style(specimenDock)>
      <column gap="0" style(specimenPad)>
        <card
          shape-specimen
          preview-specimen
          padding="none" style(specimenFrame)>
          <column gap="0" style({  box: grow(1).minH(0).h(full)  })>
            <row preview-chrome align="center" gap="2" nowrap style(specimenChrome)>
              <row gap="2" align="center" nowrap hidden="true" style(trafficDots)>
                <stack style(trafficDot, { paint: bg(destructive) })></stack>
                <stack style(trafficDot, { paint: bg(warning) })></stack>
                <stack style(trafficDot, { paint: bg(success) })></stack>
              </row>
              <p style(chromeTitle)>
                Preview · {session.nameEdit()} · Shape
              </p>
              <p style(pathMeta, { box: shrink(0) })>{session.paintMode()}</p>
            </row>

            <scroll-area style(stageSolid, { box: grow(1) })>
              <column
                specimen-body
                gap="5"
                align="center" style({ box: pad(6).minW(0).w(full) })>
                <column gap="1" style({ box: maxW(panel-max-w-sm).w(full) })>
                  <p style({ text: fontSize(md).fontWeight(semibold).fontFamily(heading) })>Studio surface</p>
                  <p style({ paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed).fontFamily(body) })>
                    Radius + elevation + border as character. Dial left — this card follows.
                  </p>
                </column>

                <!-- Single character card: freeform fill (frost α live), not solid nested plate. -->
                <stack spec-card use:liveBox={liveCardBox()} style(liveCard)>
                  <column gap="4" style({ box: maxW(panel-max-w-sm).w(full) })>
                    <row gap="3" align="center" nowrap>
                      <avatar size="sm" initials="S" tone="primary"></avatar>
                      <column gap="0" style({ box: grow(1).minW(0) })>
                        <p style({ text: fontSize(md).fontWeight(semibold).fontFamily(heading) })>{session.nameEdit()}</p>
                        <p style({ paint: fg(muted-foreground), text: fontSize(xs) })>Radius + elevation + border as character</p>
                      </column>
                      <badge tone="primary" size="sm" variant="soft" pill>shape</badge>
                    </row>
                    <column gap="1">
                      <p style(fieldHint)>Project name</p>
                      <input type="text" size="sm" value="aurora-shell" label="Sample project name" spellcheck="false" />
                    </column>
                    <row gap="2" align="center" nowrap>
                      <button type="button" size="sm" variant="solid" tone="primary">Primary</button>
                      <button type="button" size="sm" variant="soft" tone="primary">Soft</button>
                      <button type="button" size="sm" variant="outline" tone="primary">Outline</button>
                    </row>
                    <row gap="2" align="stretch">
                      <column gap="0" style(statCell, { box: grow(1).minW(0) })>
                        <p style(pathMeta)>radius.md</p>
                        <p style({ text: fontSize(sm).fontFamily(mono).fontWeight(semibold) })>{shape.statRadius()}</p>
                      </column>
                      <column gap="0" style(statCell, { box: grow(1).minW(0) })>
                        <p style(pathMeta)>shadow</p>
                        <p style({ text: fontSize(sm).fontFamily(mono).fontWeight(semibold) })>{shape.statShadow()}</p>
                      </column>
                      <column gap="0" style(statCell, { box: grow(1).minW(0) })>
                        <p style(pathMeta)>border</p>
                        <p style({ text: fontSize(sm).fontFamily(mono).fontWeight(semibold) })>{shape.statBorder()}</p>
                      </column>
                    </row>
                    <row align="center" justify="between" gap="2" nowrap>
                      <p style(pathMeta)>{shape.statRecipe()}</p>
                      <button type="button" size="sm" variant="ghost" tone="muted">Details</button>
                    </row>
                  </column>
                </stack>

                <if {shape.showGlass()}>
                  <column glass-mini gap="2" use:liveBox={liveGlassBox()} style(glassMini)>
                    <p style(pathMeta)>Glass material</p>
                    <p style({ text: fontSize(sm).fontWeight(semibold).fontFamily(heading) })>Frosted pane</p>
                    <p style({ paint: fg(muted-foreground), text: fontSize(xs).leading(relaxed) })>
                      Translucent fill freeform live. Backdrop blur writes to theme; shows after Save.
                    </p>
                    <row gap="3" align="center" nowrap>
                      <p style(pathMeta)>blur {shape.surfaceBlurLabel()}</p>
                      <p style(pathMeta)>α {shape.surfaceOpLabel()}</p>
                    </row>
                  </column>
                </if>

                <column radius-ladder gap="2" align="center" hidden="true">
                  <p style(fieldHint)>Radius ladder</p>
                  <row gap="4" align="end" nowrap>
                    <column gap="1" align="center">
                      <stack use:liveBox={liveLadderSmBox()} style(ladderChip)></stack>
                      <p style(pathMeta)>sm</p>
                    </column>
                    <column gap="1" align="center">
                      <stack use:liveBox={liveLadderMdBox()} style(ladderChip)></stack>
                      <p style(pathMeta)>md</p>
                    </column>
                    <column gap="1" align="center">
                      <stack use:liveBox={liveLadderLgBox()} style(ladderChip)></stack>
                      <p style(pathMeta)>lg</p>
                    </column>
                    <column gap="1" align="center">
                      <stack use:liveBox={liveLadderXlBox()} style(ladderChipXl)></stack>
                      <p style(pathMeta)>xl</p>
                    </column>
                  </row>
                </column>

                <row align="center" justify="between" gap="2" nowrap style({ box: maxW(panel-max-w-md).w(full) })>
                  <p style(fieldHint)>Globals only — dial once, surfaces follow.</p>
                  <p style(pathMeta)>{shape.liveCssLine()}</p>
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
