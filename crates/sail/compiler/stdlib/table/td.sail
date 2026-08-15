<!--
  <td> — table data cell. Style product only.
  numeric is a cell attr (end + tabular). Last-row hairline kill and
  host dense/bordered stay on platform/table.css — last-row is not
  :last-child on the cell, and a part cannot see the table host attrs.
-->

<scope>
  props: {
    numeric = false
  }

  style base = {pad({ y: 2_5, x: 3 }).border({ color: border-subtle, width: 1, edge: b })}
  style nums = {type({ align: end, nums: tabular })}
</scope>

<td style(base, {match: { numeric: { true: nums } }})>
  <slot></slot>
</td>
