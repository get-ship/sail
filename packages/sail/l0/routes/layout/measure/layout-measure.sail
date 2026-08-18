<!--
  Registered CE is src/layout-measure/.
  Look: a painted reading column (theme measure width)
  Spelling: style col = { size(measure), bg(muted) }
  Owner: theme
  Default: hug content
  Trap: none as a compile error. size(measure) on a sidebar is mapping
        crime (the look is not a reading column) — forbidden on these
        pages, not a new diagnostic this slice.
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style col = { size(measure), pad(4), bg(muted) }
</scope>
<column gap="3">
  <p style(look)>a reading column — theme measure width, 4-step pad</p>
  <p style(spell)>size(measure) · pad(4) · bg(muted)</p>
  <column style(col)>measure is a reading column, not a width stamp</column>
</column>
