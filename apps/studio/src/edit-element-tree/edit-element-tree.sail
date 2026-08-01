<!--
  <edit-element-tree> — sail-tree hierarchy with L1/L2/L3 badges in labels.
  Select → shared selection + source span scroll. Ban-clean L1 + sail-tree.
-->
<controller>
import { createEditElementTree } from './edit-element-tree-vm'
export default controller(() => {
  const vm = createEditElementTree()
  return {
    hasElements: vm.hasElements,
    activePathLabel: vm.activePathLabel,
    onTreeSelect: (e: Event) => vm.onTreeSelect(e),
  }
})
</controller>

<column data-element-tree gap="0" grow sx={style.minH(0).minW(0)}>
  <row align="center" shrink="0" sx={style.h(7).padX(3).borderBWidth(1).borderB(border-subtle)}>
    <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
      Elements · {activePathLabel()}
    </p>
  </row>
  <column grow gap="0" sx={style.minH(0).padX(1).padY(1).overflow(hidden)}>
    <if {!hasElements()}>
      <empty-state variant="placeholder" size="sm" w="full" sx={style.borderWidth(1).border(border-subtle).radius(md).pad(3)}>
        <strong slot="title">No elements yet</strong>
        Open a .sail file to browse its tree.
      </empty-state>
    </if>
    <if {hasElements()}>
      <sail-tree
        data-element-tree-host
        data-layer-badge
        label="Element tree"
        selectable="single"
        size="sm"
        show-guides
        on:sail-select={onTreeSelect}
        sx={style.w(full).h(full).minH(0)}
      ></sail-tree>
    </if>
  </column>
</column>
