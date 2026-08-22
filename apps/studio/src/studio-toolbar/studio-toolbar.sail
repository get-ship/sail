<!--
  <studio-toolbar> — filter / control strip under a page header.
  Default slot: fields, separators, ghost actions (row + gap).
-->
<scope>
  // no preset: min-w lock + edge-only border
  style toolbarRoot = { size({ h: 7, min-w: 0 }).pad(2).overflow(hidden).border(subtle-b).bg(surface) }
</scope>

<row studio-toolbar align="center" gap="2" nowrap style(toolbarRoot)>
  <slot></slot>
</row>
