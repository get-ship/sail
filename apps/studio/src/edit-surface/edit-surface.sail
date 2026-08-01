<!--
  <edit-surface> — editor tabs + source + live preview + inspector rail.
  Panel grammar: header · content. Ban-clean: no div/span.
-->
<controller>
import { createEditSurface } from './edit-surface-vm'
export default controller(() => {
  const vm = createEditSurface()
  return {
    stateCards: vm.stateCards,
    childRows: vm.childRows,
    structOps: vm.structOps,
    sourceText: vm.sourceText,
    sourceStatus: vm.sourceStatus,
    livePreviewSrc: vm.livePreviewSrc,
    hasLivePreview: vm.hasLivePreview,
    auditSrc: vm.auditSrc,
    hasAudit: vm.hasAudit,
    showAudit: vm.showAudit,
    showDebug: vm.showDebug,
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
    onSourceSailChange: (e: Event) => vm.onSourceSailChange(e),
    onSaveSource: () => vm.onSaveSource(),
    onAuditSnapshot: () => vm.onAuditSnapshot(),
    onToggleDebug: (e: Event) => vm.onToggleDebug(e),
    onPreviewClick: (e: Event) => vm.onPreviewClick(e),
    onPreviewMove: (e: Event) => vm.onPreviewMove(e),
    onPreviewLeave: () => vm.onPreviewLeave(),
    onElementClick: (e: Event) => vm.onElementClick(e),
    onHideAudit: () => vm.onHideAudit(),
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
            <button
              type="button"
              data-tab-close
              size="sm"
              variant="ghost"
              aria-label="Close tab"
              sx={style.w(5).h(5)}
            >
              ×
            </button>
          </if>
        </row>
      </for>
      <row grow min-w="0"></row>
      <row gap="2" align="center" shrink="0" sx={style.padX(2)}>
        <p sx={style.fontSize(xs).fg(muted-foreground)}>{liveRouteLabel()}</p>
        <button type="button" size="sm" variant="ghost" tone="muted" data-audit-snapshot on:click={onAuditSnapshot}>
          Audit snapshot
        </button>
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
      <!-- Viewport-relative max (not fixed 720px) so tall displays fill; minH/overflow keep flex chain. -->
      <row grow gap="0" nowrap align="stretch" sx={style.minH(0).basis(0).h(full).maxH(panel-vh-lg).overflow(hidden)}>
        <column
          grow
          min-w="0"
          data-edit-main
          data-view-panel="source"
          data-cm-mode="sail"
          gap="0"
          sx={style.minH(0).basis(0).h(full).overflow(hidden).bg(surface-0)}
        >
          <row
            gap="2"
            align="center"
            justify="between"
            shrink="0"
            sx={style.h(7).padX(3).borderBWidth(1).borderB(border-subtle)}
          >
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
            <column
              data-source-gutter
              gap="0"
              shrink="0"
              sx={style.padX(3).padY(1).fontSize(xs).fontFamily(mono).bg(surface-2).maxH(24).overflow(auto)}
            >
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

        <column
          grow
          min-w="0"
          data-view-panel="preview"
          gap="0"
          sx={style.minH(0).basis(0).h(full).borderLWidth(1).borderL(border).overflow(hidden)}
        >
          <if {hasLivePreview()}>
            <column grow gap="0" sx={style.minH(0).h(full).overflow(hidden)}>
              <iframe
                data-live-preview
                title="Live app preview"
                src={livePreviewSrc()}
                sx={style.w(full).h(full).minH(0).borderWidth(0).bg(background)}
              ></iframe>
            </column>
          </if>
          <if {!hasLivePreview()}>
            <column grow gap="3" sx={style.minH(0).maxH(panel-max-h-md).overflow(auto).pad(4).bg(background)}>
              <card data-preview-skeleton variant="outline" padding="sm" sx={style.w(full).maxW(panel-max-w-md)}>
                <column gap="2">
                  <row gap="2" align="center">
                    <center sx={style.w(8).h(8).radius(md).bg(muted)} aria-hidden="true"></center>
                    <column grow gap="1">
                      <center sx={style.w(40).h(3).bg(muted).radius(sm)} aria-hidden="true"></center>
                      <center sx={style.w(28).h(2).bg(muted).radius(sm)} aria-hidden="true"></center>
                    </column>
                  </row>
                  <center sx={style.w(full).h(24).radius(sm).bg(muted)} aria-hidden="true"></center>
                  <row gap="2">
                    <center sx={style.w(16).h(6).radius(sm).bg(muted)} aria-hidden="true"></center>
                    <center sx={style.w(16).h(6).radius(sm).bg(muted)} aria-hidden="true"></center>
                  </row>
                </column>
              </card>
              <empty-state variant="placeholder" size="sm" w="full" sx={style.borderWidth(1).border(border-subtle).radius(md).pad(3)}>
                <strong slot="title">No live preview</strong>
                This file is not a page the app serves. Use Audit snapshot for a still frame of the layout.
              </empty-state>
            </column>
          </if>
          <if {showAudit()}>
            <column
              data-audit-panel
              gap="2"
              shrink="0"
              sx={style.maxH(64).borderTWidth(1).borderT(border).bg(surface-1).pad(2).overflow(auto)}
            >
              <row gap="2" align="center" justify="between">
                <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)} data-provenance>
                  Audit snapshot
                </p>
                <if {hasHover()}>
                  <p data-region-tooltip sx={style.fontSize(xs).fg(primary)}>{hoverLabel()}</p>
                </if>
                <button type="button" size="sm" variant="ghost" on:click={onHideAudit}>Hide</button>
              </row>
              <if {hasAudit()}>
                <img
                  data-preview-img
                  src={auditSrc()}
                  alt="Headless audit snapshot"
                  on:click={onPreviewClick}
                  on:mousemove={onPreviewMove}
                  on:mouseleave={onPreviewLeave}
                  sx={style.maxW(full).cursor(crosshair).radius(md)}
                />
              </if>
              <if {!hasAudit()}>
                <p sx={style.fontSize(xs).fg(muted-foreground)}>Rendering audit snapshot…</p>
              </if>
            </column>
          </if>
        </column>

        <scroll-area
          data-edit-rail
          shrink="0"
          sx={style.w(56).minH(0).h(full).borderLWidth(1).borderL(border).bg(surface-1)}
        >
          <column gap="0" sx={style.fontSize(sm)}>
            <row
              align="center"
              sx={style.h(7).padX(3).borderBWidth(1).borderB(border-subtle)}
            >
              <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
                Inspector
              </p>
            </row>
            <column gap="0" data-debug-toggle sx={style.padX(3).padY(2)}>
              <row gap="2" align="center">
                <switch
                  size="sm"
                  tone="muted"
                  checked={showDebug()}
                  on:change={onToggleDebug}
                  aria-label="Show debug state"
                ></switch>
                <p sx={style.fontSize(xs).fg(muted-foreground)}>Show debug state</p>
              </row>
            </column>

            <if {showDebug()}>
              <column gap="2" data-anatomy data-debug-state sx={style.pad(3)}>
                <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
                  State
                </p>
                <for {stateCards()} as="card" key={card.key}>
                  <column
                    data-state-card
                    data-binding={card.name}
                    data-ssr-tier={card.ssrTier}
                    gap="1"
                    sx={style.pad(2).borderWidth(1).border(border-subtle).radius(sm)}
                  >
                    <row gap="2" align="center">
                      <strong sx={style.fontSize(sm)}>{card.name}</strong>
                      <p data-tier-badge sx={style.fontSize(xs).fg(muted-foreground)}>{card.tierLabel}</p>
                    </row>
                    <p sx={style.fontSize(xs).fg(muted-foreground)}>{card.tierWhy}</p>
                    <p sx={style.fontSize(xs)}>{card.readersLabel} · {card.writersLabel}</p>
                  </column>
                </for>
                <if {!stateCards().length}>
                  <p sx={style.fg(muted-foreground).fontSize(xs)}>No state in this file.</p>
                </if>
                <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
                  Structured edit
                </p>
                <column gap="1">
                  <for {structOps()} as="op" key={op.key}>
                    <button
                      type="button"
                      size="sm"
                      variant="ghost"
                      tone="muted"
                      disabled={op.disabled}
                      data-struct-op={op.id}
                    >
                      {op.label}
                    </button>
                  </for>
                </column>
              </column>
            </if>
          </column>
        </scroll-area>
      </row>
    </if>
  </column>
</row>
