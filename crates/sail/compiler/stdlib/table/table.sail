<!--
  <table> — structural data table. Style product only.
  Variants dense / striped / hoverable / bordered / sticky-header are host props.
  Cell paint (th/td/caption/sticky) via descendant map (platform/table.css).
  Style cannot target thead — sticky-header compounds live on that map.
-->

<scope>
  props: {
    dense = false
    striped = false
    hoverable = false
    bordered = false
    'sticky-header' = false
  }

  // Host text rides foreground.
  style base = {size({ w: full }).fg(foreground).type({ family: body, size: sm, leading: normal, align: start })}
</scope>

<table style(base)>
  <slot></slot>
</table>
