<!--
  Look: one painted box with page inset. The illegal sibling const did not unregister base.
  Spelling: style base = { pad(page) }
  Owner: theme (pad(page))
  Trap: fixtures/poison-trap.bad.sail
-->
<column gap="6">
  <l0-case
    look="base survived — pad page still paints after the illegal sibling"
    spell="base = pad page · illegal sibling size.wide did not unregister base"
    note="trap: Style wide — unknown vocabulary size.wide"
  >
    <poison-trap></poison-trap>
  </l0-case>
</column>
