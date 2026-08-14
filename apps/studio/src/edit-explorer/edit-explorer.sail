<!--
  <edit-explorer> — project file tree (studio-sidebar + sail-tree).
-->
<scope>
  style toolbar = { shrink(0).pad(2).border({ edge: b, color: border-subtle, width: 1 }).bg(surface-1) }
  style filterInput = { size({ w: full }) }
  style countMono = { fg(muted-foreground).type({ family: mono, size: xs }) }
  style treeBody = { grow(1).size({h: full, min-h: 0, basis: 0}).pad({ x: 1, y: 1 }).overflow({ all: hidden }).bg(surface-0) }
  style emptyFrame = { size({ w: full }).pad(3).border({ color: border-subtle, width: 1 }).radius(md) }
  style treeHost = { size({ w: full, h: full, min-h: 0 }) }
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
