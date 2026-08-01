<!--
  <edit-surface> — editor tabs + view switcher + work panels + structured-edit rail.
  Ban-clean: no div/span. Split panes absent as L2 → fixed asymmetric widths (absorption).
-->
<controller>
import { createEditSurface } from './edit-surface-vm'
export default controller(() => {
  const vm = createEditSurface()
  return {
    stateCards: vm.stateCards,
    structOps: vm.structOps,
    sourceText: vm.sourceText,
    sourceStatus: vm.sourceStatus,
    livePreviewSrc: vm.livePreviewSrc,
    hasLivePreview: vm.hasLivePreview,
    auditSrc: vm.auditSrc,
    hasAudit: vm.hasAudit,
    editError: vm.editError,
    hasEditError: vm.hasEditError,
    checkRows: vm.checkRows,
    hasChecks: vm.hasChecks,
    checksLabel: vm.checksLabel,
    hoverLabel: vm.hoverLabel,
    hasHover: vm.hasHover,
    sourceScrollLine: vm.sourceScrollLine,
    sourceDirty: vm.sourceDirty,
    liveRouteLabel: vm.liveRouteLabel,
    docTabs: vm.docTabs,
    hasOpenDoc: vm.hasOpenDoc,
    activePathLabel: vm.activePathLabel,
    activeView: vm.activeView,
    isPreview: vm.isPreview,
    isSource: vm.isSource,
    isState: vm.isState,
    isLayout: vm.isLayout,
    isData: vm.isData,
    isA11y: vm.isA11y,
    stateJson: vm.stateJson,
    dataJson: vm.dataJson,
    layoutRows: vm.layoutRows,
    a11yRows: vm.a11yRows,
    onViewChange: (e: Event) => vm.onViewChange(e),
    onSourceSailChange: (e: Event) => vm.onSourceSailChange(e),
    onSaveSource: () => vm.onSaveSource(),
    onPreviewClick: (e: Event) => vm.onPreviewClick(e),
    onPreviewMove: (e: Event) => vm.onPreviewMove(e),
    onPreviewLeave: () => vm.onPreviewLeave(),
    onLayoutClick: (e: Event) => vm.onLayoutClick(e),
    onSignalWrite: (e: Event) => vm.onSignalWrite(e),
    onTabClick: (e: Event) => vm.onTabClick(e),
    onTabPointer: (e: Event) => vm.onTabPointer(e),
  }
})
</controller>

<row
  data-surface="edit"
  data-edit-surface
  gap="0"
  grow
  nowrap
  align="stretch"
  sx={style.minH(0).basis(0).h(full).minW(0).overflow(hidden)}
>
  <edit-explorer shrink="0" sx={style.minH(0).h(full).overflow(hidden)} />

  <column grow min-w="0" gap="0" sx={style.minH(0).h(full).basis(0).minW(0).overflow(hidden)}>
    <row
      data-studio-tabs
      data-edit-tabs
      gap="0"
      shrink="0"
      align="stretch"
      nowrap
      on:click={onTabClick}
      on:mouseover={onTabPointer}
      on:mouseleave={onTabPointer}
      sx={style.h(8).borderBWidth(1).borderB(border).bg(surface-1).overflow(auto).minW(0)}
    >
      <for {docTabs()} as="tab" key={tab.key}>
        <row
          data-tab-id={tab.id}
          data-active={tab.active ? 'true' : 'false'}
          data-tab-select
          aria-current={tab.active ? 'page' : undefined}
          gap="1"
          align="center"
          nowrap
          shrink="0"
          sx={style.h(full).padX(3).borderRWidth(1).borderR(border-subtle).borderTWidth(2).borderT(transparent).bg(surface-1).fg(muted-foreground).fontSize(sm).when({ hover: style.bg(hover-bg).fg(foreground) }).when({ current: style.bg(surface-0).fg(foreground).borderT(primary) })}
        >
          <icon name="file-code-2" size="sm"></icon>
          <p sx={style.truncate()}>{tab.label}</p>
          <if {tab.dirtyDot}>
            <p data-dirty-dot sx={style.fontSize(xs).fg(warning)}>{tab.dirtyDot}</p>
          </if>
          <if {tab.showClose}>
            <button type="button" data-tab-close size="sm" variant="ghost" aria-label="Close tab" sx={style.w(5).h(5)}>×</button>
          </if>
        </row>
      </for>
      <row grow min-w="0"></row>
      <row gap="2" align="center" shrink="0" sx={style.padX(2)}>
        <p sx={style.fontSize(xs).fg(muted-foreground)}>{liveRouteLabel()}</p>
        <button type="button" size="sm" tone="primary" data-save-source title="Save (Ctrl/Cmd+S)" on:click={onSaveSource}>
          Save
        </button>
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
      <column grow gap="0" sx={style.minH(0).basis(0).maxH(panel-max-h-lg).overflow(hidden)}>
      <row
        data-view-switcher
        shrink="0"
        align="center"
        sx={style.h(8).padX(2).borderBWidth(1).borderB(border-subtle).bg(surface-0).minW(0)}
      >
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

      <row grow gap="0" nowrap align="stretch" sx={style.minH(0).basis(0).maxH(full).overflow(hidden)}>
        <column grow min-w="0" gap="0" sx={style.minH(0).basis(0).maxH(full).overflow(hidden).bg(surface-0)}>
          <!-- Preview: static render + region-overlay (+ live iframe when available) -->
          <if {isPreview()}>
            <column data-view-panel="preview" grow gap="0" sx={style.minH(0).maxH(full).overflow(hidden)}>
              <row gap="2" align="center" shrink="0" sx={style.h(7).padX(3).borderBWidth(1).borderB(border-subtle)}>
                <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)} data-provenance>
                  Static render
                </p>
                <if {hasHover()}>
                  <p data-region-tooltip sx={style.fontSize(xs).fg(primary)}>{hoverLabel()}</p>
                </if>
              </row>
              <scroll-area grow sx={style.minH(0).pad(3).bg(background)}>
                <if {hasAudit()}>
                  <column data-region-overlay gap="0" sx={style.position(relative).w(full).maxW(panel-max-w-lg)}>
                    <img
                      data-preview-img
                      src={auditSrc()}
                      alt="Static preview"
                      on:click={onPreviewClick}
                      on:mousemove={onPreviewMove}
                      on:mouseleave={onPreviewLeave}
                      sx={style.w(full).cursor(crosshair).radius(md)}
                    />
                  </column>
                </if>
                <if {!hasAudit()}>
                  <empty-state variant="placeholder" size="sm" w="full" sx={style.borderWidth(1).border(border-subtle).radius(md).pad(3)}>
                    <strong slot="title">Rendering preview…</strong>
                    Static frame loads from SailRender regions.
                  </empty-state>
                </if>
              </scroll-area>
              <if {hasLivePreview()}>
                <column shrink="0" gap="0" sx={style.h(48).borderTWidth(1).borderT(border).overflow(hidden)}>
                  <iframe
                    data-live-preview
                    title="Live app preview"
                    src={livePreviewSrc()}
                    sx={style.w(full).h(full).minH(0).borderWidth(0).bg(background)}
                  ></iframe>
                </column>
              </if>
            </column>
          </if>

          <!-- Source CodeMirror -->
          <if {isSource()}>
            <column
              data-view-panel="source"
              data-cm-mode="sail"
              grow
              gap="0"
              sx={style.minH(0).maxH(full).overflow(hidden)}
            >
              <row gap="2" align="center" justify="between" shrink="0" sx={style.h(7).padX(3).borderBWidth(1).borderB(border-subtle)}>
                <p sx={style.fontSize(xs).fg(muted-foreground).fontFamily(mono).truncate()}>
                  {sourceStatus()}
                  <if {checksLabel()}> · {checksLabel()}</if>
                  <if {sourceScrollLine()}> · L{sourceScrollLine()}</if>
                </p>
                <if {sourceDirty()}>
                  <p sx={style.fontSize(xs).fg(warning)}>Unsaved</p>
                </if>
              </row>
              <if {hasChecks()}>
                <column data-source-gutter gap="0" shrink="0" sx={style.padX(3).padY(1).fontSize(xs).fontFamily(mono).bg(surface-2).maxH(24).overflow(auto)}>
                  <for {checkRows()} as="ck" key={ck.key}>
                    <p data-check-line={ck.line} data-severity={ck.severity} sx={style.fg(destructive).h(6)}>
                      L{ck.line} · {ck.severity}: {ck.message}
                    </p>
                  </for>
                </column>
              </if>
              <column grow gap="0" sx={style.minH(0).basis(0).overflow(hidden)}>
                <sail-code-editor
                  data-source-editor
                  language="sail"
                  size="sm"
                  chrome="flush"
                  value={sourceText()}
                  on:sail-change={onSourceSailChange}
                  aria-label="Sail source"
                  sx={style.w(full).h(full).minH(0)}
                ></sail-code-editor>
              </column>
            </column>
          </if>

          <!-- State: signal cards + sail-json-view -->
          <if {isState()}>
            <scroll-area data-view-panel="state" grow sx={style.minH(0).maxH(full).pad(3)}>
              <column gap="3">
                <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
                  Signals
                </p>
                <for {stateCards()} as="card" key={card.key}>
                  <column
                    data-state-card
                    data-binding={card.name}
                    data-ssr-tier={card.ssrTier}
                    gap="2"
                    sx={style.pad(3).borderWidth(1).border(border-subtle).radius(md).bg(surface-1)}
                  >
                    <row gap="2" align="center">
                      <strong sx={style.fontSize(sm)}>{card.name}</strong>
                      <badge data-tier-badge size="sm" tone="muted">{card.tierLabel}</badge>
                    </row>
                    <p sx={style.fontSize(xs).fg(muted-foreground)}>{card.tierWhy}</p>
                    <p sx={style.fontSize(xs)}>{card.readersLabel} · {card.writersLabel}</p>
                    <row gap="2" align="center">
                      <input
                        type="text"
                        size="sm"
                        placeholder="Write value"
                        value={card.value}
                        aria-label={`Write ${card.name}`}
                        data-signal-input
                        sx={style.w(full)}
                      />
                      <button type="button" size="sm" tone="primary" data-signal-write on:click={onSignalWrite}>
                        Write
                      </button>
                    </row>
                  </column>
                </for>
                <if {!stateCards().length}>
                  <p sx={style.fg(muted-foreground).fontSize(xs)}>No state in this file.</p>
                </if>
                <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
                  Graph
                </p>
                <sail-json-view data={stateJson()} depth={2} sx={style.w(full)}></sail-json-view>
              </column>
            </scroll-area>
          </if>

          <!-- Layout: taffy boxes → source span -->
          <if {isLayout()}>
            <scroll-area data-view-panel="layout" grow sx={style.minH(0).maxH(panel-max-h-md).overflow(auto).pad(2)} on:click={onLayoutClick}>
              <column gap="0" sx={style.fontFamily(mono).fontSize(xs)}>
                <for {layoutRows()} as="row" key={row.key}>
                  <button
                    type="button"
                    size="sm"
                    variant={row.selected ? 'soft' : 'ghost'}
                    tone={row.selected ? 'primary' : 'muted'}
                    data-layout-id={row.id}
                    sx={style.w(full).h(7).padX(2).radius(none).textAlign(start).when({ hover: style.bg(hover-bg) })}
                  >
                    {row.line}
                  </button>
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

          <!-- Data: sail-json-view -->
          <if {isData()}>
            <scroll-area data-view-panel="data" grow sx={style.minH(0).maxH(full).pad(3)}>
              <column gap="2">
                <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
                  Query / data keys
                </p>
                <sail-json-view data={dataJson()} depth={2} sx={style.w(full)}></sail-json-view>
              </column>
            </scroll-area>
          </if>

          <!-- A11y checklist -->
          <if {isA11y()}>
            <scroll-area data-view-panel="a11y" grow sx={style.minH(0).maxH(full).pad(3)}>
              <column gap="2">
                <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
                  Contract checklist
                </p>
                <for {a11yRows()} as="row" key={row.key}>
                  <column
                    data-a11y-row
                    data-status={row.status}
                    gap="1"
                    sx={style.pad(2).borderWidth(1).border(border-subtle).radius(sm)}
                  >
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

        <!-- Structured-edit rail (source / preview) -->
        <if {isSource() || isPreview()}>
          <scroll-area
            data-edit-rail
            shrink="0"
            sx={style.w(52).minH(0).borderLWidth(1).borderL(border).bg(surface-1)}
          >
            <column gap="0" sx={style.fontSize(sm)}>
              <row align="center" sx={style.h(7).padX(3).borderBWidth(1).borderB(border-subtle)}>
                <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
                  Inspector
                </p>
              </row>
              <column gap="2" data-structured-edit sx={style.pad(3)}>
                <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
                  Structured edit
                </p>
                <column gap="1">
                  <for {structOps()} as="op" key={op.key}>
                    <button type="button" size="sm" variant="ghost" tone="muted" disabled={op.disabled} data-struct-op={op.id}>
                      {op.label}
                    </button>
                  </for>
                </column>
              </column>
            </column>
          </scroll-area>
        </if>
      </row>
      </column>
    </if>
  </column>
</row>
