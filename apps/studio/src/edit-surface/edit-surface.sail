<!--
  <edit-surface> — file explorer + doc tabs + source editor. Live app is the host.
-->
<scope>
  style surfaceRoot = { size(grow).overflow(hidden) }
  style explorerHost = { size(max).overflow(hidden) }
  style mainCol = { size(grow).overflow(hidden) }
  // no preset: min-w lock + edge-only border
  style tabsStrip = { size({ h: 8, min-w: 0 }).overflow(hidden).border({ edge: b, color: border, width: 1 }).bg(surface-1) }
  style tabList = { size(grow).overflow({ x: auto, y: hidden }) }
  // no preset: size lock + edge-only border
  style docTab = { size({ h: full, max-w: 48 }).pad(3).border({ edge: r, color: border-subtle, width: 1 }).border({ edge: t, color: transparent, width: 2 }).bg(surface-1).fg(muted).cursor(pointer), when: { hover: bg(hover-bg).fg(canvas), current: bg(surface-0).fg(canvas).border({ edge: t, color: primary }) } }
  style tabLabel = { size(grow).type(body-sm) }
  style dirtyDot = { fg(warning).type(body-sm) }
  style tabClose = { size(control-sm) }
  // no preset: edge-only border
  style tabsTrail = { pad(2).size(max).border({ edge: l, color: border-subtle, width: 1 }) }
  // no preset: size lock
  style routeLabel = { size({ max-w: 40 }).fg(muted).type(body-sm) }
  // no preset: edge-only border
  style errorBand = { pad(3).bg(surface-1).border({ edge: b, color: border, width: 1 }) }
  style errorText = { fg(destructive).type(body-sm) }
  style emptyHost = { size(grow).overflow(hidden).bg(surface-0) }
  style emptyFrame = { size(max).pad(4).border(subtle).radius(md) }
  style sourceHost = { size(grow).overflow(hidden).bg(surface-0) }
  style emptyInset = { size(max).pad(3).margin(3).border(subtle).radius(md) }
  style fillCol = { size(grow).overflow(hidden) }
  style editorFill = { size(grow).overflow(hidden) }
</scope>

<controller>
import type { UiEvent } from '../compositions/ui-event'
import type { Action } from '@ship/sail-runtime'
import { createEditSurface } from '../compositions/edit-surface/edit-surface-vm'
export default controller(() => {
  const vm = createEditSurface()
  const captureEditor: Action<void> = (node) => {
    vm.editorEl.set(asSailHost(node))
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
    onTabClick: (e: UiEvent) => vm.onTabClick(e),
    onTabMiddle: (e: UiEvent) => vm.onTabMiddle(e),
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
            <icon name="file-code" size="sm"></icon>
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
                size="iconSm"
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
