<!--
  Registered CE is src/layout-participate/.
  Look: a row; one child hugs, one grows (takes leftover)
  Spelling: <row> + size(hug) and size(grow) on children. Paint with named bg only.
  Owner: language (participation)
  Default: hug
  Trap: none this slice (size(grow) outside a host is P2)
-->
<scope>
  style hugged = { size(hug), bg(primary) }
  style leftover = { size(grow), bg(accent) }
</scope>
<row>
  <column style(hugged)>hug</column>
  <column style(leftover)>grow</column>
</row>
