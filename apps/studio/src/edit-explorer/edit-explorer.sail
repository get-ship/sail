<!--
  <edit-explorer> — project file tree.
  studio-sidebar · input · empty-state · sail-tree (createTable identity).
-->
<controller>
import { createEditExplorer } from './edit-explorer-vm'
export default controller(() => {
  const c = createEditExplorer()
  return {
    tree: c.tree,
    filterText: c.filterText,
    hasTree: c.hasTree,
    dirtyCountLabel: c.dirtyCountLabel,
    fileCountLabel: c.fileCountLabel,
    onFilterInput: (e: Event) => c.onFilterInput(e),
    onTreeSelect: (e: Event) => c.onTreeSelect(e),
  }
})
</controller>

<studio-sidebar edit-explorer title="Explorer">
  <column
    slot="toolbar"
    shrink="0"
    gap="2"
    sx={style.pad(2).borderBWidth(1).borderB(border-subtle).bg(surface-1)}
  >
    <input
      type="search"
      edit-filter
      placeholder="Filter files…"
      value={filterText()}
      on:input={onFilterInput}
      aria-label="Filter files"
      size="sm"
      sx={style.w(full)}
    />
    <row align="center" justify="between" nowrap gap="2">
      <p file-count sx={style.fontSize(xs).fg(muted-foreground).fontFamily(mono)}>
        {fileCountLabel()}
      </p>
      <if {dirtyCountLabel()}>
        <badge dirty-count tone="warning" size="sm" variant="soft" pill>
          {dirtyCountLabel()}
        </badge>
      </if>
    </row>
  </column>

  <column
    file-tree
    grow
    gap="0"
    sx={style.minH(0).basis(0).h(full).padX(1).padY(1).overflow(hidden).bg(surface-0)}
  >
    <if {!hasTree()}>
      <empty-state
        variant="placeholder"
        size="sm"
        w="full"
        files-empty
        sx={style.borderWidth(1).border(border-subtle).radius(md).pad(3)}
      >
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
        on:sail-select={onTreeSelect}
        sx={style.w(full).h(full).minH(0)}
      ></sail-tree>
    </if>
  </column>
</studio-sidebar>
