<!--
  <themes-surface> — project theme catalog master–detail (Variant B).
  List (In project / Catalog) + preview with color story + L1 chrome under scoped theme.
  Membership = project file on disk. Catalog: Add to project (confirm). No fake enable/default.
  Ban-clean: layout keywords + L1. Color identity via theme= + registered tables (no CSSOM).
-->
<controller>
import { createThemesSurface } from './themes-surface-vm'
export default controller(() => {
  const vm = createThemesSurface()
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
    onListClick: (e: Event) => vm.onListClick(e),
    onMenuOpen: (e: Event) => vm.onMenuOpen(e),
    onAddToProject: (e: Event) => vm.onAddToProject(e),
    onPreviewOpen: () => vm.onPreviewOpen(),
    onPreviewAddToProject: () => vm.onPreviewAddToProject(),
    onPreviewModeSwitch: (e: Event) => vm.onPreviewModeSwitch(e),
    onNewTheme: () => vm.onNewTheme(),
    onRowMenu: (e: Event) => vm.onRowMenu(e),
    onRetryThemes: () => vm.onRetryThemes(),
    onToastDismiss: () => vm.onToastDismiss(),
  }
})
</controller>

<column
  surface="themes"
  themes-surface
  gap="0"
  grow
  sx={style.minH(0).h(full).minW(0).overflow(hidden).bg(background)}
>
  <!-- Page header — not the compact uppercase chrome strip -->
  <row
    themes-header
    align="center"
    justify="between"
    gap="4"
    shrink="0"
    nowrap
    sx={style.padX(6).padY(4).borderBWidth(1).borderB(border-subtle).bg(surface-1)}
  >
    <column grow min-w="0" gap="1">
      <p sx={style.fontSize(lg).fontWeight(semibold).fg(foreground).tracking(tight)}>
        Themes
      </p>
      <p sx={style.fontSize(sm).fg(muted-foreground).leading(snug)}>
        Project themes live on disk. Add catalog themes to the project, then open one to edit.
      </p>
    </column>
    <row gap="2" align="center" shrink="0" nowrap>
      <button type="button" size="sm" variant="solid" tone="primary" new-theme on:click={onNewTheme}>
        New theme
      </button>
    </row>
  </row>

  <if {hasThemesError()}>
    <row themes-error align="center" gap="2" shrink="0" sx={style.padX(6).padY(2).borderBWidth(1).borderB(border)}>
      <p grow min-w="0" sx={style.fontSize(sm).fg(destructive).truncate()}>{themesError()}</p>
      <button type="button" size="sm" variant="ghost" retry-themes on:click={onRetryThemes}>
        Retry
      </button>
    </row>
  </if>

  <if {isLoading()}>
    <row themes-loading align="center" gap="2" shrink="0" sx={style.padX(6).padY(2)}>
      <spinner size="sm"></spinner>
      <p sx={style.fontSize(sm).fg(muted-foreground)}>{loadingLabel()}</p>
    </row>
  </if>

  <row
    themes-workbench
    themes-split
    grow
    gap="0"
    nowrap
    align="stretch"
    sx={style.minH(0).basis(0).h(full).minW(0).overflow(hidden)}
  >
    <!-- LIST -->
    <column
      themes-list-pane
      grow
      gap="0"
      sx={style.minH(0).minW(panel-min-w-md).maxW(panel-max-w-md).basis(0).borderRWidth(1).borderR(border-subtle).bg(surface-1)}
    >
      <column themes-list-head shrink="0" gap="1" sx={style.padX(4).padT(4).padB(3).borderBWidth(1).borderB(border-subtle)}>
        <p sx={style.fontSize(xs).fontWeight(bold).textCase(uppercase).tracking(wide).fg(muted-foreground)}>
          Library
        </p>
        <p sx={style.fontSize(xs).fg(muted-foreground)}>
          {projectCountLabel()} in project · {catalogCountLabel()} catalog
        </p>
      </column>

      <scroll-area themes-list grow on:click={onListClick} sx={style.minH(0).basis(0).h(full)}>
        <column gap="1" sx={style.pad(3)}>
          <if {!hasAnyThemes()}>
            <if {!hasThemesError()}>
              <empty-state variant="placeholder" size="sm" w="full" themes-empty>
                <strong slot="title">No themes loaded</strong>
                Retry, or create a project theme with New theme.
              </empty-state>
            </if>
          </if>

          <if {hasProjectRows()}>
            <p
              list-group="project"
              sx={style.fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide).fg(muted-foreground).padX(2).padT(1).padB(1)}
            >
              In project
            </p>
            <for {projectRows()} as="row" key={row.key}>
              <row
                theme-row
                theme-id={row.id}
                theme-scope="project"
                selected={row.selected ? 'true' : 'false'}
                aria-current={row.selected ? 'true' : null}
                gap="3"
                align="center"
                nowrap
                sx={style.pad(2).radius(md).cursor(pointer).borderWidth(1).border(transparent).when({ hover: style.bg(hover-bg) }).when({ current: style.bg(selected-bg).border(selected-border) })}
              >
                <!-- Color identity: scoped token paint via theme= (registered tables) -->
                <row
                  theme-swatches
                  theme={row.previewId}
                  mode={row.paintMode}
                  shrink="0"
                  gap="0"
                  nowrap
                  sx={style.w(14).h(7).minW(14).radius(sm).overflow(hidden).borderWidth(1).border(border-subtle)}
                  aria-hidden="true"
                >
                  <center grow sx={style.bg(background).minW(0).h(full)}></center>
                  <center grow sx={style.bg(surface-2).minW(0).h(full)}></center>
                  <center grow sx={style.bg(primary).minW(0).h(full)}></center>
                  <center grow sx={style.bg(accent).minW(0).h(full)}></center>
                </row>
                <column grow min-w="0" gap="0">
                  <p sx={style.fontSize(sm).fontWeight(semibold).truncate()}>{row.name}</p>
                  <p sx={style.fontSize(xs).fg(muted-foreground).truncate()}>{row.meta}</p>
                </column>
                <row gap="1" align="center" shrink="0" nowrap>
                  <button
                    type="button"
                    size="sm"
                    variant="ghost"
                    tone="muted"
                    popovertarget={row.menuId}
                    theme-menu
                    theme-id={row.id}
                    aria-label="Theme menu"
                    aria-haspopup="menu"
                    on:click={onMenuOpen}
                  >
                    <icon name="ellipsis" size="sm"></icon>
                  </button>
                  <dropdown id={row.menuId} placement="bottom-end" on:sail-menu-select={onRowMenu}>
                    <menu-item value="open">Open</menu-item>
                    <menu-item value="remix">Remix</menu-item>
                    <menu-item value="delete">Delete</menu-item>
                  </dropdown>
                </row>
              </row>
            </for>
          </if>

          <if {hasCatalogRows()}>
            <p
              list-group="catalog"
              sx={style.fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide).fg(muted-foreground).padX(2).padT(4).padB(1)}
            >
              Catalog
            </p>
            <for {catalogRows()} as="row" key={row.key}>
              <row
                theme-row
                theme-id={row.id}
                theme-scope="catalog"
                selected={row.selected ? 'true' : 'false'}
                aria-current={row.selected ? 'true' : null}
                gap="3"
                align="center"
                nowrap
                sx={style.pad(2).radius(md).cursor(pointer).borderWidth(1).border(transparent).when({ hover: style.bg(hover-bg) }).when({ current: style.bg(selected-bg).border(selected-border) })}
              >
                <row
                  theme-swatches
                  theme={row.previewId}
                  mode={row.paintMode}
                  shrink="0"
                  gap="0"
                  nowrap
                  sx={style.w(14).h(7).minW(14).radius(sm).overflow(hidden).borderWidth(1).border(border-subtle)}
                  aria-hidden="true"
                >
                  <center grow sx={style.bg(background).minW(0).h(full)}></center>
                  <center grow sx={style.bg(surface-2).minW(0).h(full)}></center>
                  <center grow sx={style.bg(primary).minW(0).h(full)}></center>
                  <center grow sx={style.bg(accent).minW(0).h(full)}></center>
                </row>
                <column grow min-w="0" gap="0">
                  <p sx={style.fontSize(sm).fontWeight(semibold).truncate()}>{row.name}</p>
                  <p sx={style.fontSize(xs).fg(muted-foreground).truncate()}>{row.meta}</p>
                </column>
                <badge tone="muted" size="sm" variant="outline" pill shrink="0">Catalog</badge>
                <row gap="1" align="center" shrink="0" nowrap>
                  <button
                    type="button"
                    size="sm"
                    variant="outline"
                    tone="primary"
                    theme-add-project
                    theme-id={row.id}
                    aria-label="Add to project"
                    on:click={onAddToProject}
                  >
                    Add
                  </button>
                  <button
                    type="button"
                    size="sm"
                    variant="ghost"
                    tone="muted"
                    popovertarget={row.menuId}
                    theme-menu
                    theme-id={row.id}
                    aria-label="Theme menu"
                    aria-haspopup="menu"
                    on:click={onMenuOpen}
                  >
                    <icon name="ellipsis" size="sm"></icon>
                  </button>
                  <dropdown id={row.menuId} placement="bottom-end" on:sail-menu-select={onRowMenu}>
                    <menu-item value="enable">Add to project</menu-item>
                    <menu-item value="open">Open</menu-item>
                    <menu-item value="duplicate">Duplicate</menu-item>
                  </dropdown>
                </row>
              </row>
            </for>
          </if>
        </column>
      </scroll-area>
    </column>

    <!-- PREVIEW -->
    <scroll-area
      themes-preview
      theme-preview-pane
      grow
      min-w="0"
      sx={style.minH(0).basis(0).h(full).minW(0).bg(background)}
    >
      <column gap="6" sx={style.padX(8).padT(6).padB(10).maxW(panel-max-w-lg)}>
        <if {!hasSelected()}>
          <empty-state variant="placeholder" size="sm" w="full" themes-preview-empty>
            <strong slot="title">Nothing selected</strong>
            Pick a theme in the library to preview its surfaces and controls.
          </empty-state>
        </if>

        <if {hasSelected()}>
          <column preview-head gap="3">
            <row gap="2" align="center">
              <p sx={style.fontFamily(display).fontSize(2xl).fontWeight(medium).tracking(tight)}>
                {previewTitle()}
              </p>
              <if {previewStatusLabel()}>
                <badge tone={previewStatusTone()} size="sm" variant="solid" pill>
                  {previewStatusLabel()}
                </badge>
              </if>
              <badge tone="muted" size="sm" variant="outline" pill modes-chip>
                {previewModesLabel()}
              </badge>
            </row>
            <p sx={style.fontSize(sm).fg(muted-foreground).leading(relaxed)}>{previewDesc()}</p>
            <p sx={style.fontSize(xs).fg(muted-foreground)}>{previewMeta()}</p>

            <!-- Mode: one sun/moon switch only (no Light/Dark button segments) -->
            <if {canToggleMode()}>
              <row preview-mode dual-mode gap="2" align="center" nowrap aria-label="Theme mode">
                <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground)}>Mode</p>
                <icon name="sun" size="sm" label="Light" shrink="0" sx={style.fg(muted-foreground)}></icon>
                <switch
                  size="sm"
                  tone="muted"
                  mode-switch
                  checked={modeDarkOn()}
                  aria-label={modeDarkOn() ? 'Switch to light mode' : 'Switch to dark mode'}
                  on:change={onPreviewModeSwitch}
                ></switch>
                <icon name="moon" size="sm" label="Dark" shrink="0" sx={style.fg(muted-foreground)}></icon>
              </row>
            </if>
            <if {!canToggleMode()}>
              <p mode-fixed sx={style.fontSize(xs).fg(muted-foreground)}>
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

          <!-- Color story -->
          <column color-story gap="3">
            <p sx={style.fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide).fg(muted-foreground)}>
              Color story
            </p>
            <row
              theme={previewPaintId()}
              mode={previewModeId()}
              gap="2"
              color-story-grid
            >
              <column grow gap="1" min-w="0">
                <center sx={style.h(16).radius(md).bg(primary).borderWidth(1).border(border-subtle)}></center>
                <p sx={style.fontSize(xs).fontWeight(medium)}>Primary</p>
              </column>
              <column grow gap="1" min-w="0">
                <center sx={style.h(16).radius(md).bg(secondary).borderWidth(1).border(border-subtle)}></center>
                <p sx={style.fontSize(xs).fontWeight(medium)}>Secondary</p>
              </column>
              <column grow gap="1" min-w="0">
                <center sx={style.h(16).radius(md).bg(accent).borderWidth(1).border(border-subtle)}></center>
                <p sx={style.fontSize(xs).fontWeight(medium)}>Accent</p>
              </column>
              <column grow gap="1" min-w="0">
                <center sx={style.h(16).radius(md).bg(background).borderWidth(1).border(border-subtle)}></center>
                <p sx={style.fontSize(xs).fontWeight(medium)}>Background</p>
              </column>
              <column grow gap="1" min-w="0">
                <center sx={style.h(16).radius(md).bg(surface-1).borderWidth(1).border(border-subtle)}></center>
                <p sx={style.fontSize(xs).fontWeight(medium)}>Surface</p>
              </column>
            </row>
          </column>

          <!-- Type + radius under scoped theme -->
          <column
            type-story
            theme={previewPaintId()}
            mode={previewModeId()}
            gap="2"
            sx={style.pad(4).radius(lg).borderWidth(1).border(border-subtle).bg(surface-1).fg(foreground)}
          >
            <p sx={style.fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide).fg(muted-foreground)}>
              Type · corners
            </p>
            <p sx={style.fontFamily(display).fontSize(xl).fontWeight(semibold).tracking(tight)}>
              Display / heading sample
            </p>
            <p sx={style.fontFamily(body).fontSize(sm).leading(relaxed)}>
              Body — the quick brown fox jumps over the lazy dog. 0123456789
            </p>
            <p sx={style.fontFamily(mono).fontSize(xs)}>
              mono — const theme = "preview"
            </p>
            <!-- Outline tiles so radius is legible; solid fills read as one blob when steps are close. -->
            <row gap="2" align="end" radius-chips>
              <column gap="1" align="center">
                <center sx={style.w(8).h(8).bg(transparent).borderWidth(2).border(primary).radius(sm)} aria-hidden="true"></center>
                <p sx={style.fontSize(xs).fg(muted-foreground)}>sm</p>
              </column>
              <column gap="1" align="center">
                <center sx={style.w(8).h(8).bg(transparent).borderWidth(2).border(primary).radius(md)} aria-hidden="true"></center>
                <p sx={style.fontSize(xs).fg(muted-foreground)}>md</p>
              </column>
              <column gap="1" align="center">
                <center sx={style.w(8).h(8).bg(transparent).borderWidth(2).border(primary).radius(lg)} aria-hidden="true"></center>
                <p sx={style.fontSize(xs).fg(muted-foreground)}>lg</p>
              </column>
              <column gap="1" align="center">
                <center sx={style.w(8).h(8).bg(transparent).borderWidth(2).border(primary).radius(xl)} aria-hidden="true"></center>
                <p sx={style.fontSize(xs).fg(muted-foreground)}>xl</p>
              </column>
            </row>
          </column>

          <!-- Live chrome specimen under scoped theme -->
          <column preview-chrome gap="2">
            <p sx={style.fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide).fg(muted-foreground)}>
              Live chrome
            </p>
            <column
              theme={previewPaintId()}
              mode={previewModeId()}
              preview-specimen
              gap="0"
              sx={style.radius(lg).borderWidth(1).border(border).overflow(hidden).minW(0).shadow(md).bg(background).fg(foreground)}
            >
              <row
                align="center"
                gap="2"
                nowrap
                justify="between"
                sx={style.padX(4).h(9).bg(surface-1).borderBWidth(1).borderB(border-subtle)}
              >
                <p sx={style.fontSize(sm).fontWeight(semibold).truncate()}>{previewTitle()}</p>
                <badge tone="primary" size="sm" variant="solid" pill>Preview</badge>
              </row>
              <column gap="4" sx={style.pad(5).bg(background)}>
                <card sx={style.pad(4)}>
                  <column gap="3">
                    <p sx={style.fontSize(md).fontWeight(semibold).fontFamily(heading)}>Sample card</p>
                    <p sx={style.fontSize(sm).fg(muted-foreground).leading(relaxed).fontFamily(body)}>
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
                    <input type="search" size="sm" placeholder="Search field" aria-label="Sample search" />
                  </column>
                </card>
              </column>
            </column>
          </column>
        </if>
      </column>
    </scroll-area>
  </row>

  <!-- Ephemeral notes: fixed overlay — does not reflow master–detail -->
  <toast-region position="bottom-end" themes-toast-region>
    <for {toastItems()} as="t" key={t.key}>
      <toast tone={t.tone} themes-toast on:sail-toast-dismiss={onToastDismiss}>
        {t.message}
      </toast>
    </for>
  </toast-region>
</column>
