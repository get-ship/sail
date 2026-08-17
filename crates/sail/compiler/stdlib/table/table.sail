<!--
  <table> — structural data table. Style product only.
  Variants dense / striped / hoverable / bordered / sticky-header / searchable
  are host props. Cell rest paint is family parts (th.sail / td.sail / caption.sail).
  Host compounds stay on platform/table.css — parts cannot see host attrs,
  and thead/tbody/tr stay HTML.

  searchable: a search field in the caption; use:tableQuery filters rows.
  Sortable columns are <th sortable> — the same action cycles aria-sort.
  The host IS the native table (an inner <table> would recurse the L1 tag).
-->

<scope>
  props: {
    dense = false
    striped = false
    hoverable = false
    bordered = false
    'sticky-header' = false
    searchable = false
  }

  style host = {group().size({ w: 100%, min-w: 0 }).overflow({ x: auto, y: visible }).fg(foreground).bg(background).border({ color: border, width: 1 }).radius(md).type({ family: body, size: sm, leading: normal, align: start })}
  style toolbar = {display(none).size({ w: 100%, min-w: 0 }).pad({ b: 2 }).type({ align: start }), when: {group-searchable: display(block)}}
  style search = {size({ w: 100%, min-w: 0 })}
</scope>

<table use:tableQuery style(host)>
  <caption data-table-search-wrap style(toolbar)>
    <input type="search" size="sm" data-table-search placeholder="Search" label="Search table" style(search)></input>
  </caption>
  <slot></slot>
</table>
