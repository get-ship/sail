<!--
  Theme shape facet — specimen-first character dials (mock SSOT: shape.html).
  Controls ~panel | large stage. Outline radius chips; elev / border / glass.
  Live geometry via freeform px/shadow bind (not CSS-var product state).
-->

<scope>
  style sectionTitle = { type(body-sm) }
  style fieldHint = { fg(muted).type(body-sm) }
  style pathMeta = { fg(muted).type(mono-sm) }
  style monoMeta = { fg(muted).type(mono-sm) }
  // no preset: type weight
  style doorTitle = { type({ weight: semibold }) }
  style eyebrow = { fg(muted).type(label) }
  // no preset: min-w lock
  style controlsCol = { pad(4).size({ min-w: 0 }) }
  style bandStatus = { pad(4) }
  // no preset: edge-only border
  style bandError = { pad(4).border({ edge: b, color: border, width: 1 }) }
  style fullBtn = { size(max) }
  // no preset: min-w lock
  style chipCol = { size({ min-w: 0 }) }
  // no preset: min-w lock
  style sliderRow = { size({ min-w: 0 }) }
  style sectionPanel = { pad(3).radius(lg).border(subtle).bg(surface) }
  style tokenStrip = { pad(2).fg(muted).radius(sm).bg(app).type(mono-sm) }
  // Outline chip shells — continuous radius via use:liveBox (token radius cannot drag continuously).
  // no preset: size lock
  style ladderChip = { size({ w: 9, h: 7, min-w: 9, min-h: 7 }).border(subtle).bg(transparent) }
  // no preset: size lock
  style ladderChipXl = { size({ w: 10, h: 8, min-w: 10, min-h: 8 }).border(subtle).bg(transparent) }
  // no preset: size lock
  style cornerChip = { size({ w: 8, h: 6, min-w: 8, min-h: 6 }).border(subtle).bg(transparent) }
  // Elev chip marks — stepped vocabulary (none dashed → xl deep lift).
  // no preset: size lock
  style elevMarkNone = { size({ w: 5, h: 4, min-w: 5, min-h: 4 }).radius(sm).border(subtle).bg(background) }
  // no preset: size lock
  style elevMarkXs = { size({ w: 5, h: 4, min-w: 5, min-h: 4 }).radius(sm).border(subtle).bg(background).elevation(xs) }
  // no preset: size lock
  style elevMarkSm = { size({ w: 5, h: 4, min-w: 5, min-h: 4 }).radius(sm).border(subtle).bg(background).elevation(sm) }
  // no preset: size lock
  style elevMarkMd = { size({ w: 5, h: 4, min-w: 5, min-h: 4 }).radius(sm).border(subtle).bg(background).elevation(md) }
  // no preset: size lock
  style elevMarkLg = { size({ w: 5, h: 4, min-w: 5, min-h: 4 }).radius(sm).border(subtle).bg(background).elevation(lg) }
  // no preset: size lock
  style elevMarkXl = { size({ w: 5, h: 4, min-w: 5, min-h: 4 }).radius(sm).border(subtle).bg(background).elevation(xl) }
  // no preset: min-w lock
  style statCell = { pad(2_5).size({ min-w: 0 }).radius(sm).border(subtle) }
  // no preset: size lock
  style stageSolid = { size({ h: 100%, min-h: 0 }).bg(background) }
  // Primary specimen card: radius.lg + elev + border + freeform fill (frost α live).
  style liveCard = { pad(5).border(subtle) }
  // no preset: size lock
  style glassMini = { size({ w: 100%, max-w: overlay-max-w-sm }).pad(4).border(subtle) }
  style surfaceRoot = { size(grow).overflow(hidden).bg(background) }
  style workbench = { size(grow).overflow(hidden).bg(background) }
  // no preset: panel width lock + edge-only border
  style panelRail = { size({ w: overlay-max-w-sm, h: 100%, min-h: 0, max-w: overlay-max-w-sm }).border({ edge: r, color: border-subtle, width: 1 }).bg(background) }
  style specimenDock = { size(grow).overflow(hidden).bg(background) }
  style specimenPad = { size(grow).pad(4) }
  // no preset: min-w lock
  style specimenFrame = { size({ h: 100%, min-w: 0, min-h: 0 }).overflow(hidden).elevation(lg) }
  // no preset: min-w lock + edge-only border
  style specimenChrome = { pad(3).size({ h: 8, min-w: 0, min-h: 8 }).border({ edge: b, color: border-subtle, width: 1 }).bg(surface) }
  // no preset: size lock
  style trafficDots = { size({ min-w: 12 }) }
  // no preset: size lock
  style trafficDot = { size({ w: 2, h: 2, min-w: 2, min-h: 2 }).radius(full) }
  style chromeTitle = { size(grow).fg(muted).type(body-sm) }
  style errorText = { size(grow).fg(danger).type(body-sm) }
  style pageTitle = { type(heading) }
  style growMin = { size(grow) }
  // no preset: min-w lock
  style elevHintCol = { size({ min-w: 0 }) }
  type ElevChipProps = {
    elev: string
    label: string
    hint: string
    on: boolean
    canEdit: boolean
    onPick: any
  }
  style shrink0 = { size(hug) }
</scope>

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

<column surface="themes" theme-shape-surface shape-chrome gap="0" style(surfaceRoot)>
  <theme-facet-chrome session={session}></theme-facet-chrome>

  <if {session.hasError()}>
    <row shape-error align="center" gap="2" style(bandError, shrink0)>
      <p style(errorText)>{session.errorText()}</p>
      <button type="button" size="sm" variant="ghost" shape-retry on:click={session.onRetry}>
        Retry
      </button>
    </row>
  </if>

  <if {session.loading()}>
    <row shape-loading align="center" gap="2" style(bandStatus, shrink0)>
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
          <column gap="0" style(growMin)>
            <row preview-chrome align="center" gap="2" nowrap style(specimenChrome)>
              <row gap="2" align="center" nowrap hidden="true" style(trafficDots)>
                <stack style(trafficDot, { bg(danger) })></stack>
                <stack style(trafficDot, { bg(warning) })></stack>
                <stack style(trafficDot, { bg(success) })></stack>
              </row>
              <p style(chromeTitle)>
                Preview · {session.nameEdit()} · Shape
              </p>
              <p style(pathMeta, shrink0)>{session.paintMode()}</p>
            </row>

            <scroll-area style(stageSolid, growMin)>
              <!-- no preset: min-w lock -->
              <column
                specimen-body
                gap="5"
                align="center" style({ pad(6).size({ w: 100%, min-w: 0 }) })>
                <!-- no preset: size lock -->
                <column gap="1" style({ size({ w: 100%, max-w: overlay-max-w-sm }) })>
                  <p style({ type(heading) })>Studio surface</p>
                  <p style({ fg(muted).type(body-sm) })>
                    Radius + elevation + border as character. Dial left — this card follows.
                  </p>
                </column>

                <!-- Single character card: freeform fill (frost α live), not solid nested plate. -->
                <stack spec-card use:liveBox={liveCardBox()} style(liveCard)>
                  <!-- no preset: size lock -->
                  <column gap="4" style({ size({ w: 100%, max-w: overlay-max-w-sm }) })>
                    <row gap="3" align="center" nowrap>
                      <avatar size="sm" initials="S" tone="primary"></avatar>
                      <column gap="0" style(growMin)>
                        <p style({ type(heading) })>{session.nameEdit()}</p>
                        <p style({ fg(muted).type(body-sm) })>Radius + elevation + border as character</p>
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
                      <column gap="0" style(statCell, growMin)>
                        <p style(pathMeta)>radius.md</p>
                        <p style({ type(mono) })>{shape.statRadius()}</p>
                      </column>
                      <column gap="0" style(statCell, growMin)>
                        <p style(pathMeta)>shadow</p>
                        <p style({ type(mono) })>{shape.statShadow()}</p>
                      </column>
                      <column gap="0" style(statCell, growMin)>
                        <p style(pathMeta)>border</p>
                        <p style({ type(mono) })>{shape.statBorder()}</p>
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
                    <p style({ type(heading) })>Frosted pane</p>
                    <p style({ fg(muted).type(body-sm) })>
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

                <!-- no preset: size lock -->
                <row align="center" justify="between" gap="2" nowrap style({ size({ w: 100%, max-w: overlay-max-w-md }) })>
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
