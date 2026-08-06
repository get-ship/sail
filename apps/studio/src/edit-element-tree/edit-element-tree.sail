<!--
  <edit-element-tree> — Components tab: sail-tree of the active doc.
  Labels short; L1/L2/L3 in node-trailing. Select → Source + span scroll.
  Engine: tree={tree} identity bind.
-->
<controller>
import { createEditElementTree } from './edit-element-tree-vm'
export default controller(() => {
  const vm = createEditElementTree()
  return {
    tree: vm.tree,
    hasElements: vm.hasElements,
    activePathLabel: vm.activePathLabel,
    onTreeSelect: (e: Event) => vm.onTreeSelect(e),
  }
})
</controller>

<column
  element-tree
  components-tree
  gap="0"
  grow

>
  style({ box: minH(0).basis(0).minW(0).h(full).overflow(hidden) })

  <row
    align="center"
    shrink="0"
    gap="2"

  >
    style({ box: h(7).padX(3), paint: borderBWidth(1).borderB(border-subtle) })

    <p>
      style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide) })

      Components
    </p>
    <p
      element-file
      grow
      min-w="0"

    >
      style({ paint: fg(muted-foreground), text: fontSize(xs).truncate().fontFamily(mono) })

      {activePathLabel()}
    </p>
  </row>
  <column grow gap="0">
    style({ box: minH(0).basis(0).padX(1).padY(1).overflow(hidden) })

    <if {!hasElements()}>
      <empty-state
        variant="placeholder"
        size="sm"
        w="full"
        elements-empty

      >
        style({ box: pad(3), paint: borderWidth(1).border(border-subtle).radius(md) })

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
        on:sail-select={onTreeSelect}

      >
        style({ box: w(full).h(full).minH(0) })
</sail-tree>
    </if>
  </column>
</column>
