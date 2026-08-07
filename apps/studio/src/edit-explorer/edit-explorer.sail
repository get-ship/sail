<!--
  <edit-explorer> — project file tree.
  studio-sidebar · input · empty-state · sail-tree (createTable identity).
-->
<controller>
import { createEditExplorer } from '../compositions/edit-explorer/edit-explorer-vm'
export default controller(() => {
  const c = createEditExplorer()
  return {
    tree: c.tree,
    filterText: c.filterText,
    hasTree: c.hasTree,
    dirtyCountLabel: c.dirtyCountLabel,
    fileCountLabel: c.fileCountLabel,
    onFilterInput: (e: Event) => c.onFilterInput(e),
    onTreeSelect: (detail: { id?: string }) => c.onTreeSelect(detail),
  }
})
</controller>

<studio-sidebar edit-explorer title="Explorer">
  <column slot="toolbar" gap="2" style({  box: shrink(0).pad(2), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1)  })>
    <input
      type="search"
      edit-filter
      placeholder="Filter files…"
      value={filterText()}
      on:input={onFilterInput}
      label="Filter files"
      size="sm" style({ box: w(full) })>
    <row align="center" justify="between" nowrap gap="2">
      <p file-count style({ paint: fg(muted-foreground), text: fontSize(xs).fontFamily(mono) })>
        {fileCountLabel()}
      </p>
      <if {dirtyCountLabel()}>
        <badge dirty-count tone="warning" size="sm" variant="soft" pill>
          {dirtyCountLabel()}
        </badge>
      </if>
    </row>
  </column>

  <column file-tree gap="0" style({  box: grow(1).minH(0).basis(0).h(full).padX(1).padY(1).overflow(hidden), paint: bg(surface-0)  })>
    <if {!hasTree()}>
      <empty-state variant="placeholder" size="sm" files-empty style({  box: w(full).pad(3), paint: borderWidth(1).border(border-subtle).radius(md)  })>
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
        on-select={onTreeSelect} style({ box: w(full).h(full).minH(0) })></sail-tree>
    </if>
  </column>
</studio-sidebar>
