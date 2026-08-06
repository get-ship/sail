<!--
  <studio-toolbar> — filter / control strip under a page header.
  Default slot: fields, separators, ghost actions (row + gap).
  Ban-clean: layout keywords + L1 only.
-->
<row
  studio-toolbar
  align="center"
  gap="2"
  shrink="0"
  nowrap
  min-w="0"

>
  style.h(7).padX(2).borderBWidth(1).borderB(border-subtle).bg(surface-1).minW(0).overflow(hidden)

  <slot></slot>
</row>
