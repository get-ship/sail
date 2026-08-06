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
    checkRows: vm.checkRows,
    hasChecks: vm.hasChecks,
    checksLabel: vm.checksLabel,
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

<row
  surface="edit"
  edit-surface
  gap="0"
  grow
  nowrap
  align="stretch"

>
  style.minH(0).basis(0).h(full).minW(0).overflow(hidden)

  <edit-explorer shrink="0">
    style.minH(0).h(full).overflow(hidden)
  </edit-explorer>

  <column
    grow
    min-w="0"
    gap="0"

  >
    style.minH(0).h(full).basis(0).minW(0).overflow(hidden)

    <!-- Doc tabs strip -->
    <row
      studio-tabs
      edit-tabs
      gap="0"
      shrink="0"
      align="stretch"
      nowrap

    >
      style.h(8).borderBWidth(1).borderB(border).bg(surface-1).minW(0).overflow(hidden)

      <row
        edit-tab-list
        label="Open documents"
        grow
        min-w="0"
        gap="0"
        align="stretch"
        nowrap
        on:click={onTabClick}
        on:mousedown={onTabMiddle}

      >
        style.minW(0).h(full).basis(0).overflowX(auto).overflowY(hidden)

        <for {docTabs()} as="tab" key={tab.key}>
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
            shrink="0"

          >
            style.h(full).padX(3).maxW(48).borderRWidth(1).borderR(border-subtle).borderTWidth(2).borderT(transparent).bg(surface-1).fg(muted-foreground).fontSize(sm).cursor(pointer).when({ hover: style.bg(hover-bg).fg(foreground) }).when({ current: style.bg(surface-0).fg(foreground).borderT(primary) })

            <icon name="file-code-2" size="sm"></icon>
            <p grow min-w="0">
              style.truncate()
{tab.label}</p>
            <if {tab.dirtyDot}>
              <p dirty-dot label="Unsaved">
                style.fontSize(xs).fg(warning)

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
                style.w(5).h(5)

                ×
              </button>
            </if>
          </row>
        </for>
      </row>
      <row
        gap="2"
        align="center"
        shrink="0"

      >
        style.padX(2).h(full).borderLWidth(1).borderL(border-subtle)

        <p>
          style.fontSize(xs).fg(muted-foreground).maxW(40).truncate()

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
        style.padX(3).padY(1).bg(surface-1).borderBWidth(1).borderB(border)

        <p>
          style.fg(destructive).fontSize(xs)
{editError()}</p>
      </row>
    </if>

    <if {!hasOpenDoc()}>
      <column
        grow
        align="center"
        justify="center"

      >
        style.minH(0).h(full).overflow(hidden).bg(surface-0)

        <empty-state
          variant="placeholder"
          size="sm"
          w="full"
          edit-empty

        >
          style.borderWidth(1).border(border-subtle).radius(md).pad(4)

          <strong slot="title">Open a file</strong>
          Pick a file from the Explorer.
        </empty-state>
      </column>
    </if>

    <if {hasOpenDoc()}>
      <column
        source-host
        cm-mode="sail"
        grow
        gap="0"

      >
        style.minH(0).basis(0).h(full).overflow(hidden).bg(surface-0)

        <!-- Status lives on tab (dirty) + chrome status bar — no SOURCE toolbar row. -->
        <if {isSourceLoading()}>
          <empty-state
            source-loading
            variant="placeholder"
            size="sm"
            w="full"

          >
            style.borderWidth(1).border(border-subtle).radius(md).pad(3).m(3)

            <strong slot="title">Loading…</strong>
            {sourceStatus()}
          </empty-state>
        </if>

        <if {isSourceEmpty()}>
          <empty-state
            source-empty
            variant="placeholder"
            size="sm"
            w="full"

          >
            style.borderWidth(1).border(border-subtle).radius(md).pad(3).m(3)

            <strong slot="title">No source loaded</strong>
            Open a file from the Explorer. Non-Sail text needs a readable ops path.
          </empty-state>
        </if>

        <if {isSourceReady()}>
          <column
            source-ready
            grow
            gap="0"

          >
            style.minH(0).basis(0).h(full).overflow(hidden)

            <if {hasChecks()}>
              <column
                source-gutter
                gap="0"
                shrink="0"

              >
                style.padX(3).padY(1).fontSize(xs).fontFamily(mono).bg(surface-2).maxH(20).minH(0).overflow(auto)

                <for {checkRows()} as="ck" key={ck.key}>
                  <p
                    check-line={ck.line}
                    severity={ck.severity}

                  >
                    style.fg(destructive).h(5)

                    L{ck.line} · {ck.severity}: {ck.message}
                  </p>
                </for>
              </column>
            </if>
            <column grow gap="0">
              style.minH(0).basis(0).h(full).overflow(hidden)

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
                style.w(full).h(full).minH(0)
</sail-code-editor>
            </column>
          </column>
        </if>
      </column>
    </if>
  </column>
</row>
