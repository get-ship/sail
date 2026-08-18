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
  style col = { size(measure), bg(muted) }
</scope>
<column style(col)>measure</column>
