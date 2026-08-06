<!--
  <studio-toolbar> — filter / control strip under a page header.
  Default slot: fields, separators, ghost actions (row + gap).
  Ban-clean: layout keywords + L1 only.
-->
<row studio-toolbar align="center" gap="2" nowrap>
  style({  box: shrink(0).minW(0).h(7).padX(2).minW(0).overflow(hidden), paint: borderBWidth(1).borderB(border-subtle).bg(surface-1)  })

  <slot></slot>
</row>
