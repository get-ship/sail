<!--
  <studio-toolbar> — filter / control strip under a page header.
  Default slot: fields, separators, ghost actions (row + gap).
-->
<scope>
  style toolbarRoot = {
    box: flow({ shrink: 0 }).size({ h: 7, min-w: 0 }).space({ pad-x: 2 }).overflow({ all: hidden }),
    paint: border({ edge: b, color: border-subtle, width: 1 }).fill({ color: surface-1 }),
  }
</scope>

<row studio-toolbar align="center" gap="2" nowrap style(toolbarRoot)>
  <slot></slot>
</row>
