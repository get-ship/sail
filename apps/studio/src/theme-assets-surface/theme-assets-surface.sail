<!--
  Theme assets facet — type roles + icons deep edit.
  Thin shell: chrome + editor + specimen (draft paint via applyScope bind).
-->

<scope>
  style sectionTitle = { type(body-sm) }
  style fieldHint = { fg(muted).type(body-sm) }
  style pathMeta = { fg(muted).type(mono-sm) }
  // no preset: type weight
  style doorTitle = { type({ weight: semibold }) }
  // no preset: size lock
  style editorCol = { pad(6).size({ max-w: panel-max-w-lg }) }
  style bandStatus = { pad(4) }
  // no preset: edge-only border
  style bandError = { pad(4).border({ edge: b, color: border, width: 1 }) }
  // no preset: size lock
  style iconCell = { size({ w: 10, h: 10, min-w: 10, min-h: 10 }).radius(md).border(subtle).bg(surface-0) }
  style fullBtn = { size(max) }
  // no preset: min-w lock
  style fontHit = { pad(2_5).size({ min-w: 0 }).radius(md).border(subtle).bg(surface-0) }
  // no preset: min-w lock
  style fontSample = { size({ min-w: 0 }).fg(canvas).type(body) }
  style catChip = { when: { selected: bg(selected-bg).fg(canvas).border(subtle) } }
  style surfaceRoot = { size(grow).overflow(hidden).bg(canvas) }
  style workbench = { size(grow).overflow(hidden).bg(canvas) }
  style editorScroll = { size(grow).bg(canvas) }
  style pageTitle = { type(heading) }
  style errorText = { size(grow).fg(destructive).type(body-sm) }
  style growMin = { size(grow) }
  style shrink0 = { size(hug) }
  // no preset: min-w lock
  style catsScroll = { overflow({ x: auto }).size({ min-w: 0 }) }
  // no preset: edge-only border
  style specimenDock = { size(grow).overflow(hidden).border({ edge: l, color: border-subtle, width: 1 }).bg(canvas) }
  style specimenPad = { size(grow).pad(4) }
  style specimenFrame = { size(max).overflow(hidden).elevation(lg) }
  // no preset: min-w lock + edge-only border
  style specimenChrome = { pad(3).size({ h: 8, min-w: 0, min-h: 8 }).border({ edge: b, color: border-subtle, width: 1 }).bg(surface-1) }
  // no preset: size lock
  style trafficDots = { size({ min-w: 12 }) }
  // no preset: size lock
  style trafficDot = { size({ w: 2, h: 2, min-w: 2, min-h: 2 }).radius(full) }
  style chromeTitle = { size(grow).fg(muted).type(body-sm) }
  style scrollGrow = { size(grow).bg(canvas) }
  // no preset: min-w lock
  style stageBody = { pad(5).size({ min-w: 0 }) }
  style typeDisplay = { type(display-sm) }
  style typeBody = { type(body-sm) }
  style typeMono = { type(mono-sm) }
  type TypeRoleSelectProps = {
    token: string
    label: string
    value: string
    disabled: boolean
    options: any
    onChange: any
  }
  type FontHitProps = {
    fontFamily: string
    category: string
    sample: string
    canEdit: boolean
    onUseRole: any
    onInstall: any
  }
  style scrollCanvas = { size(grow).bg(canvas) }
</scope>

<controller>
import { createThemeSession } from '../compositions/theme-session/theme-session'
import { watchPreviewHost } from '../compositions/theme-session/bind-preview-host'
import { createAssetsEdit } from '../compositions/theme-assets-surface/theme-assets-edit'
export default controller(() => {
  const session = createThemeSession({
    facet: 'assets',
    editGuardMessage: 'Add this theme to the project to edit assets.',
    saveSuccessMessage: 'Type and icon choices saved where the store persists them.',
  })
  const assets = createAssetsEdit(session)
  mount((host) => watchPreviewHost(host, (el) => session.attachPreviewHost(el)))
  return { session, assets }
})
</controller>

<partial type-role-select props: TypeRoleSelectProps>
  <column gap="1">
    <p style(fieldHint)>{label}</p>
    <select type-token={token} size="sm" value={value} disabled={disabled} on:change={onChange}>
      <for {options as o} key={o.key}>
        <option value={o.value}>{o.label}</option>
      </for>
    </select>
  </column>
</partial>

<partial font-hit-row props: FontHitProps>
  <column gap="2" font-hit font-family={fontFamily} style(fontHit)>
    <row align="baseline" justify="between" gap="2" nowrap>
      <p style(doorTitle, growMin)>{fontFamily}</p>
      <p style(pathMeta, shrink0)>{category}</p>
    </row>
    <p font-sample face={fontFamily} style(fontSample)>{sample}</p>
    <row gap="1" align="center">
      <button type="button" size="xs" variant="outline" tone="muted" font-family={fontFamily} font-token="display" on:click={onUseRole} disabled={!canEdit}>
        Display
      </button>
      <button type="button" size="xs" variant="outline" tone="muted" font-family={fontFamily} font-token="body" on:click={onUseRole} disabled={!canEdit}>
        Body
      </button>
      <button type="button" size="xs" variant="outline" tone="muted" font-family={fontFamily} font-token="mono" on:click={onUseRole} disabled={!canEdit}>
        Mono
      </button>
      <button type="button" size="xs" variant="soft" tone="primary" font-family={fontFamily} font-token="body" on:click={onInstall} disabled={!canEdit} style(shrink0)>
        Install
      </button>
    </row>
  </column>
</partial>

<column surface="themes" theme-assets-surface assets-chrome gap="0" style(surfaceRoot)>
  <theme-facet-chrome session={session}></theme-facet-chrome>

  <if {session.hasError()}>
    <row assets-error align="center" gap="2" style(bandError, shrink0)>
      <p style(errorText)>{session.errorText()}</p>
      <button type="button" size="sm" variant="ghost" assets-retry on:click={session.onRetry}>Retry</button>
    </row>
  </if>

  <if {session.loading()}>
    <row assets-loading align="center" gap="2" style(bandStatus, shrink0)>
      <spinner size="sm"></spinner>
      <p style(fieldHint)>Loading…</p>
    </row>
  </if>

  <row assets-workbench gap="0" nowrap align="stretch" style(workbench)>
    <scroll-area assets-editor style(editorScroll)>
      <column gap="8" style(editorCol)>
        <column assets-header gap="2">
          <p style(fieldHint)>Assets facet</p>
          <p style(pageTitle)>Type &amp; icons</p>
          <p style(fieldHint)>
            Deep type roles and icon pack judgment. Overview keeps shallow teasers only.
          </p>
          <p style(pathMeta)>{session.modesMeta()}</p>
          <if {session.catalogHint()}>
            <p catalog-hint style(fieldHint)>Add to project to edit.</p>
          </if>
        </column>

        <column section="type" gap="4">
          <row align="baseline" justify="between" gap="3" nowrap>
            <p style(sectionTitle)>Type</p>
            <p style(pathMeta)>display · body · mono</p>
          </row>
          <column gap="3">
            <type-role-select
              token="display"
              label="Display"
              value={assets.typeDisplay()}
              disabled={!session.canEdit()}
              options={assets.displayFontOpts()}
              on-change={assets.onTypeSelect}
            />
            <type-role-select
              token="body"
              label="Body"
              value={assets.typeBody()}
              disabled={!session.canEdit()}
              options={assets.bodyFontOpts()}
              on-change={assets.onTypeSelect}
            />
            <type-role-select
              token="mono"
              label="Mono"
              value={assets.typeMono()}
              disabled={!session.canEdit()}
              options={assets.monoFontOpts()}
              on-change={assets.onTypeSelect}
            />
          </column>

          <column font-discoverer gap="3">
            <row align="baseline" justify="between" gap="3" nowrap>
              <p style(sectionTitle)>Font discoverer</p>
              <p style(pathMeta)>{assets.fontCatalogMeta()}</p>
            </row>
            <p style(fieldHint)>
              Full Google Fonts catalog — search, preview, assign to a role, or install (vendor woff2 into the project).
            </p>
            <row gap="2" align="center" nowrap font-search>
              <input type="search" size="sm" placeholder="Search 1,900+ families (Poppins, Noto, Fraunces…)" value={assets.fontSearchQ()} on:input={assets.onFontSearchInput} label="Search fonts" style(growMin)>
              <button type="button" size="sm" variant="outline" tone="muted" font-search-btn on:click={assets.onFontSearch} style(shrink0)>
                Search
              </button>
            </row>
            <row gap="1" align="center" nowrap font-cats style(catsScroll)>
              <for {assets.fontCategoryOpts() as c} key={c.key}>
                <button type="button" size="xs" variant="outline" tone="muted" font-cat={c.id} selected={c.on ? 'true' : null} on:click={() => assets.onFontCategory(c.id)} style(catChip, shrink0)>
                  {c.label}
                </button>
              </for>
            </row>
            <if {assets.fontSearching()}>
              <row align="center" gap="2" font-loading>
                <spinner size="sm"></spinner>
                <p style(fieldHint)>Searching catalog…</p>
              </row>
            </if>
            <if {assets.hasFontHits()}>
              <column gap="2" font-hits>
                <for {assets.fontSearchHits() as f} key={f.key}>
                  <font-hit-row
                    fontFamily={f.family}
                    category={f.category}
                    sample={f.sample}
                    canEdit={session.canEdit()}
                    onUseRole={assets.onFontUseRole}
                    onInstall={assets.onFontInstall}
                  ></font-hit-row>
                </for>
              </column>
            </if>
            <if {!assets.hasFontHits() && !assets.fontSearching()}>
              <p font-empty style(fieldHint)>No families match — try another query or category.</p>
            </if>
          </column>
        </column>

        <column section="icons" gap="4">
          <row align="baseline" justify="between" gap="3" nowrap>
            <p style(sectionTitle)>Icons</p>
            <p style(pathMeta)>browse · size scale</p>
          </row>
          <column gap="1">
            <p style(fieldHint)>Pack (session until pack token ships)</p>
            <select icon-pack size="sm" value={assets.iconPack()} disabled={!session.canEdit()} on:change={assets.onIconPack}>
              <option value="lucide">Lucide</option>
              <option value="phosphor">Phosphor</option>
            </select>
          </column>
          <column gap="1">
            <p style(fieldHint)>Browse glyphs</p>
            <input
              type="search"
              size="sm"
              placeholder="Filter by name…"
              value={assets.iconQuery()}
              on:input={assets.onIconQuery}
              label="Filter icons"
            />
          </column>
          <if {assets.hasIconGlyphs()}>
            <grid cols="4" gap="2" icon-grid>
              <for {assets.iconGlyphs() as g} key={g.key}>
                <center title={g.name} style(iconCell, shrink0)>
                  <icon name={g.name} size="md" label={g.name}></icon>
                </center>
              </for>
            </grid>
          </if>
          <column icon-scale gap="2">
            <row align="baseline" justify="between" gap="3" nowrap>
              <p style(fieldHint)>Size scale (sm → xl)</p>
              <p style(pathMeta)>{assets.iconScaleReadout()}</p>
            </row>
            <p style(fieldHint)>Drag master to set the md step; other steps follow when scale-together is on.</p>
            <column gap="1">
              <row align="center" justify="between" gap="2" nowrap>
                <p style(fieldHint)>Master</p>
                <p style(pathMeta)>{assets.iconScaleMasterPx()}px</p>
              </row>
              <slider
        
        min="10"
        max="32"
        step="1"
        value={assets.iconScaleMasterPx()}
        disabled={!session.canEdit()}
        icon-scale-master
        label="Icon scale master"
        on:input={assets.onIconScaleMaster}
       />
            </column>
          </column>
        </column>
      </column>
    </scroll-area>

    <column specimen-dock gap="0" style(specimenDock)>
      <column gap="0" style(specimenPad)>
        <card
          assets-specimen
          preview-specimen
          padding="none" style(specimenFrame)>
          <column gap="0" style(growMin)>
            <row preview-chrome align="center" gap="2" nowrap style(specimenChrome)>
              <row gap="2" align="center" nowrap hidden="true" style(trafficDots)>
                <stack style(trafficDot, { bg(destructive) })></stack>
                <stack style(trafficDot, { bg(warning) })></stack>
                <stack style(trafficDot, { bg(success) })></stack>
              </row>
              <p style(chromeTitle)>Preview · {session.nameEdit()}</p>
              <p style(pathMeta, shrink0)>{session.paintMode()}</p>
            </row>
          <scroll-area style(scrollGrow)>
            <column specimen-body gap="5" style(stageBody)>
              <column gap="2" type-stack>
                <p style(typeDisplay)>
                  {assets.typeDisplay()}
                </p>
                <p style(typeBody)>
                  {assets.typeBody()} — body copy for product UI judgment.
                </p>
                <p style(typeMono)>
                  {assets.typeMono()} — const pack = "lucide"
                </p>
              </column>
              <row gap="2" align="center" nowrap specimen-toolbar>
                <button type="button" size="sm" variant="solid" tone="primary">
                  <icon name="plus" size="sm"></icon>
                  New
                </button>
                <button type="button" size="sm" variant="outline" tone="muted">
                  <icon name="search" size="sm"></icon>
                  Find
                </button>
                <button type="button" size="sm" variant="ghost" tone="muted" label="Settings">
                  <icon name="settings" size="sm"></icon>
                </button>
              </row>
              <if {assets.hasIconGlyphs()}>
                <row specimen-icons gap="3" align="center" nowrap>
                  <for {assets.iconGlyphs() as g} key={g.key}>
                    <icon name={g.name} size={assets.iconPreviewSize()} label={g.name} style(shrink0)></icon>
                  </for>
                </row>
              </if>
              <card padding="sm">
                <column gap="2">
                  <row gap="2" align="center" nowrap>
                    <icon name="folder" size="sm"></icon>
                    <p style(doorTitle)>Project list row</p>
                  </row>
                  <p style(fieldHint)>Icons inherit currentColor under the theme table.</p>
                </column>
              </card>
            </column>
          </scroll-area>
          </column>
        </card>
      </column>
    </column>
  </row>

  <toast-region position="bottom-end" assets-toast-region>
    <for {session.toastItems() as t} key={t.key}>
      <toast tone={t.tone} assets-toast on-dismiss={session.onToastDismiss}>
        {t.message}
      </toast>
    </for>
  </toast-region>
</column>
