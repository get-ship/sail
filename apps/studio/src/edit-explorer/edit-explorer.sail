<!--
  <edit-explorer> — sail-tree file nav + element outline.
  Panel grammar: header · toolbar · tree · outline. Ban-clean.
-->
<controller>
import { createEditExplorer } from './edit-explorer-vm'
export default controller(() => {
  const vm = createEditExplorer()
  return {
    filterText: vm.filterText,
    hasFiles: vm.hasFiles,
    onFilterInput: (e: Event) => vm.onFilterInput(e),
    onTreeSelect: (e: Event) => vm.onTreeSelect(e),
  }
})
</controller>

<column
  data-edit-explorer
  shrink="0"
  gap="0"
  sx={style.w(56).minH(0).h(full).borderRWidth(1).borderR(border).bg(surface-1).overflow(hidden)}
>
  <row align="center" shrink="0" sx={style.h(7).padX(3).borderBWidth(1).borderB(border-subtle)}>
    <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
      Explorer
    </p>
  </row>
  <column shrink="0" gap="1" sx={style.padX(2).padY(1).borderBWidth(1).borderB(border-subtle)}>
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
    <p sx={style.fontSize(xs).fg(muted-foreground)}>Open a file to edit and preview.</p>
  </column>
  <column
    data-file-tree
    grow
    gap="0"
    sx={style.minH(0).basis(0).padX(1).padY(1).overflow(hidden)}
  >
    <if {!hasFiles()}>
      <empty-state variant="placeholder" size="sm" w="full" data-files-empty sx={style.borderWidth(1).border(border-subtle).radius(md).pad(3)}>
        <strong slot="title">No files</strong>
        Open a project to list Sail sources.
      </empty-state>
    </if>
    <if {hasFiles()}>
      <sail-tree
        data-file-tree-host
        label="Project files"
        selectable="single"
        size="sm"
        show-guides
        on:sail-select={onTreeSelect}
        sx={style.w(full).h(full).minH(0)}
      ></sail-tree>
    </if>
  </column>

  <edit-element-tree grow sx={style.minH(0).borderTWidth(1).borderT(border)} />
</column>
