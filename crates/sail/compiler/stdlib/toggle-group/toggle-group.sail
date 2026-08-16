<!--
  <toggle-group> — segmented Style match; structural orientation stamp on cells.
  Size/variant paint is authored on each toggle.
  Vertical: <row> dir(column) + align=stretch so cells share one column width.
-->

<scope>
  props: {
    type: 'single' | 'multiple' = 'single'
    value = ''
    variant: 'outline' | 'ghost' | 'segmented' = 'outline'
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    onToggleChange: (detail: { value: string }) => void
  }

  style host = {display(inline-flex)}
  style horiz = {dir(row)}
  style vert = {dir(column)}
  style segmented = {pad(1).bg(muted).border({ color: border, width: 1 }).radius(md)}
</scope>

<row inline align="stretch" gap="0" use:toggleGroup style(host, {match: { orientation: { horizontal: horiz, vertical: vert }, variant: {outline: bg(transparent), ghost: bg(transparent), segmented: segmented}}})>
  <slot></slot>
</row>
