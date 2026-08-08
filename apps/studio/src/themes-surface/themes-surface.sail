<!--
  <themes-surface> — project theme catalog master–detail.
  List (In project / Catalog) + preview (color story, type, live chrome) under scoped theme.
  Membership = project file on disk. Catalog: Add to project (confirm).
-->
<scope>
  style pageHeader = { box: shrink(0).padX(6).padY(4), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1) }
  style errorBand = { box: shrink(0).padX(6).padY(2), paint: borderBWidth(1).borderB(border) }
  style loadBand = { box: shrink(0).padX(6).padY(2) }
  style workbench = { box: grow(1).minH(0).basis(0).h(full).minW(0).overflow(hidden) }
  style listPane = { box: grow(1).minH(0).minW(panel-min-w-md).maxW(panel-max-w-md).basis(0), paint: borderRWidth(1).borderR(border-subtle).bg(surface-1) }
  style listHead = { box: shrink(0).padX(4).padT(4).padB(3), paint: borderBWidth(1).borderB(border-subtle) }
  style groupLabel = { box: padX(2).padT(1).padB(1), paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide) }
  style groupLabelCatalog = { box: padX(2).padT(4).padB(1), paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide) }
  style themeRow = { box: pad(2), paint: radius(md).cursor(pointer).borderWidth(1).border(transparent), when: { hover: { paint: bg(hover-bg) }, current: { paint: bg(selected-bg).border(selected-border) } } }
  style swatchStrip = { box: shrink(0).w(14).h(7).minW(14).overflow(hidden), paint: radius(sm).borderWidth(1).border(border-subtle) }
  style swatchCell = { box: grow(1).minW(0).h(full) }
  style growMin = { box: grow(1).minW(0) }
  style shrink0 = { box: shrink(0) }
  style titleSm = { text: fontSize(sm).fontWeight(semibold).truncate() }
  style mutedXs = { paint: fg(muted-foreground), text: fontSize(xs).truncate() }
  style mutedSm = { paint: fg(muted-foreground), text: fontSize(sm) }
  style mutedSmLead = { paint: fg(muted-foreground), text: fontSize(sm).leading(snug) }
  style mutedXsPlain = { paint: fg(muted-foreground), text: fontSize(xs) }
  style pageTitle = { paint: fg(foreground), text: fontSize(lg).fontWeight(semibold).tracking(tight) }
  style libraryEyebrow = { paint: fg(muted-foreground), text: fontSize(xs).fontWeight(bold).textCase(uppercase).tracking(wide) }
  style sectionEyebrow = { paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide) }
  style errorText = { box: grow(1).minW(0), paint: fg(destructive), text: fontSize(sm).truncate() }
  style surfaceRoot = { box: grow(1).minH(0).h(full).minW(0).overflow(hidden), paint: bg(background) }
  style scrollGrow = { box: grow(1).minH(0).basis(0).h(full) }
  style listPad = { box: pad(3) }
  style previewScroll = { box: grow(1).minW(0).minH(0).basis(0).h(full), paint: bg(background) }
  style previewCol = { box: padX(8).padT(6).padB(10).maxW(panel-max-w-lg) }
  style previewTitle = { text: fontFamily(display).fontSize(2xl).fontWeight(medium).tracking(tight) }
  style previewDesc = { paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) }
  style modeLabel = { paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold) }
  style modeIcon = { box: shrink(0), paint: fg(muted-foreground) }
  style storySwatch = { box: h(16), paint: radius(md).borderWidth(1).border(border-subtle) }
  style storyLabel = { text: fontSize(xs).fontWeight(medium) }
  style typeStory = { box: pad(4), paint: radius(lg).borderWidth(1).border(border-subtle).bg(surface-1).fg(foreground) }
  style typeDisplay = { text: fontFamily(display).fontSize(xl).fontWeight(semibold).tracking(tight) }
  style typeBody = { text: fontFamily(body).fontSize(sm).leading(relaxed) }
  style typeMono = { text: fontFamily(mono).fontSize(xs) }
  style radiusChip = { box: w(8).h(8), paint: bg(transparent).borderWidth(2).border(primary) }
  style specimen = { box: overflow(hidden).minW(0), paint: radius(lg).borderWidth(1).border(border).shadow(md).bg(background).fg(foreground) }
  style specimenHead = { box: padX(4).h(9), paint: bg(surface-1).borderBWidth(1).borderB(border-subtle) }
  style specimenBody = { box: pad(5), paint: bg(background) }
  style specimenCard = { box: pad(4) }
  style cardTitle = { text: fontSize(md).fontWeight(semibold).fontFamily(heading) }
  style cardBody = { paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed).fontFamily(body) }
  style fullW = { box: w(full) }

  type ThemeSwatchProps = {
    previewId: string
    paintMode: any
  }
  type ThemeListRowProps = {
    id: string
    name: string
    meta: string
    menuId: string
    selected: boolean
    previewId: string
    paintMode: any
    scope: string
    isCatalog: boolean
    onAddToProject: any
    onMenuOpen: any
    onRowMenu: any
  }
</scope>

<partial theme-swatches props: ThemeSwatchProps>
  <row
    theme-swatches
    theme={previewId}
    use:paintMode={paintMode}
    gap="0"
    nowrap
    hidden="true"
    style(swatchStrip)
  >
    <center style(swatchCell, { paint: bg(background) })></center>
    <center style(swatchCell, { paint: bg(surface-2) })></center>
    <center style(swatchCell, { paint: bg(primary) })></center>
    <center style(swatchCell, { paint: bg(accent) })></center>
  </row>
</partial>

<partial theme-list-row props: ThemeListRowProps>
  <row
    theme-row
    theme-id={id}
    theme-scope={scope}
    selected={selected ? 'true' : 'false'}
    current={selected ? 'true' : null}
    gap="3"
    align="center"
    nowrap
    style(themeRow)
  >
    <theme-swatches previewId={previewId} paintMode={paintMode}></theme-swatches>
    <column gap="0" style(growMin)>
      <p style(titleSm)>{name}</p>
      <p style(mutedXs)>{meta}</p>
    </column>
    <if {isCatalog}>
      <badge tone="muted" size="sm" variant="outline" pill style(shrink0)>Catalog</badge>
      <row gap="1" align="center" nowrap style(shrink0)>
        <button
          type="button"
          size="sm"
          variant="outline"
          tone="primary"
          theme-add-project
          theme-id={id}
          label="Add to project"
          on:click={onAddToProject}
        >
          Add
        </button>
        <button
          type="button"
          size="sm"
          variant="ghost"
          tone="muted"
          popovertarget={menuId}
          theme-menu
          theme-id={id}
          label="Theme menu"
          on:click={onMenuOpen}
        >
          <icon name="ellipsis" size="sm"></icon>
        </button>
        <dropdown id={menuId} placement="bottom-end" on-select={onRowMenu}>
          <menu-item value="enable">Add to project</menu-item>
          <menu-item value="open">Open</menu-item>
          <menu-item value="duplicate">Duplicate</menu-item>
        </dropdown>
      </row>
    </if>
    <if {!isCatalog}>
      <row gap="1" align="center" nowrap style(shrink0)>
        <button
          type="button"
          size="sm"
          variant="ghost"
          tone="muted"
          popovertarget={menuId}
          theme-menu
          theme-id={id}
          label="Theme menu"
          on:click={onMenuOpen}
        >
          <icon name="ellipsis" size="sm"></icon>
        </button>
        <dropdown id={menuId} placement="bottom-end" on-select={onRowMenu}>
          <menu-item value="open">Open</menu-item>
          <menu-item value="remix">Remix</menu-item>
          <menu-item value="delete">Delete</menu-item>
        </dropdown>
      </row>
    </if>
  </row>
</partial>

<controller>
import type { Action } from '@ship/sail-runtime'
import { createThemesSurface } from '../compositions/themes-surface/themes-surface-vm'
export default controller(() => {
  const vm = createThemesSurface()
  // Stamp data-mode via action (no data-* in markup).
  const paintMode: Action<string | null | undefined> = (node, initial) => {
    const el = node as HTMLElement
    const apply = (m: string | null | undefined): void => {
      if (m === 'light' || m === 'dark') el.setAttribute('data-mode', m)
      else el.removeAttribute('data-mode')
    }
    apply(initial)
    return {
      update: (next) => apply(next),
      destroy: () => el.removeAttribute('data-mode'),
    }
  }
  return {
    projectRows: vm.projectRows,
    catalogRows: vm.catalogRows,
    hasProjectRows: vm.hasProjectRows,
    hasCatalogRows: vm.hasCatalogRows,
    hasAnyThemes: vm.hasAnyThemes,
    hasSelected: vm.hasSelected,
    previewTitle: vm.previewTitle,
    previewDesc: vm.previewDesc,
    previewMeta: vm.previewMeta,
    previewStatusLabel: vm.previewStatusLabel,
    previewStatusTone: vm.previewStatusTone,
    previewInProject: vm.previewInProject,
    previewIsCatalog: vm.previewIsCatalog,
    previewThemeId: vm.previewThemeId,
    previewPaintId: vm.previewPaintId,
    previewModeId: vm.previewModeId,
    previewModesLabel: vm.previewModesLabel,
    canToggleMode: vm.canToggleMode,
    modeLightOn: vm.modeLightOn,
    modeDarkOn: vm.modeDarkOn,
    showToast: vm.showToast,
    toastMessage: vm.toastMessage,
    toastTone: vm.toastTone,
    toastItems: vm.toastItems,
    themesError: vm.themesError,
    hasThemesError: vm.hasThemesError,
    loadingLabel: vm.loadingLabel,
    isLoading: vm.isLoading,
    projectCountLabel: vm.projectCountLabel,
    catalogCountLabel: vm.catalogCountLabel,
    paintMode,
    onListClick: (e: Event) => vm.onListClick(e),
    onMenuOpen: (e: Event) => vm.onMenuOpen(e),
    onAddToProject: (e: Event) => vm.onAddToProject(e),
    onPreviewOpen: () => vm.onPreviewOpen(),
    onPreviewAddToProject: () => vm.onPreviewAddToProject(),
    onPreviewModeSwitch: (e: Event) => vm.onPreviewModeSwitch(e),
    onNewTheme: () => vm.onNewTheme(),
    onRowMenu: (detail: { value?: string }) => vm.onRowMenu(detail),
    onRetryThemes: () => vm.onRetryThemes(),
    onToastDismiss: () => vm.onToastDismiss(),
  }
})
</controller>

<column surface="themes" themes-surface gap="0" style(surfaceRoot)>
  <row themes-header align="center" justify="between" gap="4" nowrap style(pageHeader)>
    <column gap="1" style(growMin)>
      <p style(pageTitle)>Themes</p>
      <p style(mutedSmLead)>
        Project themes live on disk. Add catalog themes to the project, then open one to edit.
      </p>
    </column>
    <row gap="2" align="center" nowrap style(shrink0)>
      <button type="button" size="sm" variant="solid" tone="primary" new-theme on:click={onNewTheme}>
        New theme
      </button>
    </row>
  </row>

  <if {hasThemesError()}>
    <row themes-error align="center" gap="2" style(errorBand)>
      <p style(errorText)>{themesError()}</p>
      <button type="button" size="sm" variant="ghost" retry-themes on:click={onRetryThemes}>
        Retry
      </button>
    </row>
  </if>

  <if {isLoading()}>
    <row themes-loading align="center" gap="2" style(loadBand)>
      <spinner size="sm"></spinner>
      <p style(mutedSm)>{loadingLabel()}</p>
    </row>
  </if>

  <row themes-workbench themes-split gap="0" nowrap align="stretch" style(workbench)>
    <column themes-list-pane gap="0" style(listPane)>
      <column themes-list-head gap="1" style(listHead)>
        <p style(libraryEyebrow)>Library</p>
        <p style(mutedXsPlain)>
          {projectCountLabel()} in project · {catalogCountLabel()} catalog
        </p>
      </column>

      <scroll-area themes-list on:click={onListClick} style(scrollGrow)>
        <column gap="1" style(listPad)>
          <if {!hasAnyThemes()}>
            <if {!hasThemesError()}>
              <empty-state variant="placeholder" size="sm" themes-empty style(fullW)>
                <strong slot="title">No themes loaded</strong>
                Retry, or create a project theme with New theme.
              </empty-state>
            </if>
          </if>

          <if {hasProjectRows()}>
            <p list-group="project" style(groupLabel)>In project</p>
            <for {projectRows() as row} key={row.key}>
              <theme-list-row
                id={row.id}
                name={row.name}
                meta={row.meta}
                menuId={row.menuId}
                selected={row.selected}
                previewId={row.previewId}
                paintMode={row.paintMode}
                scope="project"
                isCatalog={false}
                onAddToProject={onAddToProject}
                onMenuOpen={onMenuOpen}
                onRowMenu={onRowMenu}
              ></theme-list-row>
            </for>
          </if>

          <if {hasCatalogRows()}>
            <p list-group="catalog" style(groupLabelCatalog)>Catalog</p>
            <for {catalogRows() as row} key={row.key}>
              <theme-list-row
                id={row.id}
                name={row.name}
                meta={row.meta}
                menuId={row.menuId}
                selected={row.selected}
                previewId={row.previewId}
                paintMode={row.paintMode}
                scope="catalog"
                isCatalog={true}
                onAddToProject={onAddToProject}
                onMenuOpen={onMenuOpen}
                onRowMenu={onRowMenu}
              ></theme-list-row>
            </for>
          </if>
        </column>
      </scroll-area>
    </column>

    <scroll-area themes-preview theme-preview-pane style(previewScroll)>
      <column gap="6" style(previewCol)>
        <if {!hasSelected()}>
          <empty-state variant="placeholder" size="sm" themes-preview-empty style(fullW)>
            <strong slot="title">Nothing selected</strong>
            Pick a theme in the library to preview its surfaces and controls.
          </empty-state>
        </if>

        <if {hasSelected()}>
          <column preview-head gap="3">
            <row gap="2" align="center">
              <p style(previewTitle)>{previewTitle()}</p>
              <if {previewStatusLabel()}>
                <badge tone={previewStatusTone()} size="sm" variant="solid" pill>
                  {previewStatusLabel()}
                </badge>
              </if>
              <badge tone="muted" size="sm" variant="outline" pill modes-chip>
                {previewModesLabel()}
              </badge>
            </row>
            <p style(previewDesc)>{previewDesc()}</p>
            <p style(mutedXsPlain)>{previewMeta()}</p>

            <if {canToggleMode()}>
              <row preview-mode dual-mode gap="2" align="center" nowrap label="Theme mode">
                <p style(modeLabel)>Mode</p>
                <icon name="sun" size="sm" label="Light" style(modeIcon)></icon>
                <switch
                  size="sm"
                  tone="muted"
                  mode-switch
                  checked={modeDarkOn()}
                  label={modeDarkOn() ? 'Switch to light mode' : 'Switch to dark mode'}
                  on:change={onPreviewModeSwitch}
                ></switch>
                <icon name="moon" size="sm" label="Dark" style(modeIcon)></icon>
              </row>
            </if>
            <if {!canToggleMode()}>
              <p mode-fixed style(mutedXsPlain)>
                This theme ships {previewModesLabel().toLowerCase()} — preview uses that table.
              </p>
            </if>

            <row gap="2" align="center">
              <if {previewInProject()}>
                <button type="button" size="sm" variant="solid" tone="primary" preview-open on:click={onPreviewOpen}>
                  Open
                </button>
              </if>
              <if {previewIsCatalog()}>
                <button type="button" size="sm" variant="solid" tone="primary" preview-add-project on:click={onPreviewAddToProject}>
                  Add to project
                </button>
                <button type="button" size="sm" variant="outline" tone="primary" preview-open on:click={onPreviewOpen}>
                  Open
                </button>
              </if>
            </row>
          </column>

          <column color-story gap="3">
            <p style(sectionEyebrow)>Color story</p>
            <row theme={previewPaintId()} use:paintMode={previewModeId()} gap="2" color-story-grid>
              <column gap="1" style(growMin)>
                <center style(storySwatch, { paint: bg(primary) })></center>
                <p style(storyLabel)>Primary</p>
              </column>
              <column gap="1" style(growMin)>
                <center style(storySwatch, { paint: bg(secondary) })></center>
                <p style(storyLabel)>Secondary</p>
              </column>
              <column gap="1" style(growMin)>
                <center style(storySwatch, { paint: bg(accent) })></center>
                <p style(storyLabel)>Accent</p>
              </column>
              <column gap="1" style(growMin)>
                <center style(storySwatch, { paint: bg(background) })></center>
                <p style(storyLabel)>Background</p>
              </column>
              <column gap="1" style(growMin)>
                <center style(storySwatch, { paint: bg(surface-1) })></center>
                <p style(storyLabel)>Surface</p>
              </column>
            </row>
          </column>

          <column
            type-story
            theme={previewPaintId()}
            use:paintMode={previewModeId()}
            gap="2"
            style(typeStory)
          >
            <p style(sectionEyebrow)>Type · corners</p>
            <p style(typeDisplay)>Display / heading sample</p>
            <p style(typeBody)>
              Body — the quick brown fox jumps over the lazy dog. 0123456789
            </p>
            <p style(typeMono)>mono — const theme = "preview"</p>
            <row gap="2" align="end" radius-chips>
              <column gap="1" align="center">
                <center hidden="true" style(radiusChip, { paint: radius(sm) })></center>
                <p style(mutedXsPlain)>sm</p>
              </column>
              <column gap="1" align="center">
                <center hidden="true" style(radiusChip, { paint: radius(md) })></center>
                <p style(mutedXsPlain)>md</p>
              </column>
              <column gap="1" align="center">
                <center hidden="true" style(radiusChip, { paint: radius(lg) })></center>
                <p style(mutedXsPlain)>lg</p>
              </column>
              <column gap="1" align="center">
                <center hidden="true" style(radiusChip, { paint: radius(xl) })></center>
                <p style(mutedXsPlain)>xl</p>
              </column>
            </row>
          </column>

          <column preview-chrome gap="2">
            <p style(sectionEyebrow)>Live chrome</p>
            <column
              theme={previewPaintId()}
              use:paintMode={previewModeId()}
              preview-specimen
              gap="0"
              style(specimen)
            >
              <row align="center" gap="2" nowrap justify="between" style(specimenHead)>
                <p style(titleSm)>{previewTitle()}</p>
                <badge tone="primary" size="sm" variant="solid" pill>Preview</badge>
              </row>
              <column gap="4" style(specimenBody)>
                <card style(specimenCard)>
                  <column gap="3">
                    <p style(cardTitle)>Sample card</p>
                    <p style(cardBody)>
                      Controls, badges, and fields read this theme’s token table.
                    </p>
                    <row gap="2" align="center">
                      <button type="button" size="sm" variant="solid" tone="primary">Primary</button>
                      <button type="button" size="sm" variant="solid" tone="accent">Accent</button>
                      <button type="button" size="sm" variant="outline" tone="primary">Outline</button>
                      <button type="button" size="sm" variant="ghost" tone="muted">Ghost</button>
                    </row>
                    <row gap="2" align="center">
                      <badge tone="primary" size="sm" variant="solid" pill>Primary</badge>
                      <badge tone="success" size="sm" variant="solid" pill>Success</badge>
                      <badge tone="warning" size="sm" variant="solid" pill>Warning</badge>
                      <badge tone="destructive" size="sm" variant="solid" pill>Danger</badge>
                    </row>
                    <input type="search" size="sm" placeholder="Search field" label="Sample search" />
                  </column>
                </card>
              </column>
            </column>
          </column>
        </if>
      </column>
    </scroll-area>
  </row>

  <toast-region position="bottom-end" themes-toast-region>
    <for {toastItems() as t} key={t.key}>
      <toast tone={t.tone} themes-toast on-dismiss={onToastDismiss}>
        {t.message}
      </toast>
    </for>
  </toast-region>
</column>
