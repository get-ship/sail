<!--
  <edit-element-tree> — active-doc component tree (sail-tree identity bind).
-->
<scope>
  style surfaceRoot = { grow(1).size({h: full, min-w: 0, min-h: 0, basis: 0}).overflow({ all: hidden }) }
  style headRow = { shrink(0).size({ h: 7 }).pad({ x: 3 }).border({ edge: b, color: border-subtle, width: 1 }) }
  style eyebrow = { fg(muted-foreground).type({ size: xs, weight: semibold, tracking: wide, case: uppercase }) }
  style pathMeta = { grow(1).size({ min-w: 0 }).fg(muted-foreground).type({ family: mono, size: xs, wrap: truncate }) }
  style treeBody = { grow(1).size({min-h: 0, basis: 0}).pad({ x: 1, y: 1 }).overflow({ all: hidden }) }
  style emptyFrame = { size({ w: full }).pad(3).border({ color: border-subtle, width: 1 }).radius(md) }
  style treeHost = { size({ w: full, h: full, min-h: 0 }) }
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
