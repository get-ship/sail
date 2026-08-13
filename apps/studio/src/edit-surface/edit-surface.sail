<!--
  <edit-surface> — file explorer + doc tabs + source editor. Live app is the host.
-->
<scope>
  style surfaceRoot = { box: flow({ grow: 1 }).size({ h: full, min-w: 0, min-h: 0 }).basis(0).overflow({ all: hidden }) }
  style explorerHost = { box: flow({ shrink: 0 }).size({ h: full, min-h: 0 }).overflow({ all: hidden }) }
  style mainCol = { box: flow({ grow: 1 }).size({ h: full, min-w: 0, min-h: 0 }).basis(0).overflow({ all: hidden }) }
  style tabsStrip = { box: flow({ shrink: 0 }).size({ h: 8, min-w: 0 }).overflow({ all: hidden }), paint: border({ edge: b, color: border, width: 1 }).fill({ color: surface-1 }) }
  style tabList = { box: flow({ grow: 1 }).size({ h: full, min-w: 0 }).basis(0).overflow({ x: auto, y: hidden }) }
  style docTab = {
    box: flow({ shrink: 0 }).size({ h: full, max-w: 48 }).space({ pad-x: 3 }),
    paint: border({ edge: r, color: border-subtle, width: 1 }).border({ edge: t, color: transparent, width: 2 }).fill({ color: surface-1 }).ink({ color: muted-foreground }).cursor(pointer),
    when: {
      hover: { paint: fill({ color: hover-bg }).ink({ color: foreground }) },
      current: { paint: fill({ color: surface-0 }).ink({ color: foreground }).border({ edge: t, color: primary }) },
    },
  }
  style tabLabel = { box: flow({ grow: 1 }).size({ min-w: 0 }), text: type({ size: sm, wrap: truncate }) }
  style dirtyDot = { paint: ink({ color: warning }), text: type({ size: xs }) }
  style tabClose = { box: flow({ shrink: 0 }).size({ w: control-h-sm, h: control-h-sm, min-w: control-h-sm, min-h: control-h-sm }) }
  style tabsTrail = { box: flow({ shrink: 0 }).space({ pad-x: 2 }).size({ h: full }), paint: border({ edge: l, color: border-subtle, width: 1 }) }
  style routeLabel = { box: size({ max-w: 40 }), paint: ink({ color: muted-foreground }), text: type({ size: xs, wrap: truncate }) }
  style errorBand = { box: space({ pad-x: 3, pad-y: 1 }), paint: fill({ color: surface-1 }).border({ edge: b, color: border, width: 1 }) }
  style errorText = { paint: ink({ color: destructive }), text: type({ size: xs }) }
  style emptyHost = { box: flow({ grow: 1 }).size({ h: full, min-h: 0 }).overflow({ all: hidden }), paint: fill({ color: surface-0 }) }
  style emptyFrame = { box: size({ w: full }).space({ pad: 4 }), paint: border({ color: border-subtle, width: 1 }).radius(md) }
  style sourceHost = { box: flow({ grow: 1 }).size({ h: full, min-h: 0 }).basis(0).overflow({ all: hidden }), paint: fill({ color: surface-0 }) }
  style emptyInset = { box: size({ w: full }).space({ pad: 3, m: 3 }), paint: border({ color: border-subtle, width: 1 }).radius(md) }
  style fillCol = { box: flow({ grow: 1 }).size({ h: full, min-h: 0 }).basis(0).overflow({ all: hidden }) }
  style editorFill = { box: size({ w: full, h: full, min-h: 0 }) }
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
