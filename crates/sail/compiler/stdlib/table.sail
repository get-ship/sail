<!--
  <table> — structural data table. Style product only.
  Variants dense / striped / hoverable / bordered are host props.
  Cell paint (th/td/caption/sticky) via descendant map.
-->

<scope>
  props: {
    dense = false
    striped = false
    hoverable = false
    bordered = false
  }

  // Host text rides component.table tokens (global fallbacks in the pack).
  const base = style({ box: w(full), paint: fg(table-foreground), text: fontFamily(body).fontSize(sm).leading(normal).textAlign(start) })
</scope>

<table>
  style(base)

  <slot></slot>
</table>
