<!--
  <themes-surface> — project theme catalog master–detail.
  List (In project / Catalog) + preview (color story, type, live chrome) under scoped theme.
  Membership = project file on disk. Catalog: Add to project (confirm).
-->

<scope>
  // no preset: edge-only border
  style pageHeader = { size(hug).pad(6).border(subtle-b).bg(surface) }
  // no preset: edge-only border
  style errorBand = { size(hug).pad(6).border(default-b) }
  style loadBand = { size(hug).pad(6) }
  style workbench = { size(grow).overflow(hidden) }
  // no preset: edge-only border
  style listPane = { size(grow).border(subtle-r).bg(surface) }
  // no preset: edge-only border
  style listHead = { size(hug).pad(4).border(subtle-b) }
  style groupLabel = { pad(2).fg(muted).type(label) }
  style groupLabelCatalog = { pad(2).fg(muted).type(label) }
  // no preset: transparent border
  style themeRow = { pad(2).radius(md).cursor(pointer).border({ color: transparent, width: 1 }), when: { hover: bg(hover-bg), current: bg(selected-bg).border({ color: selected-border }) } }
  // no preset: size lock
  style swatchStrip = { size({ w: 14, h: 7, min-w: 14 }).overflow(hidden).radius(sm).border(subtle) }
  style swatchCell = { size(grow) }
  style swatchCanvas = { bg(app) }
  style swatchSurface2 = { bg(surface) }
  style swatchPrimary = { bg(primary) }
  style swatchAccent = { bg(accent) }
  style swatchSurface1 = { bg(surface) }
  style swatchSecondary = { bg(secondary) }
  style growMin = { size(grow) }
  style shrink0 = { size(hug) }
  style titleSm = { type(body-sm) }
  style mutedXs = { fg(muted).type(body-sm) }
  style mutedSm = { fg(muted).type(body-sm) }
  style mutedSmLead = { fg(muted).type(body-sm) }
  style mutedXsPlain = { fg(muted).type(body-sm) }
  style pageTitle = { fg(foreground).type(heading) }
  style libraryEyebrow = { fg(muted).type(label) }
  style sectionEyebrow = { fg(muted).type(label) }
  style errorText = { size(grow).fg(danger).type(body-sm) }
  style surfaceRoot = { grow(1).overflow(hidden).bg(app) }
  style scrollGrow = { size(grow) }
  style listPad = { pad(3) }
  style previewScroll = { size(grow).bg(app) }
  style previewTitle = { type(display-sm) }
  style previewDesc = { fg(muted).type(body-sm) }
  style modeLabel = { fg(muted).type(label) }
  style modeIcon = { size(hug).fg(muted) }
  // no preset: size lock
  style storySwatch = { size({ h: 16 }).radius(md).border(subtle) }
  style storyLabel = { type(body-sm) }
  style typeStory = { pad(4).radius(lg).border(subtle).bg(surface).fg(foreground) }
  style typeDisplay = { type(display-sm) }
  style typeBody = { type(body-sm) }
  style typeMono = { type(mono-sm) }
  // no preset: size lock + transparent border
  style radiusChip = { size({ w: 8, h: 8, min-w: 8, min-h: 8 }).bg(transparent).border({ color: primary, width: 2 }) }
  // no preset: min-w lock
  style specimen = { overflow(hidden).size({ min-w: 0 }).radius(lg).border(subtle).elevation(md).bg(app).fg(foreground) }
  // no preset: size lock + edge-only border
  style specimenHead = { pad(4).size({ h: 9 }).bg(surface).border(subtle-b) }
  style specimenBody = { pad(5).bg(app) }
  // no preset: min-w lock
  style specimenCard = { pad(4).size({ min-w: 0 }) }
  // no preset: min-w lock
  style specimenActions = { size({ min-w: 0 }).overflow({ x: auto }) }
  // no preset: type wrap
  style chipLabel = { size(hug).type({ wrap: truncate }) }
  style cardTitle = { type(heading) }
  style cardBody = { fg(muted).type(body-sm) }
  style fullW = { size(max) }
  style tapFloor = { size(control-sm) }
  style iconBtn = { size(control-sm) }

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
    onListClick: any
    onAddToProject: any
    onMenuOpen: any
    onRowMenu: any
  }
</scope>

<controller>
import type { UiEvent } from '../compositions/ui-event'
import type { Action } from '@ship/sail-runtime'
import { hostSetAttribute, hostRemoveAttribute } from '../compositions/host-paint'
import { createThemesSurface } from '../compositions/themes-surface/themes-surface-vm'
export default controller(() => {
  const vm = createThemesSurface()
  // Mode stamp for themed preview subtrees (action writes host attribute).
  const paintMode: Action<string | null | undefined> = (node, initial) => {
    const host = asSailHost(node)
    const apply = (m: string | null | undefined): void => {
      if (m === 'light' || m === 'dark') {
        hostSetAttribute(host, 'mode', m)
        hostRemoveAttribute(host, 'data-mode')
      } else hostRemoveAttribute(host, 'mode')
    }
    apply(initial)
    return {
      update: (next) => apply(next),
      destroy: () => hostRemoveAttribute(host, 'mode'),
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
    onListClick: (id: string) => vm.onListClick(id),
    onMenuOpen: (id: string, e?: UiEvent) => vm.onMenuOpen(id, e),
    onAddToProject: (id: string, e?: UiEvent) => vm.onAddToProject(id, e),
    onPreviewOpen: () => vm.onPreviewOpen(),
    onPreviewAddToProject: () => vm.onPreviewAddToProject(),
    onPreviewModeSwitch: (e: UiEvent) => vm.onPreviewModeSwitch(e),
    onNewTheme: () => vm.onNewTheme(),
    onRowMenu: (detail: { value?: string }) => vm.onRowMenu(detail),
    onRetryThemes: () => vm.onRetryThemes(),
    onToastDismiss: () => vm.onToastDismiss(),
  }
})
</controller>

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
    <center style(swatchCell, swatchCanvas)></center>
    <center style(swatchCell, swatchSurface2)></center>
    <center style(swatchCell, swatchPrimary)></center>
    <center style(swatchCell, swatchAccent)></center>
  </row>
</partial>

<partial theme-list-row props: ThemeListRowProps>
  <row
    theme-row
    theme-scope={scope}
    selected={selected ? 'true' : 'false'}
    current={selected ? 'true' : null}
    gap="3"
    align="center"
    nowrap
    style(themeRow)
    on:click={() => onListClick(id)}
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
          label="Add to project"
          on:click={(e) => onAddToProject(id, e)}
          style(tapFloor)
        >
          Add
        </button>
        <button
          type="button"
          size="iconSm"
          variant="ghost"
          tone="muted"
          popovertarget={menuId}
          theme-menu
          label="Theme menu"
          on:click={(e) => onMenuOpen(id, e)}
          style(iconBtn)
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
          size="iconSm"
          variant="ghost"
          tone="muted"
          popovertarget={menuId}
          theme-menu
          label="Theme menu"
          on:click={(e) => onMenuOpen(id, e)}
          style(iconBtn)
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

<column surface="themes" themes-surface gap="0" style(surfaceRoot)>
  <row themes-header align="center" justify="between" gap="4" nowrap style(pageHeader)>
    <column gap="1" style(growMin)>
      <p style(pageTitle)>Themes</p>
      <p style(mutedSmLead)>
        Project themes live on disk. Add catalog themes to the project, then open one to edit.
      </p>
    </column>
    <row gap="2" align="center" nowrap style(shrink0)>
      <button type="button" size="sm" variant="solid" tone="primary" new-theme on:click={onNewTheme} style(tapFloor)>
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

      <scroll-area themes-list style(scrollGrow)>
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
                onListClick={onListClick}
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
                onListClick={onListClick}
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
      <column gap="6" style({ pad(8).size({ max-w: panel-max-w-lg }) })>
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
                <button type="button" size="sm" variant="solid" tone="primary" preview-open on:click={onPreviewOpen} style(tapFloor)>
                  Open
                </button>
              </if>
              <if {previewIsCatalog()}>
                <button type="button" size="sm" variant="solid" tone="primary" preview-add-project on:click={onPreviewAddToProject} style(tapFloor)>
                  Add to project
                </button>
                <button type="button" size="sm" variant="outline" tone="primary" preview-open on:click={onPreviewOpen} style(tapFloor)>
                  Open
                </button>
              </if>
            </row>
          </column>

          <column color-story gap="3">
            <p style(sectionEyebrow)>Color story</p>
            <row theme={previewPaintId()} use:paintMode={previewModeId()} gap="2" color-story-grid>
              <column gap="1" style(growMin)>
                <center style(storySwatch, swatchPrimary)></center>
                <p style(storyLabel)>Primary</p>
              </column>
              <column gap="1" style(growMin)>
                <center style(storySwatch, swatchSecondary)></center>
                <p style(storyLabel)>Secondary</p>
              </column>
              <column gap="1" style(growMin)>
                <center style(storySwatch, swatchAccent)></center>
                <p style(storyLabel)>Accent</p>
              </column>
              <column gap="1" style(growMin)>
                <center style(storySwatch, swatchCanvas)></center>
                <p style(storyLabel)>Background</p>
              </column>
              <column gap="1" style(growMin)>
                <center style(storySwatch, swatchSurface1)></center>
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
                <center hidden="true" style(radiusChip, { radius(sm) })></center>
                <p style(mutedXsPlain)>sm</p>
              </column>
              <column gap="1" align="center">
                <center hidden="true" style(radiusChip, { radius(md) })></center>
                <p style(mutedXsPlain)>md</p>
              </column>
              <column gap="1" align="center">
                <center hidden="true" style(radiusChip, { radius(lg) })></center>
                <p style(mutedXsPlain)>lg</p>
              </column>
              <column gap="1" align="center">
                <center hidden="true" style(radiusChip, { radius(xl) })></center>
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
                <badge tone="primary" size="sm" variant="solid" pill style(chipLabel)>Preview</badge>
              </row>
              <column gap="4" style(specimenBody)>
                <card style(specimenCard)>
                  <column gap="3">
                    <p style(cardTitle)>Sample card</p>
                    <p style(cardBody)>
                      Controls, badges, and fields read this theme’s token table.
                    </p>
                    <row gap="2" align="center" style(specimenActions)>
                      <button type="button" size="sm" variant="solid" tone="primary" style(tapFloor)>Primary</button>
                      <button type="button" size="sm" variant="solid" tone="accent" style(tapFloor)>Accent</button>
                      <button type="button" size="sm" variant="outline" tone="primary" style(tapFloor)>Outline</button>
                      <button type="button" size="sm" variant="ghost" tone="muted" style(tapFloor)>Ghost</button>
                    </row>
                    <row gap="2" align="center" style(specimenActions)>
                      <badge tone="primary" size="sm" variant="solid" pill style(chipLabel)>Primary</badge>
                      <badge tone="success" size="sm" variant="solid" pill style(chipLabel)>Success</badge>
                      <badge tone="warning" size="sm" variant="solid" pill style(chipLabel)>Warning</badge>
                      <badge tone="danger" size="sm" variant="solid" pill style(chipLabel)>Danger</badge>
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
