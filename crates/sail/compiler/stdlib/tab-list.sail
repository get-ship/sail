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

  const base = style({ paint: borderBWidth(1).border(tab-list-border).bg(tab-list) })
  const line = style({ paint: borderBWidth(1).border(tab-list-border) })
  const pill = style({ box: pad(1), paint: borderWidth(0).radius(md).bg(tab-list-rail).borderBWidth(0) })
  const enclosed = style({ paint: borderBWidth(1).border(tab-list-border) })
</scope>

<row align="stretch" gap="1" style(base, { match: { variant: { line: line, pill: pill, enclosed: enclosed } }, compound: [ { sticky: true, box: position(sticky).top(0), paint: z(sticky) }, { orientation: vertical, paint: borderBWidth(0).borderEWidth(1).border(tab-list-border) }, { orientation: vertical, variant: pill, paint: borderEWidth(0) } ] })>
  <slot></slot>
</row>
