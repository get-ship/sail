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

  // Host text rides foreground.
  style base = {box: size({ w: full }), paint: ink({ color: foreground }), text: type({ family: body, size: sm, leading: normal, align: start })}
</scope>

<table style(base)>
  <slot></slot>
</table>
