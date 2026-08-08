<!--
  <studio-toolbar> — filter / control strip under a page header.
  Default slot: fields, separators, ghost actions (row + gap).
-->
<scope>
  style toolbarRoot = {
    box: shrink(0).minW(0).h(7).padX(2).overflow(hidden),
    paint: borderBWidth(1).borderB(border-subtle).bg(surface-1),
  }
</scope>

<row studio-toolbar align="center" gap="2" nowrap style(toolbarRoot)>
  <slot></slot>
</row>
