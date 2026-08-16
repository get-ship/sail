<!--
  <tabs> — tab strip + panels. use:tabs owns behavior + stamps orientation
  onto <tab-list>/<tab>/<tab-panel>. Size Style match lives on <tab>.
  Horizontal = list above panels (column). Vertical = list beside panels (row).
-->
<scope>
  props: {
    active = ''
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    onTabChange: (detail: { name: string }) => void
  }

  style host = {size({ min-w: 0 })}
  style horiz = {dir(column)}
  style vert = {dir(row)}
</scope>
<column gap="3" use:tabs style(host, {match: { orientation: { horizontal: horiz, vertical: vert } }})>
  <slot></slot>
</column>
