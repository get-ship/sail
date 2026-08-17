<!--
  <edit-element-tree> — active-doc component tree (sail-tree identity bind).
-->
<scope>
  style surfaceRoot = { size(grow).overflow(hidden) }
  // no preset: size lock + edge-only border
  style headRow = { size({ h: 7 }).pad(3).border({ edge: b, color: border-subtle, width: 1 }) }
  style eyebrow = { fg(muted).type(label) }
  style pathMeta = { size(grow).fg(muted).type(mono-sm) }
  style treeBody = { size(grow).pad(1).overflow(hidden) }
  style emptyFrame = { size(max).pad(3).border(subtle).radius(md) }
  style treeHost = { size(max) }
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
