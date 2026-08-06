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
    onTreeSelect: (detail: { id?: string }) => vm.onTreeSelect(detail),
  }
})
</controller>

<column element-tree components-tree gap="0">
  style({  box: grow(1).minH(0).basis(0).minW(0).h(full).overflow(hidden)  })

  <row align="center" gap="2">
    style({  box: shrink(0).h(7).padX(3), paint: borderBWidth(1).borderB(border-subtle)  })

    <p>
      style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(semibold).textCase(uppercase).tracking(wide) })

      Components
    </p>
    <p element-file>
      style({ box: grow(1).minW(0), paint: fg(muted-foreground), text: fontSize(xs).truncate().fontFamily(mono) })

      {activePathLabel()}
    </p>
  </row>
  <column gap="0">
    style({  box: grow(1).minH(0).basis(0).padX(1).padY(1).overflow(hidden)  })

    <if {!hasElements()}>
      <empty-state variant="placeholder" size="sm" elements-empty>
        style({  box: w(full).pad(3), paint: borderWidth(1).border(border-subtle).radius(md)  })

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

      >
        style({ box: w(full).h(full).minH(0) })
</sail-tree>
    </if>
  </column>
</column>
