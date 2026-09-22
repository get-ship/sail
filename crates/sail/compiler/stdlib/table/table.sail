<!--
  <table> — structural data table. Style product only.
  Variants dense / striped / hoverable / bordered / sticky-header
  are host props. Cell rest paint is family parts (th.sail / td.sail / caption.sail).
  Host compounds stay on platform/table.css — parts cannot see host attrs,
  and thead/tbody/tr stay HTML.

  Sortable columns are <th sortable> — the sort mark shows once the th
  carries aria-sort. Rows come through the slot; filtering belongs to the caller.
  The host IS the native table (an inner <table> would recurse the L1 tag).
-->

<scope>
  props: {
    dense = false
    striped = false
    hoverable = false
    bordered = false
    'sticky-header' = false
  }

  style host = {size({ w: 100%, min-w: 0 }).overflow({ x: auto, y: visible }).fg(foreground).bg(background).border({ color: border, width: 1 }).radius(md).type({ family: body, size: sm, leading: normal, align: start })}
</scope>

<table style(host)>
  <slot></slot>
</table>
