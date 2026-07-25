<!--
  <tab-list> — strip chrome. Sticky + variant Style. Orientation structural stamp from use:tabs.
  Paint: strip/rail from component.tab-list tokens
         (cssNames tab-list, tab-list-border, tab-list-rail).
-->

<scope>
  props: {
    sticky = false
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    variant: 'line' | 'pill' | 'enclosed' = 'line'
  }

  const base = style.borderBWidth(1).border(tab-list-border).bg(tab-list)
  const line = style.borderBWidth(1).border(tab-list-border)
  const pill = style.pad(1).borderWidth(0).radius(md).bg(tab-list-rail).borderBWidth(0)
  const enclosed = style.borderBWidth(1).border(tab-list-border)
</scope>

<row align="stretch" gap="1" role="tablist" sx={[
  base,
  match(variant, {
    line: line,
    pill: pill,
    enclosed: enclosed,
  }),
  compound({ sticky: true }, style.position(sticky).top(0).z(sticky)),
  compound({ orientation: vertical }, style.borderBWidth(0).borderEWidth(1).border(tab-list-border)),
  compound({ orientation: vertical, variant: pill }, style.borderEWidth(0)),
]}>
  <slot></slot>
</row>
