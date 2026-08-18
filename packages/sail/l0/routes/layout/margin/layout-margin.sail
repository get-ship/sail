<!--
  Registered CE is src/layout-margin/.
  Look: two painted boxes; the second sits 4 steps below the first
  Spelling: style box = { margin({ t: 4 }), bg(primary) }
  Owner: theme scale + free edges
  Default: no margin
  Trap: fixtures/margin-off-scale.bad.sail — Style error: 3.5 is not a space step: did you mean 3 | 4 — or is this an exception: raw(3.5)?
        Quoted below. Not compiled on this page (CI: pages compile;
        trap fixtures assert the error).

        <column style({ margin({ t: 3.5 }) })>x</column>
-->
<scope>
  style first = { size({ w: 4, h: 4 }), bg(accent) }
  style box = { margin({ t: 4 }), size({ w: 4, h: 4 }), bg(primary) }
</scope>
<column>
  <column style(first)></column>
  <column style(box)></column>
</column>
