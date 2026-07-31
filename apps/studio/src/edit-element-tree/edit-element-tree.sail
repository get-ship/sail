<!--
  <edit-element-tree> — hierarchy navigator: indent + tag + L1 badge.
  Keyboard: ArrowUp/Down + Enter. Click selects → source span.
  Ban-clean L1 only.
-->
<controller>
import { createEditElementTree } from './edit-element-tree-vm'
export default controller(() => {
  const vm = createEditElementTree()
  return {
    elementRows: vm.elementRows,
    hasElements: vm.hasElements,
    activePathLabel: vm.activePathLabel,
    activeDescendant: vm.activeDescendant,
    onElementClick: (e: Event) => vm.onElementClick(e),
    onTreeKeydown: (e: Event) => vm.onTreeKeydown(e),
  }
})
</controller>

<column data-element-tree gap="0" grow sx={style.minH(0).minW(0)}>
  <row align="center" shrink="0" sx={style.h(7).padX(3).borderBWidth(1).borderB(border-subtle)}>
    <p sx={style.fontSize(xs).fontWeight(semibold).fg(muted-foreground).textCase(uppercase).tracking(wide)}>
      Elements · {activePathLabel()}
    </p>
  </row>
  <scroll-area
    grow
    role="tree"
    tabindex="0"
    aria-label="Element tree"
    aria-activedescendant={activeDescendant()}
    on:click={onElementClick}
    on:keydown={onTreeKeydown}
    sx={style.minH(0)}
  >
    <column gap="0" sx={style.fontFamily(mono).fontSize(xs).padY(1)}>
      <if {!hasElements()}>
        <empty-state variant="placeholder" size="sm" w="full" sx={style.borderWidth(1).border(border-subtle).radius(md).pad(3)}>
          <strong slot="title">No elements yet</strong>
          Open a .sail file to browse its tree.
        </empty-state>
      </if>
      <for {elementRows()} as="el" key={el.key}>
        <button
          type="button"
          id={el.domId}
          role="treeitem"
          tabindex={el.tabIndex}
          data-node-id={el.id}
          data-depth={el.depth}
          data-selected={el.selected ? 'true' : 'false'}
          data-focused={el.focused ? 'true' : 'false'}
          aria-selected={el.selected ? 'true' : 'false'}
          size="sm"
          variant={el.selected || el.focused ? 'soft' : 'ghost'}
          tone={el.selected ? 'primary' : 'muted'}
          sx={style.w(full).h(6).padX(2).radius(none).when({ hover: style.bg(hover-bg) })}
        >
          {el.pad}{el.label}
          <if {el.layerBadge}>
            <p data-layer-badge sx={style.fontSize(xs).fg(muted-foreground)}>{el.layerBadge}</p>
          </if>
        </button>
      </for>
    </column>
  </scroll-area>
</column>
