<!--
  Theme assets facet — type roles + icons deep edit.
  Thin shell: chrome + editor + specimen (draft paint via applyScope bind).
-->
<scope>
  style sectionTitle = { text: fontSize(sm).fontWeight(semibold) }
  style fieldHint = { paint: fg(muted-foreground), text: fontSize(xs) }
  style pathMeta = { paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) }
  style doorTitle = { text: fontWeight(semibold) }
  style editorCol = { box: padX(6).padT(6).padB(10).maxW(panel-max-w-lg) }
  style bandStatus = { box: padX(4).padY(2) }
  style bandError = { box: padX(4).padY(2), paint: borderBWidth(1).borderB(border) }
  style iconCell = { box: w(10).h(10).minW(10).minH(10), paint: radius(md).borderWidth(1).border(border-subtle).bg(surface-0) }
  style fullBtn = { box: w(full) }
  style fontHit = { box: pad(2_5).minW(0), paint: radius(md).borderWidth(1).border(border-subtle).bg(surface-0) }
  style fontSample = { box: minW(0), paint: fg(foreground), text: fontSize(lg).leading(snug) }
  style catChip = { when: { selected: { paint: bg(selected-bg).fg(foreground).border(border) } } }
  type TypeRoleSelectProps = {
    token: string
    label: string
    value: string
    disabled: boolean
    options: any
    onChange: any
  }
</scope>

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

<column surface="themes" theme-assets-surface assets-chrome gap="0" style({  box: grow(1).minH(0).h(full).minW(0).overflow(hidden), paint: bg(background)  })>
  <theme-facet-chrome session={session}></theme-facet-chrome>

  <if {session.hasError()}>
    <row assets-error align="center" gap="2" style(bandError, { box: shrink(0) })>
      <p style({ box: grow(1).minW(0), paint: fg(destructive), text: fontSize(sm).truncate() })>{session.errorText()}</p>
      <button type="button" size="sm" variant="ghost" assets-retry on:click={session.onRetry}>Retry</button>
    </row>
  </if>

  <if {session.loading()}>
    <row assets-loading align="center" gap="2" style(bandStatus, { box: shrink(0) })>
      <spinner size="sm"></spinner>
      <p style(fieldHint)>Loading…</p>
    </row>
  </if>

  <row assets-workbench gap="0" nowrap align="stretch" style({  box: grow(1).minH(0).basis(0).h(full).minW(0).overflow(hidden), paint: bg(background)  })>
    <scroll-area assets-editor style({  box: grow(1).minW(0).minH(0).basis(0).h(full), paint: bg(background)  })>
      <column gap="8" style(editorCol)>
        <column assets-header gap="2">
          <p style(fieldHint)>Assets facet</p>
          <p style({ text: fontSize(xl).fontWeight(semibold).tracking(tight) })>Type &amp; icons</p>
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
              <input type="search" size="sm" placeholder="Search 1,900+ families (Poppins, Noto, Fraunces…)" value={assets.fontSearchQ()} on:input={assets.onFontSearchInput} label="Search fonts" style({ box: grow(1).minW(0) })>
              <button type="button" size="sm" variant="outline" tone="muted" font-search-btn on:click={assets.onFontSearch} style({ box: shrink(0) })>
                Search
              </button>
            </row>
            <row gap="1" align="center" nowrap font-cats style({ box: overflowX(auto).minW(0) })>
              <for {assets.fontCategoryOpts() as c} key={c.key}>
                <button type="button" size="xs" variant="outline" tone="muted" font-cat={c.id} selected={c.on ? 'true' : null} on:click={assets.onFontCategory} style(catChip, { box: shrink(0) })>
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
                  <column gap="2" font-hit font-family={f.family} style(fontHit)>
                    <row align="baseline" justify="between" gap="2" nowrap>
                      <p style(doorTitle, { box: grow(1).minW(0) })>{f.family}</p>
                      <p style(pathMeta, { box: shrink(0) })>{f.category}</p>
                    </row>
                    <p font-sample face={f.family} style(fontSample)>{f.sample}</p>
                    <row gap="1" align="center">
                      <button type="button" size="xs" variant="outline" tone="muted" font-family={f.family} font-token="display" on:click={assets.onFontUseRole} disabled={!session.canEdit()}>
                        Display
                      </button>
                      <button type="button" size="xs" variant="outline" tone="muted" font-family={f.family} font-token="body" on:click={assets.onFontUseRole} disabled={!session.canEdit()}>
                        Body
                      </button>
                      <button type="button" size="xs" variant="outline" tone="muted" font-family={f.family} font-token="mono" on:click={assets.onFontUseRole} disabled={!session.canEdit()}>
                        Mono
                      </button>
                      <button type="button" size="xs" variant="soft" tone="primary" font-family={f.family} font-token="body" on:click={assets.onFontInstall} disabled={!session.canEdit()} style({ box: shrink(0) })>
                        Install
                      </button>
                    </row>
                  </column>
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
                <center title={g.name} style(iconCell, { box: shrink(0) })>
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
              <input
                type="range"
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

    <column specimen-dock gap="0" style({  box: grow(1).minW(0).maxW(panel-max-w-md).basis(0).minH(0).h(full).overflow(hidden), paint: borderLWidth(1).borderL(border-subtle).bg(background)  })>
      <column gap="0" style({  box: grow(1).minH(0).h(full).pad(4)  })>
        <card
          assets-specimen
          preview-specimen
          mode={session.paintMode()}
          padding="none" style({ box: minW(0).minH(0).h(full).overflow(hidden), paint: shadow(lg) })>
          <column gap="0" style({  box: grow(1).minH(0).h(full)  })>
            <row specimen-chrome align="center" gap="2" nowrap style({  box: shrink(0).padX(3).h(8).minH(8).minW(0), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1)  })>
              <row gap="2" align="center" nowrap hidden="true" style({  box: shrink(0).minW(12)  })>
                <stack style({  box: shrink(0).w(2).h(2).minW(2).minH(2), paint: radius(full).bg(destructive)  })></stack>
                <stack style({  box: shrink(0).w(2).h(2).minW(2).minH(2), paint: radius(full).bg(warning)  })></stack>
                <stack style({  box: shrink(0).w(2).h(2).minW(2).minH(2), paint: radius(full).bg(success)  })></stack>
              </row>
              <p style({ box: grow(1).minW(0), paint: fg(muted-foreground), text: fontSize(xs).truncate() })>{session.nameEdit()}</p>
              <p style(pathMeta, { box: shrink(0) })>{session.paintMode()}</p>
            </row>
          <scroll-area style({  box: grow(1).minH(0).h(full), paint: bg(background)  })>
            <column specimen-body gap="5" style({ box: pad(5).minW(0) })>
              <column gap="2" type-stack>
                <p style({ text: fontFamily(display).fontSize(2xl).fontWeight(semibold).tracking(tight) })>
                  {assets.typeDisplay()}
                </p>
                <p style({ text: fontFamily(body).fontSize(sm).leading(relaxed) })>
                  {assets.typeBody()} — body copy for product UI judgment.
                </p>
                <p style({ text: fontFamily(mono).fontSize(xs) })>
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
                    <icon name={g.name} size={assets.iconPreviewSize()} label={g.name} style({ box: shrink(0) })></icon>
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
