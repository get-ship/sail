<!--
  <edit-explorer> — project file tree only (VS Code explorer).
  Full height sail-tree; filter in toolbar. No Routes/Components tabs.
  Engine: tree={tree} identity bind (plain field — prop: prefix is refused).
  Ban-clean.
-->
<controller>
import { createEditExplorer } from './edit-explorer-vm'
export default controller(() => {
  const vm = createEditExplorer()
  return {
    tree: vm.tree,
    filterText: vm.filterText,
    hasTree: vm.hasTree,
    onFilterInput: (e: Event) => vm.onFilterInput(e),
    onTreeSelect: (e: Event) => vm.onTreeSelect(e),
  }
})
</controller>

<studio-sidebar data-edit-explorer title="Explorer">
  <column
    slot="toolbar"
    shrink="0"
    gap="2"
    sx={style.pad(2).borderBWidth(1).borderB(border-subtle)}
  >
    <input
      type="search"
      data-edit-filter
      placeholder="Filter files"
      value={filterText()}
      on:input={onFilterInput}
      aria-label="Filter files"
      size="sm"
      sx={style.w(full)}
    />
  </column>

  <column
    data-file-tree
    grow
    gap="0"
    sx={style.minH(0).basis(0).h(full).padX(1).padY(1).overflow(hidden)}
  >
    <if {!hasTree()}>
      <empty-state
        variant="placeholder"
        size="sm"
        w="full"
        data-files-empty
        sx={style.borderWidth(1).border(border-subtle).radius(md).pad(3)}
      >
        <strong slot="title">No files</strong>
        Open a project to list sources.
      </empty-state>
    </if>
    <if {hasTree()}>
      <sail-tree
        data-file-tree-host
        tree={tree}
        label="Project files"
        selectable="single"
        size="sm"
        show-guides
        on:sail-select={onTreeSelect}
        sx={style.w(full).h(full).minH(0)}
      ></sail-tree>
    </if>
  </column>
</studio-sidebar>
