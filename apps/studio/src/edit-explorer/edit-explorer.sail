<!--
  <edit-explorer> — project file tree (studio-sidebar + sail-tree).
-->
<scope>
  // no preset: edge-only border
  style toolbar = { size(hug).pad(2).border(subtle-b).bg(surface) }
  style filterInput = { size(max) }
  style countMono = { fg(muted).type(mono-sm) }
  // Participation form — sidebar slot parent is not a layout host.
  style treeBody = { grow(1).pad(1).overflow(hidden).bg(app) }
  style emptyFrame = { size(max).pad(3).border(subtle).radius(md) }
  style treeHost = { size(max) }
</scope>

<controller>
import type { UiEvent } from '../compositions/ui-event'
import { createEditExplorer } from '../compositions/edit-explorer/edit-explorer-vm'
export default controller(() => {
  const c = createEditExplorer()
  return {
    tree: c.tree,
    filterText: c.filterText,
    hasTree: c.hasTree,
    dirtyCountLabel: c.dirtyCountLabel,
    fileCountLabel: c.fileCountLabel,
    onFilterInput: (e: UiEvent) => c.onFilterInput(e),
    onTreeSelect: (detail: { id?: string }) => c.onTreeSelect(detail),
  }
})
</controller>

<studio-sidebar edit-explorer title="Explorer">
  <column slot="toolbar" gap="2" style(toolbar)>
    <input
      type="search"
      edit-filter
      placeholder="Filter files…"
      value={filterText()}
      on:input={onFilterInput}
      label="Filter files"
      size="sm"
      style(filterInput)
    >
    <row align="center" justify="between" nowrap gap="2">
      <p file-count style(countMono)>
        {fileCountLabel()}
      </p>
      <if {dirtyCountLabel()}>
        <badge dirty-count tone="warning" size="sm" variant="soft" pill>
          {dirtyCountLabel()}
        </badge>
      </if>
    </row>
  </column>

  <column file-tree gap="0" style(treeBody)>
    <if {!hasTree()}>
      <empty-state variant="placeholder" size="sm" files-empty style(emptyFrame)>
        <strong slot="title">No files</strong>
        Open a project to list sources.
      </empty-state>
    </if>
    <if {hasTree()}>
      <sail-tree
        file-tree-host
        tree={tree}
        label="Project files"
        selectable="single"
        size="sm"
        show-guides
        on-select={onTreeSelect}
        style(treeHost)
      ></sail-tree>
    </if>
  </column>
</studio-sidebar>
