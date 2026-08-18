<!--
  Look: one painted box with 4-step horizontal inset (pad({ x: 4 }))
  Spelling: style box = { pad({ x: 4 }), bg(primary) }
  Owner: theme scale + free edges
  Default: no inset
  Trap: fixtures/pad-off-scale.bad.sail — Style error: 3.5 is not a space step: did you mean 3 | 4 — or is this an exception: raw(3.5)?
        Quoted below. Not compiled on this page (CI: pages compile;
        trap fixtures assert the error).

        <column style({ pad({ x: 3.5 }) })>x</column>
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style box = { pad({ x: 4 }), size({ w: 16, h: 8 }), bg(primary) }
</scope>
<column gap="3">
  <p style(look)>16-step bar, 4-step (16px) inset on x</p>
  <p style(spell)>pad x:4 · size w:16 h:8 · bg(primary)</p>
  <column style(box)>x</column>
</column>
