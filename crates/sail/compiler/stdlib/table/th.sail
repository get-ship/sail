<!--
  <th> — table header cell. Style product only.
  numeric is a cell attr (end + tabular). Host dense/bordered/sticky stay
  on platform/table.css — a part cannot see the table host attrs.
-->

<scope>
  props: {
    numeric = false
  }

  style base = {pad({ y: 2_5, x: 3 }).fg(muted-foreground).border({ color: border, width: 1, edge: b }).type({ weight: semibold, wrap: nowrap, align: start })}
  style nums = {type({ align: end, nums: tabular })}
</scope>

<th style(base, {match: { numeric: { true: nums } }})>
  <slot></slot>
</th>
