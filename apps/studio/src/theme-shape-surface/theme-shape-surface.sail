<!--
  Theme shape facet — specimen-first character dials (mock SSOT: shape.html).
  Controls ~panel | large stage. Outline radius chips; elev / border / glass.
  Live geometry via freeform px/shadow bind (not CSS-var product state).
-->
<scope>
  style sectionTitle = { type({ size: sm, weight: semibold }) }
  style fieldHint = { fg(muted-foreground).type({ size: xs }) }
  style pathMeta = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style monoMeta = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style doorTitle = { type({ weight: semibold }) }
  style eyebrow = { fg(muted-foreground).type({ size: xs, weight: semibold, tracking: wide, case: uppercase }) }
  style controlsCol = { pad({ x: 4, t: 4, b: 8 }).size({ min-w: 0 }) }
  style bandStatus = { pad({ x: 4, y: 2 }) }
  style bandError = { pad({ x: 4, y: 2 }).border({ edge: b, color: border, width: 1 }) }
  style fullBtn = { size({ w: full }) }
  style chipCol = { size({ min-w: 0 }) }
  style sliderRow = { size({ min-w: 0 }) }
  style sectionPanel = { pad(3).radius(lg).border({ color: border-subtle, width: 1 }).bg(surface-1) }
  style tokenStrip = { pad({ x: 2, y: 1 }).fg(muted-foreground).radius(sm).bg(surface-0).type({ family: mono, size: xs }) }
  // Outline chip shells — continuous radius via use:liveBox (token radius cannot drag continuously).
  style ladderChip = { size({ w: 9, h: 7, min-w: 9, min-h: 7 }).border({ color: border, width: 1 }).bg(transparent) }
  style ladderChipXl = { size({ w: 10, h: 8, min-w: 10, min-h: 8 }).border({ color: border, width: 1 }).bg(transparent) }
  style cornerChip = { size({ w: 8, h: 6, min-w: 8, min-h: 6 }).border({ color: border, width: 1 }).bg(transparent) }
  // Elev chip marks — stepped vocabulary (none dashed → xl deep lift).
  style elevMarkNone = { size({ w: 5, h: 4, min-w: 5, min-h: 4 }).radius(sm).border({ color: border-subtle, width: 1, style: dashed }).bg(background) }
  style elevMarkXs = { size({ w: 5, h: 4, min-w: 5, min-h: 4 }).radius(sm).border({ color: border-subtle, width: 1 }).bg(background).elevation(xs) }
  style elevMarkSm = { size({ w: 5, h: 4, min-w: 5, min-h: 4 }).radius(sm).border({ color: border-subtle, width: 1 }).bg(background).elevation(sm) }
  style elevMarkMd = { size({ w: 5, h: 4, min-w: 5, min-h: 4 }).radius(sm).border({ color: border-subtle, width: 1 }).bg(background).elevation(md) }
  style elevMarkLg = { size({ w: 5, h: 4, min-w: 5, min-h: 4 }).radius(sm).border({ color: border-subtle, width: 1 }).bg(background).elevation(lg) }
  style elevMarkXl = { size({ w: 5, h: 4, min-w: 5, min-h: 4 }).radius(sm).border({ color: border-subtle, width: 1 }).bg(background).elevation(xl) }
  style statCell = { pad(2_5).size({ min-w: 0 }).radius(sm).border({ color: border-subtle, width: 1 }) }
  style stageSolid = { size({ h: full, min-h: 0 }).bg(background) }
  // Primary specimen card: radius.lg + elev + border + freeform fill (frost α live).
  style liveCard = { pad(5).border({ color: border }) }
  style glassMini = { size({ w: full, max-w: panel-max-w-sm }).pad(4).border({ color: border }) }
  style surfaceRoot = { grow(1).size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }).bg(background) }
  style workbench = { grow(1).size({h: full, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }).bg(background) }
  style panelRail = { shrink(0).size({ w: panel-max-w-sm, h: full, min-w: panel-min-w-sm, min-h: 0, max-w: panel-max-w-sm }).border({ edge: r, color: border-subtle, width: 1 }).bg(background) }
  style specimenDock = { grow(1).size({h: full, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }).bg(background) }
  style specimenPad = { grow(1).size({ h: full, min-h: 0 }).pad(4) }
  style specimenFrame = { size({ h: full, min-w: 0, min-h: 0 }).overflow({ all: hidden }).elevation(lg) }
  style specimenChrome = { shrink(0).pad({ x: 3 }).size({ h: 8, min-w: 0, min-h: 8 }).border({ edge: b, color: border-subtle, width: 1 }).bg(surface-1) }
  style trafficDots = { shrink(0).size({ min-w: 12 }) }
  style trafficDot = { shrink(0).size({ w: 2, h: 2, min-w: 2, min-h: 2 }).radius(full) }
  style chromeTitle = { grow(1).size({ min-w: 0 }).fg(muted-foreground).type({ size: xs, wrap: truncate }) }
  style errorText = { grow(1).size({ min-w: 0 }).fg(destructive).type({ size: sm, wrap: truncate }) }
  style pageTitle = { type({ family: heading, size: 2xl, weight: semibold, tracking: tight }) }
  style growMin = { grow(1).size({ min-w: 0 }) }
  style elevHintCol = { size({ min-w: 0 }) }
  style chipLabel = { type({ wrap: truncate }) }
  type ElevChipProps = {
    elev: string
    label: string
    hint: string
    on: boolean
    canEdit: boolean
    onPick: any
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
    on:click={onPick}
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
    <row shape-error align="center" gap="2" style(bandError, { shrink(0) })>
      <p style(errorText)>{session.errorText()}</p>
      <button type="button" size="sm" variant="ghost" shape-retry on:click={session.onRetry}>
        Retry
      </button>
    </row>
  </if>

  <if {session.loading()}>
    <row shape-loading align="center" gap="2" style(bandStatus, { shrink(0) })>
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
          <grid cols="3" gap="2" recipe-grid>
            <for {shape.recipes() as r} key={r.key}>
              <button
                type="button"
                size="sm"
                recipe={r.id}
                variant={r.on ? 'solid' : 'outline'}
                tone={r.on ? 'primary' : 'muted'}
                pressed={r.on ? 'true' : 'false'}
                disabled={!session.canEdit()}
                on:click={() => shape.onRecipeClick(r.id)}
                style(fullBtn)>
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
          <row radius-chips gap="2" align="stretch" nowrap label="Radius steps">
            <button
              type="button"
              size="sm"
              radius="sm"
              variant="outline"
              tone={shape.focusStep() === 'sm' ? 'primary' : 'muted'}
              pressed={shape.focusStep() === 'sm' ? 'true' : 'false'}
              title="radius.sm"
              on:click={() => shape.onRadiusChip('sm')}
              style(chipCol)>
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
              title="radius.md"
              on:click={() => shape.onRadiusChip('md')}
              style(chipCol)>
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
              title="radius.lg"
              on:click={() => shape.onRadiusChip('lg')}
              style(chipCol)>
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
              title="radius.xl"
              on:click={() => shape.onRadiusChip('xl')}
              style(chipCol)>
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
            <slider
       
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
          <grid cols="3" gap="2" elev-chips label="Elevation steps">
            <for {shape.elevChips() as c} key={c.key}>
              <elev-chip
                elev={c.elev}
                label={c.label}
                hint={c.hint}
                on={c.on}
                canEdit={session.canEdit()}
                onPick={() => shape.onElevClick(c.elev)}
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
            <slider
       
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
              <slider
        
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
              <slider
        
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
            <p style(fieldHint)>Writes card fill α + backdrop. Frost fill previews freeform; blur is theme paint after Save.</p>
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
          <column gap="0" style({ grow(1).size({ h: full, min-h: 0 }) })>
            <row preview-chrome align="center" gap="2" nowrap style(specimenChrome)>
              <row gap="2" align="center" nowrap hidden="true" style(trafficDots)>
                <stack style(trafficDot, { bg(destructive) })></stack>
                <stack style(trafficDot, { bg(warning) })></stack>
                <stack style(trafficDot, { bg(success) })></stack>
              </row>
              <p style(chromeTitle)>
                Preview · {session.nameEdit()} · Shape
              </p>
              <p style(pathMeta, { shrink(0) })>{session.paintMode()}</p>
            </row>

            <scroll-area style(stageSolid, { grow(1) })>
              <column
                specimen-body
                gap="5"
                align="center" style({ pad(6).size({ w: full, min-w: 0 }) })>
                <column gap="1" style({ size({ w: full, max-w: panel-max-w-sm }) })>
                  <p style({ type({ family: heading, size: md, weight: semibold }) })>Studio surface</p>
                  <p style({ fg(muted-foreground).type({ family: body, size: sm, leading: relaxed }) })>
                    Radius + elevation + border as character. Dial left — this card follows.
                  </p>
                </column>

                <!-- Single character card: freeform fill (frost α live), not solid nested plate. -->
                <stack spec-card use:liveBox={liveCardBox()} style(liveCard)>
                  <column gap="4" style({ size({ w: full, max-w: panel-max-w-sm }) })>
                    <row gap="3" align="center" nowrap>
                      <avatar size="sm" initials="S" tone="primary"></avatar>
                      <column gap="0" style({ grow(1).size({ min-w: 0 }) })>
                        <p style({ type({ family: heading, size: md, weight: semibold }) })>{session.nameEdit()}</p>
                        <p style({ fg(muted-foreground).type({ size: xs }) })>Radius + elevation + border as character</p>
                      </column>
                      <badge tone="primary" size="sm" variant="soft" pill style(chipLabel)>shape</badge>
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
                      <column gap="0" style(statCell, { grow(1).size({ min-w: 0 }) })>
                        <p style(pathMeta)>radius.md</p>
                        <p style({ type({ family: mono, size: sm, weight: semibold }) })>{shape.statRadius()}</p>
                      </column>
                      <column gap="0" style(statCell, { grow(1).size({ min-w: 0 }) })>
                        <p style(pathMeta)>shadow</p>
                        <p style({ type({ family: mono, size: sm, weight: semibold }) })>{shape.statShadow()}</p>
                      </column>
                      <column gap="0" style(statCell, { grow(1).size({ min-w: 0 }) })>
                        <p style(pathMeta)>border</p>
                        <p style({ type({ family: mono, size: sm, weight: semibold }) })>{shape.statBorder()}</p>
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
                    <p style({ type({ family: heading, size: sm, weight: semibold }) })>Frosted pane</p>
                    <p style({ fg(muted-foreground).type({ size: xs, leading: relaxed }) })>
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

                <row align="center" justify="between" gap="2" nowrap style({ size({ w: full, max-w: panel-max-w-md }) })>
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
