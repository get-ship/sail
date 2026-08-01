<!--
  <edit-surface> — tabs + view switcher + preview/source panels.
  Structured-edit rail deferred (ops not wired). Ban-clean: no div/span.
-->
<controller>
import { createEditSurface } from './edit-surface-vm'
export default controller(() => {
  const vm = createEditSurface()
  return {
    stateCards: vm.stateCards, sourceText: vm.sourceText, sourceStatus: vm.sourceStatus,
    livePreviewSrc: vm.livePreviewSrc, hasLivePreview: vm.hasLivePreview, auditSrc: vm.auditSrc,
    hasAudit: vm.hasAudit, isPreviewReady: vm.isPreviewReady, isPreviewLoading: vm.isPreviewLoading,
    isPreviewEmpty: vm.isPreviewEmpty, previewError: vm.previewError,
    hasPreviewError: vm.hasPreviewError, previewProvenance: vm.previewProvenance,
    previewRegionsLabel: vm.previewRegionsLabel, hasPreviewRegions: vm.hasPreviewRegions,
    editError: vm.editError, hasEditError: vm.hasEditError,
    checkRows: vm.checkRows, hasChecks: vm.hasChecks, checksLabel: vm.checksLabel,
    hoverLabel: vm.hoverLabel, hasHover: vm.hasHover, sourceScrollLine: vm.sourceScrollLine,
    sourceDirty: vm.sourceDirty, isSourceReady: vm.isSourceReady, isSourceEmpty: vm.isSourceEmpty,
    liveRouteLabel: vm.liveRouteLabel, docTabs: vm.docTabs,
    hasOpenDoc: vm.hasOpenDoc, activePathLabel: vm.activePathLabel, activeView: vm.activeView,
    isPreview: vm.isPreview, isSource: vm.isSource, isState: vm.isState, isLayout: vm.isLayout,
    isData: vm.isData, isA11y: vm.isA11y, stateJson: vm.stateJson, dataJson: vm.dataJson,
    layoutRows: vm.layoutRows, a11yRows: vm.a11yRows,
    onViewChange: (e: Event) => vm.onViewChange(e),
    onSourceSailChange: (e: Event) => vm.onSourceSailChange(e),
    onSaveSource: () => vm.onSaveSource(),
    onRetryPreview: () => vm.onRetryPreview(),
    onPreviewClick: (e: Event) => vm.onPreviewClick(e),
    onPreviewMove: (e: Event) => vm.onPreviewMove(e),
    onPreviewLeave: (e: Event) => vm.onPreviewLeave(e),
    onLayoutClick: (e: Event) => vm.onLayoutClick(e),
    onSignalWrite: (e: Event) => vm.onSignalWrite(e),
    onTabClick: (e: Event) => vm.onTabClick(e),
    onTabMiddle: (e: Event) => vm.onTabMiddle(e),
  }
})
</controller>

<row data-surface="edit" data-edit-surface gap="0" grow nowrap align="stretch" sx={style.minH(0).basis(0).h(full).minW(0).overflow(hidden)}>
  <edit-explorer shrink="0" sx={style.minH(0).h(full).overflow(hidden)} />

  <column grow min-w="0" gap="0" sx={style.minH(0).h(full).basis(0).minW(0).overflow(hidden)}>
    <!-- Doc tabs: L1 tabs lack dirty/close/middle-click → custom strip (absorption: studio-tabs). -->
    <row data-studio-tabs data-edit-tabs gap="0" shrink="0" align="stretch" nowrap sx={style.h(8).borderBWidth(1).borderB(border).bg(surface-1).minW(0).overflow(hidden)}>
      <row data-edit-tab-list role="tablist" aria-label="Open documents" grow min-w="0" gap="0" align="stretch" nowrap on:click={onTabClick} on:mousedown={onTabMiddle} sx={style.minW(0).h(full).basis(0).overflowX(auto).overflowY(hidden)}>
        <for {docTabs()} as="tab" key={tab.key}>
          <row data-tab-id={tab.id} data-active={tab.active ? 'true' : 'false'} data-tab-select role="tab" aria-selected={tab.active ? 'true' : 'false'} aria-current={tab.active ? 'page' : undefined} title={tab.id} gap="1" align="center" nowrap shrink="0" sx={style.h(full).padX(3).maxW(48).borderRWidth(1).borderR(border-subtle).borderTWidth(2).borderT(transparent).bg(surface-1).fg(muted-foreground).fontSize(sm).cursor(pointer).when({ hover: style.bg(hover-bg).fg(foreground) }).when({ current: style.bg(surface-0).fg(foreground).borderT(primary) })}>
            <icon name="file-code-2" size="sm"></icon>
            <p grow min-w="0" sx={style.truncate()}>{tab.label}</p>
            <if {tab.dirtyDot}>
              <p data-dirty-dot aria-label="Unsaved" sx={style.fontSize(xs).fg(warning)}>{tab.dirtyDot}</p>
            </if>
            <if {tab.showClose}>
              <button type="button" data-tab-close size="sm" variant="ghost" aria-label="Close tab" sx={style.w(5).h(5)}>×</button>
            </if>
          </row>
        </for>
      </row>
      <row gap="2" align="center" shrink="0" sx={style.padX(2).h(full).borderLWidth(1).borderL(border-subtle)}>
        <p sx={style.fontSize(xs).fg(muted-foreground).maxW(40).truncate()}>{liveRouteLabel()}</p>
        <button type="button" size="sm" tone="primary" data-save-source data-dirty={sourceDirty() ? 'true' : 'false'} title="Save (Ctrl/Cmd+S)" on:click={onSaveSource}>Save</button>
      </row>
    </row>

    <if {hasEditError()}>
      <row data-edit-error align="center" gap="2" sx={style.padX(3).padY(1).bg(surface-1).borderBWidth(1).borderB(border)}>
        <p sx={style.fg(destructive).fontSize(xs)}>{editError()}</p>
      </row>
    </if>

    <if {!hasOpenDoc()}>
      <column grow align="center" justify="center" sx={style.minH(0).maxH(full).overflow(hidden).bg(surface-0)}>
        <empty-state variant="placeholder" size="sm" w="full" data-edit-empty sx={style.borderWidth(1).border(border-subtle).radius(md).pad(4)}>
          <strong slot="title">Open a file from the Explorer</strong>
          Select a Sail source to edit and preview.
        </empty-state>
      </column>
    </if>

    <if {hasOpenDoc()}>
      <column grow gap="0" sx={style.minH(0).basis(0).h(full).maxH(panel-max-h-lg).overflow(hidden)}>
        <row data-view-switcher shrink="0" align="center" sx={style.h(8).padX(2).borderBWidth(1).borderB(border-subtle).bg(surface-0).minW(0)}>
          <tabs active={activeView()} size="sm" on:sail-tab-change={onViewChange} sx={style.minW(0)}>
            <tab-list variant="pill">
              <tab name="preview">Preview</tab>
              <tab name="source">Source</tab>
              <tab name="state">State</tab>
              <tab name="layout">Layout</tab>
              <tab name="data">Data</tab>
              <tab name="a11y">A11y</tab>
            </tab-list>
          </tabs>
        </row>

        <row grow gap="0" nowrap align="stretch" sx={style.minH(0).basis(0).maxH(panel-max-h-lg).overflow(hidden)}>
          <column grow min-w="0" gap="0" sx={style.minH(0).basis(0).maxH(panel-max-h-lg).overflow(hidden).bg(surface-0)}>
            <!-- Preview: static frame + regions. Exclusive ifs (not match) for layout. -->
            <if {isPreview()}>
              <column data-view-panel="preview" data-preview-host grow gap="0" sx={style.minH(0).basis(0).maxH(panel-max-h-lg).overflow(hidden)}>
                <row data-preview-toolbar gap="2" align="center" justify="between" shrink="0" sx={style.h(7).minH(7).padX(3).borderBWidth(1).borderB(border-subtle).bg(surface-0).minW(0)}>
                  <row gap="2" align="center" grow min-w="0" sx={style.minW(0)}>
                    <p data-provenance shrink="0" sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>Static render</p>
                    <p data-preview-provenance sx={style.fontSize(xs).fg(muted-foreground).truncate()}>{previewProvenance()}</p>
                    <if {hasPreviewRegions()}>
                      <p data-preview-regions shrink="0" sx={style.fontSize(xs).fg(muted-foreground)}>· {previewRegionsLabel()}</p>
                    </if>
                    <if {hasHover()}>
                      <p data-region-tooltip sx={style.fontSize(xs).fg(primary).truncate()}>{hoverLabel()}</p>
                    </if>
                  </row>
                  <if {hasPreviewError()}>
                    <button type="button" size="sm" tone="primary" data-preview-retry on:click={onRetryPreview}>Retry</button>
                  </if>
                  <if {isPreviewReady()}>
                    <button type="button" size="sm" variant="ghost" data-preview-refresh on:click={onRetryPreview} title="Re-render static frame">Refresh</button>
                  </if>
                </row>
                <scroll-area data-preview-stage grow sx={style.minH(0).basis(0).pad(3).bg(background)}>
                  <if {isPreviewReady()}>
                    <column data-preview-ready data-region-overlay gap="0" sx={style.position(relative).w(full).maxW(panel-max-w-lg).mx(auto)}>
                      <img data-preview-img src={auditSrc()} alt="Static preview of open document" on:click={onPreviewClick} on:mousemove={onPreviewMove} on:mouseleave={onPreviewLeave} sx={style.w(full).cursor(crosshair).radius(md).borderWidth(1).border(border-subtle)} />
                    </column>
                  </if>
                  <if {hasPreviewError()}>
                    <error-state data-preview-error variant="inline" size="sm" w="full">
                      <strong slot="title">Preview failed</strong>
                      {previewError()}
                      <button slot="action" type="button" size="sm" tone="primary" data-preview-retry on:click={onRetryPreview}>Retry</button>
                    </error-state>
                  </if>
                  <if {isPreviewLoading()}>
                    <empty-state data-preview-loading variant="placeholder" size="sm" w="full" sx={style.borderWidth(1).border(border-subtle).radius(md).pad(3)}>
                      <strong slot="title">Rendering preview…</strong>
                      Static frame from SailRender — hover a box to inspect, click to select source.
                    </empty-state>
                  </if>
                  <if {isPreviewEmpty()}>
                    <empty-state data-preview-empty variant="placeholder" size="sm" w="full" sx={style.borderWidth(1).border(border-subtle).radius(md).pad(3)}>
                      <strong slot="title">No static preview</strong>
                      Open this view on a Sail file to render an audit snapshot with regions.
                    </empty-state>
                  </if>
                </scroll-area>
              </column>
            </if>

            <!-- Source: CodeMirror sail mode — labeled host, ready/empty/dirty, save + scroll -->
            <if {isSource()}>
              <column data-view-panel="source" data-source-host data-cm-mode="sail" grow gap="0" sx={style.minH(0).basis(0).maxH(panel-max-h-lg).overflow(hidden)}>
                <row data-source-toolbar gap="2" align="center" justify="between" shrink="0" sx={style.h(7).minH(7).padX(3).borderBWidth(1).borderB(border-subtle).bg(surface-0).minW(0)}>
                  <row gap="2" align="center" grow min-w="0" sx={style.minW(0)}>
                    <p data-source-label shrink="0" sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>Source</p>
                    <p data-source-status sx={style.fontSize(xs).fg(muted-foreground).fontFamily(mono).truncate()}>{sourceStatus()}</p>
                    <if {hasChecks()}>
                      <p data-source-checks shrink="0" sx={style.fontSize(xs).fg(muted-foreground)}>· {checksLabel()}</p>
                    </if>
                    <if {sourceScrollLine()}>
                      <p data-source-line shrink="0" sx={style.fontSize(xs).fg(primary).fontFamily(mono)}>· L{sourceScrollLine()}</p>
                    </if>
                  </row>
                  <if {sourceDirty()}>
                    <p data-source-dirty sx={style.fontSize(xs).fg(warning)}>Unsaved</p>
                  </if>
                </row>
                <if {isSourceEmpty()}>
                  <empty-state data-source-empty variant="placeholder" size="sm" w="full" sx={style.borderWidth(1).border(border-subtle).radius(md).pad(3).m(3)}>
                    <strong slot="title">No source loaded</strong>
                    Open a Sail file from the Explorer to edit here.
                  </empty-state>
                </if>
                <if {isSourceReady()}>
                  <column data-source-ready grow gap="0" sx={style.minH(0).basis(0).overflow(hidden)}>
                    <if {hasChecks()}>
                      <column data-source-gutter gap="0" shrink="0" sx={style.padX(3).padY(1).fontSize(xs).fontFamily(mono).bg(surface-2).maxH(20).minH(0).overflow(auto)}>
                        <for {checkRows()} as="ck" key={ck.key}>
                          <p data-check-line={ck.line} data-severity={ck.severity} sx={style.fg(destructive).h(5)}>L{ck.line} · {ck.severity}: {ck.message}</p>
                        </for>
                      </column>
                    </if>
                    <column grow gap="0" sx={style.minH(0).basis(0).overflow(hidden)}>
                      <sail-code-editor data-source-editor language="sail" size="sm" chrome="flush" line-numbers value={sourceText()} on:sail-change={onSourceSailChange} on:sail-save={onSaveSource} aria-label="Sail source" sx={style.w(full).h(full).minH(0)}></sail-code-editor>
                    </column>
                  </column>
                </if>
              </column>
            </if>

            <!-- State stub: compiler control-flow scroller bug blocks real panel. -->
            <if {isState()}>
              <scroll-area data-view-panel="state" grow sx={style.minH(0).basis(0).maxH(panel-max-h-lg).overflow(auto).pad(3)}>
                <p sx={style.fg(muted-foreground).fontSize(sm)}>State view temporarily stubbed (compiler model issue).</p>
              </scroll-area>
            </if>

            <if {isLayout()}>
              <scroll-area data-view-panel="layout" grow sx={style.minH(0).basis(0).maxH(panel-max-h-lg).overflow(auto).pad(2)} on:click={onLayoutClick}>
                <column gap="0" sx={style.fontFamily(mono).fontSize(xs)}>
                  <for {layoutRows()} as="row" key={row.key}>
                    <button type="button" size="sm" variant={row.selected ? 'soft' : 'ghost'} tone={row.selected ? 'primary' : 'muted'} data-layout-id={row.id} sx={style.w(full).h(7).padX(2).radius(none).textAlign(start).when({ hover: style.bg(hover-bg) })}>{row.line}</button>
                  </for>
                  <if {!layoutRows().length}>
                    <empty-state variant="placeholder" size="sm" w="full" sx={style.borderWidth(1).border(border-subtle).radius(md).pad(3)}>
                      <strong slot="title">No layout boxes</strong>
                      Open preview once to attach taffy geometry.
                    </empty-state>
                  </if>
                </column>
              </scroll-area>
            </if>

            <if {isData()}>
              <scroll-area data-view-panel="data" grow sx={style.minH(0).basis(0).maxH(panel-max-h-lg).overflow(auto).pad(3)}>
                <column gap="2">
                  <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>Query / data keys</p>
                  <sail-json-view data={dataJson()} depth={2} sx={style.w(full)}></sail-json-view>
                </column>
              </scroll-area>
            </if>

            <if {isA11y()}>
              <scroll-area data-view-panel="a11y" grow sx={style.minH(0).basis(0).maxH(panel-max-h-lg).overflow(auto).pad(3)}>
                <column gap="2">
                  <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>Contract checklist</p>
                  <for {a11yRows()} as="row" key={row.key}>
                    <column data-a11y-row data-status={row.status} gap="1" sx={style.pad(2).borderWidth(1).border(border-subtle).radius(sm)}>
                      <row gap="2" align="center">
                        <strong sx={style.fontSize(sm)}>{row.check}</strong>
                        <badge size="sm" tone={row.tone}>{row.status}</badge>
                      </row>
                      <p sx={style.fontSize(xs).fg(muted-foreground)}>{row.detail}</p>
                    </column>
                  </for>
                </column>
              </scroll-area>
            </if>
          </column>
        </row>

        <!-- Live iframe at h(0) for State postMessage only (dual-pane dead). -->
        <if {hasLivePreview()}>
          <column data-live-preview-host data-preview-visible="false" shrink="0" gap="0" aria-hidden="true" sx={style.h(0).minH(0).maxH(0).overflow(hidden)}>
            <iframe data-live-preview title="Live app bridge (hidden)" src={livePreviewSrc()} sx={style.w(full).h(full).minH(0).borderWidth(0).bg(background)}></iframe>
          </column>
        </if>
      </column>
    </if>
  </column>
</row>
