<!--
  <edit-element-tree> — active-doc component tree (sail-tree identity bind).
-->
<scope>
  style surfaceRoot = { box: grow(1).minH(0).basis(0).minW(0).h(full).overflow(hidden) }
  style headRow = { box: shrink(0).h(7).padX(3), paint: borderBWidth(1).borderB(border-subtle) }
  style eyebrow = { paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide) }
  style pathMeta = { box: grow(1).minW(0), paint: fg(muted-foreground), text: fontSize(xs).truncate().fontFamily(mono) }
  style treeBody = { box: grow(1).minH(0).basis(0).padX(1).padY(1).overflow(hidden) }
  style emptyFrame = { box: w(full).pad(3), paint: borderWidth(1).border(border-subtle).radius(md) }
  style treeHost = { box: w(full).h(full).minH(0) }
</scope>

<controller>
import { createEditElementTree } from '../compositions/edit-element-tree/edit-element-tree-vm'
export default controller(() => {
  const vm = createEditElementTree()
  return {
    tree: vm.tree,
    hasElements: vm.hasElements,
    activePathLabel: vm.activePathLabel,
    onTreeSelect: (detail: { id?: string }) => vm.onTreeSelect(detail),
  }
})
</controller>

<column element-tree components-tree gap="0" style(surfaceRoot)>
  <row align="center" gap="2" style(headRow)>
    <p style(eyebrow)>
      Components
    </p>
    <p element-file style(pathMeta)>
      {activePathLabel()}
    </p>
  </row>
  <column gap="0" style(treeBody)>
    <if {!hasElements()}>
      <empty-state variant="placeholder" size="sm" elements-empty style(emptyFrame)>
        <strong slot="title">No elements yet</strong>
        Open a .sail file to inspect its component tree.
      </empty-state>
    </if>
    <if {hasElements()}>
      <sail-tree
        element-tree-host
        layer-badge
        tree={tree}
        label="Component tree"
        selectable="single"
        size="sm"
        show-guides
        on-select={onTreeSelect}
        style(treeHost)
      ></sail-tree>
    </if>
  </column>
</column>
