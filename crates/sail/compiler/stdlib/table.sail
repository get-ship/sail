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

  const base = style({ box: w(full), paint: fg(foreground), text: fontFamily(body).fontSize(sm).leading(normal).textAlign(start) })
</scope>

<table>
  style(base)

  <slot></slot>
</table>
