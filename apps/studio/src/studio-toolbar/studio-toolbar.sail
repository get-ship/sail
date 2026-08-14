<!--
  <studio-toolbar> — filter / control strip under a page header.
  Default slot: fields, separators, ghost actions (row + gap).
-->
<scope>
  style toolbarRoot = { shrink(0).size({ h: 7, min-w: 0 }).pad({ x: 2 }).overflow({ all: hidden }).border({ edge: b, color: border-subtle, width: 1 }).bg(surface-1) }
</scope>

<row studio-toolbar align="center" gap="2" nowrap style(toolbarRoot)>
  <slot></slot>
</row>
