<!--
  Registered CE is src/layout-pad/.
  Look: one painted box with 4-step horizontal inset (pad({ x: 4 }))
  Spelling: style box = { pad({ x: 4 }), bg(primary) }
  Owner: theme scale + free edges
  Default: no inset
  Trap: fixtures/pad-off-scale.bad.sail — Style error: 3.5 is not a space step: 3 | 4; 3.5 is not a space step: 3 | 4
        Quoted below. Not compiled on this page (CI: pages compile;
        trap fixtures assert the error).

        <column style({ pad({ x: 3.5 }) })>x</column>
-->
<scope>
  style box = { pad({ x: 4 }), bg(primary) }
</scope>
<column style(box)>x</column>
