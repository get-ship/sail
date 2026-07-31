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

  const base = style.w(full).fontFamily(body).fontSize(sm).leading(normal).fg(foreground).textAlign(start)
</scope>

<table sx={base}>
  <slot></slot>
</table>
