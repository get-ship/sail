<!--
  <table> — structural data table. Style product only.
  Variants dense / striped / hoverable / bordered / sticky-header are host props.
  Cell rest paint is family parts (th.sail / td.sail / caption.sail).
  Host compounds stay on platform/table.css — parts cannot see host attrs,
  and thead/tbody/tr stay HTML.
-->

<scope>
  props: {
    dense = false
    striped = false
    hoverable = false
    bordered = false
    'sticky-header' = false
  }

  // Host text rides foreground. Outer hairline is the rest default —
  // no-border tables vanish on several themes. `bordered` still boxes cells.
  style base = {size({ w: full }).fg(foreground).border({ color: border, width: 1 }).type({ family: body, size: sm, leading: normal, align: start })}
</scope>

<table style(base)>
  <slot></slot>
</table>
