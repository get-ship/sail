<!--
  Look: a painted box; primary by default, accent at wide
  Owner: theme breakpoints (4-bp including wide)
  Trap: fixtures/when-undeclared.bad.sail
-->
<column gap="6">
  <l0-case
    look="64px box: primary, accent at wide"
    spell="when wide · bg(accent) · default bg(primary)"
    note="theme 4-bp: mobile / tablet / desktop / wide 1600"
  >
    <break-wide></break-wide>
  </l0-case>
</column>
