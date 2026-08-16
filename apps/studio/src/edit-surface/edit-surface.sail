<!--
  <edit-surface> — file explorer + doc tabs + source editor. Live app is the host.
-->
<scope>
  style surfaceRoot = { grow(1).size({h: full, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }) }
  style explorerHost = { shrink(0).size({ h: full, min-h: 0 }).overflow({ all: hidden }) }
  style mainCol = { grow(1).size({h: full, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }) }
  style tabsStrip = { shrink(0).size({ h: 8, min-w: 0 }).overflow({ all: hidden }).border({ edge: b, color: border, width: 1 }).bg(surface-1) }
  style tabList = { grow(1).size({h: full, min-w: 0, basis: 0}).overflow({ x: auto, y: hidden }) }
  style docTab = { shrink(0).size({ h: full, max-w: 48 }).pad({ x: 3 }).border({ edge: r, color: border-subtle, width: 1 }).border({ edge: t, color: transparent, width: 2 }).bg(surface-1).fg(muted-foreground).cursor(pointer), when: { hover: bg(hover-bg).fg(foreground), current: bg(surface-0).fg(foreground).border({ edge: t, color: primary }) } }
  style tabLabel = { grow(1).size({ min-w: 0 }).type({ size: sm, wrap: truncate }) }
  style dirtyDot = { fg(warning).type({ size: xs }) }
  style tabClose = { shrink(0).size({ w: control-h-sm, h: control-h-sm, min-w: control-h-sm, min-h: control-h-sm }) }
  style tabsTrail = { shrink(0).pad({ x: 2 }).size({ h: full }).border({ edge: l, color: border-subtle, width: 1 }) }
  style routeLabel = { size({ max-w: 40 }).fg(muted-foreground).type({ size: xs, wrap: truncate }) }
  style errorBand = { pad({ x: 3, y: 1 }).bg(surface-1).border({ edge: b, color: border, width: 1 }) }
  style errorText = { fg(destructive).type({ size: xs }) }
  style emptyHost = { grow(1).size({ h: full, min-h: 0 }).overflow({ all: hidden }).bg(surface-0) }
  style emptyFrame = { size({ w: full }).pad(4).border({ color: border-subtle, width: 1 }).radius(md) }
  style sourceHost = { grow(1).size({h: full, min-h: 0, basis: 0}).overflow({ all: hidden }).bg(surface-0) }
  style emptyInset = { size({ w: full }).pad(3).margin(3).border({ color: border-subtle, width: 1 }).radius(md) }
  style fillCol = { grow(1).size({h: full, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }) }
  style editorFill = { grow(1).size({ w: full, h: full, min-w: 0, min-h: 0, basis: 0 }).overflow({ all: hidden }) }
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
