<!--
  <edit-surface> — file explorer + doc tabs + source editor. Live app is the host.
-->
<scope>
  style surfaceRoot = { box: grow(1).minH(0).basis(0).h(full).minW(0).overflow(hidden) }
  style explorerHost = { box: shrink(0).minH(0).h(full).overflow(hidden) }
  style mainCol = { box: grow(1).minW(0).minH(0).h(full).basis(0).overflow(hidden) }
  style tabsStrip = { box: shrink(0).h(8).minW(0).overflow(hidden), paint: borderBWidth(1).borderB(border).bg(surface-1) }
  style tabList = { box: grow(1).minW(0).h(full).basis(0).overflowX(auto).overflowY(hidden) }
  style docTab = {
    box: shrink(0).h(full).padX(3).maxW(48),
    paint: borderRWidth(1).borderR(border-subtle).borderTWidth(2).borderT(transparent).bg(surface-1).fg(muted-foreground).cursor(pointer),
    when: {
      hover: { paint: bg(hover-bg).fg(foreground) },
      current: { paint: bg(surface-0).fg(foreground).borderT(primary) },
    },
  }
  style tabLabel = { box: grow(1).minW(0), text: fontSize(sm).truncate() }
  style dirtyDot = { paint: fg(warning), text: fontSize(xs) }
  style tabClose = { box: w(5).h(5) }
  style tabsTrail = { box: shrink(0).padX(2).h(full), paint: borderLWidth(1).borderL(border-subtle) }
  style routeLabel = { box: maxW(40), paint: fg(muted-foreground), text: fontSize(xs).truncate() }
  style errorBand = { box: padX(3).padY(1), paint: bg(surface-1).borderBWidth(1).borderB(border) }
  style errorText = { paint: fg(destructive), text: fontSize(xs) }
  style emptyHost = { box: grow(1).minH(0).h(full).overflow(hidden), paint: bg(surface-0) }
  style emptyFrame = { box: w(full).pad(4), paint: borderWidth(1).border(border-subtle).radius(md) }
  style sourceHost = { box: grow(1).minH(0).basis(0).h(full).overflow(hidden), paint: bg(surface-0) }
  style emptyInset = { box: w(full).pad(3).m(3), paint: borderWidth(1).border(border-subtle).radius(md) }
  style fillCol = { box: grow(1).minH(0).basis(0).h(full).overflow(hidden) }
  style editorFill = { box: w(full).h(full).minH(0) }
</scope>

<controller>
import type { Action } from '@ship/sail-runtime'
import { createEditSurface } from '../compositions/edit-surface/edit-surface-vm'
export default controller(() => {
  const vm = createEditSurface()
  const captureEditor: Action<void> = (node) => {
    vm.editorEl.set(node as HTMLElement)
    return { destroy: () => vm.editorEl.set(null) }
  }
  return {
    sourceText: vm.sourceText,
    sourceStatus: vm.sourceStatus,
    sourceScrollLine: vm.sourceScrollLine,
    sourceDirty: vm.sourceDirty,
    isSourceReady: vm.isSourceReady,
    isSourceEmpty: vm.isSourceEmpty,
    isSourceLoading: vm.isSourceLoading,
    liveRouteLabel: vm.liveRouteLabel,
    docTabs: vm.docTabs,
    hasOpenDoc: vm.hasOpenDoc,
    editError: vm.editError,
    hasEditError: vm.hasEditError,
    captureEditor,
    opsUrl: vm.opsUrl,
    opsPath: vm.opsPath,
    onSourceSailChange: (detail: { value?: string }) => vm.onSourceSailChange(detail),
    onSaveSource: () => vm.onSaveSource(),
    onTabClick: (e: Event) => vm.onTabClick(e),
    onTabMiddle: (e: Event) => vm.onTabMiddle(e),
  }
})
</controller>

<row surface="edit" edit-surface gap="0" nowrap align="stretch" style(surfaceRoot)>
  <edit-explorer style(explorerHost)></edit-explorer>

  <column gap="0" style(mainCol)>
    <row studio-tabs edit-tabs gap="0" align="stretch" nowrap style(tabsStrip)>
      <row edit-tab-list label="Open documents" gap="0" align="stretch" nowrap on:click={onTabClick} on:mousedown={onTabMiddle} style(tabList)>
        <for {docTabs() as tab} key={tab.key}>
          <row
            tab-id={tab.id}
            active={tab.active ? 'true' : 'false'}
            tab-select
            selected={tab.active ? 'true' : 'false'}
            current={tab.active ? 'page' : undefined}
            title={tab.id}
            gap="1"
            align="center"
            nowrap
            style(docTab)
          >
            <icon name="file-code-2" size="sm"></icon>
            <p style(tabLabel)>{tab.label}</p>
            <if {tab.dirtyDot}>
              <p dirty-dot label="Unsaved" style(dirtyDot)>
                {tab.dirtyDot}
              </p>
            </if>
            <if {tab.showClose}>
              <button
                type="button"
                tab-close
                size="sm"
                variant="ghost"
                label="Close tab"
                style(tabClose)
              >
                ×
              </button>
            </if>
          </row>
        </for>
      </row>
      <row gap="2" align="center" style(tabsTrail)>
        <p style(routeLabel)>
          {liveRouteLabel()}
        </p>
        <button
          type="button"
          size="sm"
          tone="primary"
          save-source
          dirty={sourceDirty() ? 'true' : 'false'}
          title="Save (Ctrl/Cmd+S)"
          on:click={onSaveSource}
        >
          Save
        </button>
      </row>
    </row>

    <if {hasEditError()}>
      <row edit-error align="center" gap="2" style(errorBand)>
        <p style(errorText)>{editError()}</p>
      </row>
    </if>

    <if {!hasOpenDoc()}>
      <column align="center" justify="center" style(emptyHost)>
        <empty-state variant="placeholder" size="sm" edit-empty style(emptyFrame)>
          <strong slot="title">Open a file</strong>
          Pick a file from the Explorer.
        </empty-state>
      </column>
    </if>

    <if {hasOpenDoc()}>
      <column source-host cm-mode="sail" gap="0" style(sourceHost)>
        <if {isSourceLoading()}>
          <empty-state source-loading variant="placeholder" size="sm" style(emptyInset)>
            <strong slot="title">Loading…</strong>
            {sourceStatus()}
          </empty-state>
        </if>

        <if {isSourceEmpty()}>
          <empty-state source-empty variant="placeholder" size="sm" style(emptyInset)>
            <strong slot="title">No source loaded</strong>
            Open a file from the Explorer. Non-Sail text needs a readable ops path.
          </empty-state>
        </if>

        <if {isSourceReady()}>
          <column source-ready gap="0" style(fillCol)>
            <sail-code-editor
              source-editor
              language="sail"
              size="sm"
              chrome="flush"
              line-numbers
              ops-url={opsUrl()}
              ops-path={opsPath()}
              value={sourceText()}
              use:captureEditor
              on-change={onSourceSailChange}
              on-save={onSaveSource}
              label="Source editor"
              style(editorFill)
            ></sail-code-editor>
          </column>
        </if>
      </column>
    </if>
  </column>
</row>
