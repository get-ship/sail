<!--
  <td> — table data cell. Style product only.
  numeric is a cell attr (end + tabular). Host dense/bordered stay
  on platform/table.css — a part cannot see the table host attrs.
-->

<scope>
  props: {
    numeric = false
  }

  style base = {pad({ y: 2_5, x: 3 }).border({ color: border, width: 1, edge: b })}
  style nums = {type({ align: end, nums: tabular })}
</scope>

<td style(base, {match: { numeric: { true: nums } }})>
  <slot></slot>
</td>
