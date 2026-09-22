<!--
  <th> — table header cell. Style product only.
  numeric is a cell attr (end + tabular). Host dense/bordered/sticky stay
  on platform/table.css — a part cannot see the table host attrs.
-->

<scope>
  props: {
    numeric = false
    sortable = false
  }

  style base = {group().pad({ y: 2_5, x: 3 }).fg(muted-foreground).bg(muted).border({ color: border, width: 1, edge: b }).type({ weight: semibold, wrap: nowrap, align: start })}
  style nums = {type({ align: end, nums: tabular })}
  style sort = {cursor(pointer).select(none)}
  style mark = {display(none).fg(muted-foreground).type({ size: xs, leading: none, weight: medium }), when: {group-sortable: display(inline)}}
</scope>

<th style(base, {match: { numeric: { true: nums }, sortable: { true: sort } }})>
  <row align="center" gap="1" nowrap>
    <slot></slot>
    <span hidden="true" data-sort-mark style(mark)>↕</span>
  </row>
</th>
