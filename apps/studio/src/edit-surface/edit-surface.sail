<!--
  <edit-surface> — VS Code baseline: file explorer + doc tabs + source editor.
  No view switcher (preview/state/layout menus removed). Live app is the host.
  Ban-clean: no div/span.
-->
<controller>
import type { Action } from '@ship/sail-runtime'
import { createEditSurface } from './edit-surface-vm'
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
    onSourceSailChange: (e: Event) => vm.onSourceSailChange(e),
    onSaveSource: () => vm.onSaveSource(),
    onTabClick: (e: Event) => vm.onTabClick(e),
    onTabMiddle: (e: Event) => vm.onTabMiddle(e),
  }
})
</controller>

<row surface="edit" edit-surface gap="0" nowrap align="stretch">
  style({  box: grow(1).minH(0).basis(0).h(full).minW(0).overflow(hidden)  })

  <edit-explorer>
    style({  box: shrink(0).minH(0).h(full).overflow(hidden)  })
  </edit-explorer>

  <column gap="0">
    style({  box: grow(1).minW(0).minH(0).h(full).basis(0).overflow(hidden)  })

    <!-- Doc tabs strip -->
    <row studio-tabs edit-tabs gap="0" align="stretch" nowrap>
      style({  box: shrink(0).h(8).minW(0).overflow(hidden), paint: borderBWidth(1).borderB(border).bg(surface-1)  })

      <row edit-tab-list label="Open documents" gap="0" align="stretch" nowrap on:click={onTabClick} on:mousedown={onTabMiddle}>
        style({  box: grow(1).minW(0).h(full).basis(0).overflowX(auto).overflowY(hidden)  })

        <for {docTabs()} as="tab" key={tab.key}>
          <row tab-id={tab.id} active={tab.active ? 'true' : 'false'} tab-select selected={tab.active ? 'true' : 'false'} current={tab.active ? 'page' : undefined} title={tab.id} gap="1" align="center" nowrap>
            style({  box: shrink(0).h(full).padX(3).maxW(48), paint: borderRWidth(1).borderR(border-subtle).borderTWidth(2).borderT(transparent).bg(surface-1).fg(muted-foreground).cursor(pointer)  }).when(hover, { paint: bg(hover-bg).fg(foreground) }).when(current, { paint: bg(surface-0).fg(foreground).borderT(primary) })
            <span>
              style({ box: display(contents), text: fontSize(sm) })
              <icon name="file-code-2" size="sm"></icon>
            <p>
              style({ box: grow(1).minW(0), text: truncate() })
{tab.label}</p>
            <if {tab.dirtyDot}>
              <p dirty-dot label="Unsaved">
                style({ paint: fg(warning), text: fontSize(xs) })

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

              >
                style({ box: w(5).h(5) })

                ×
              </button>
            </if>
            </span>
</row>
        </for>
      </row>
      <row gap="2" align="center">
        style({  box: shrink(0).padX(2).h(full), paint: borderLWidth(1).borderL(border-subtle)  })

        <p>
          style({ box: maxW(40), paint: fg(muted-foreground), text: fontSize(xs).truncate() })

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
      <row
        edit-error
        align="center"
        gap="2"

      >
        style({ box: padX(3).padY(1), paint: bg(surface-1).borderBWidth(1).borderB(border) })

        <p>
          style({ paint: fg(destructive), text: fontSize(xs) })
{editError()}</p>
      </row>
    </if>

    <if {!hasOpenDoc()}>
      <column align="center" justify="center">
        style({  box: grow(1).minH(0).h(full).overflow(hidden), paint: bg(surface-0)  })

        <empty-state variant="placeholder" size="sm" edit-empty>
          style({  box: w(full).pad(4), paint: borderWidth(1).border(border-subtle).radius(md)  })

          <strong slot="title">Open a file</strong>
          Pick a file from the Explorer.
        </empty-state>
      </column>
    </if>

    <if {hasOpenDoc()}>
      <column source-host cm-mode="sail" gap="0">
        style({  box: grow(1).minH(0).basis(0).h(full).overflow(hidden), paint: bg(surface-0)  })

        <!-- Status lives on tab (dirty) + chrome status bar — no SOURCE toolbar row. -->
        <if {isSourceLoading()}>
          <empty-state source-loading variant="placeholder" size="sm">
            style({  box: w(full).pad(3).m(3), paint: borderWidth(1).border(border-subtle).radius(md)  })

            <strong slot="title">Loading…</strong>
            {sourceStatus()}
          </empty-state>
        </if>

        <if {isSourceEmpty()}>
          <empty-state source-empty variant="placeholder" size="sm">
            style({  box: w(full).pad(3).m(3), paint: borderWidth(1).border(border-subtle).radius(md)  })

            <strong slot="title">No source loaded</strong>
            Open a file from the Explorer. Non-Sail text needs a readable ops path.
          </empty-state>
        </if>

        <if {isSourceReady()}>
          <column source-ready gap="0">
            style({  box: grow(1).minH(0).basis(0).h(full).overflow(hidden)  })

            <column gap="0">
              style({  box: grow(1).minH(0).basis(0).h(full).overflow(hidden)  })

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
                on:sail-change={onSourceSailChange}
                on:sail-save={onSaveSource}
                label="Source editor"

              >
                style({ box: w(full).h(full).minH(0) })
</sail-code-editor>
            </column>
          </column>
        </if>
      </column>
    </if>
  </column>
</row>
