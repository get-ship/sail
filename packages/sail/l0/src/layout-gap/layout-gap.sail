<!--
  Look: three 4-step boxes, 8px between, none stretch
  Spelling: <row gap="2"> + three painted children (bg(primary) / bg(accent) / bg(muted)).
            8px is space-2 (gap="2"). Children use size({ w: 4, h: 4 }). w: 4 is a
            space-step length, so this is the spec look. h: 4 keeps them boxes
            (not stretch, not zero-height).
  Owner: language (gap) + theme scale for the step
  Default: no gap
  Trap: none
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style a = { size({ w: 4, h: 4 }), bg(primary) }
  style b = { size({ w: 4, h: 4 }), bg(accent) }
  style c = { size({ w: 4, h: 4 }), bg(muted) }
</scope>
<column gap="3">
  <p style(look)>three 4-step boxes (16px), 8px between, none stretch</p>
  <p style(spell)>&lt;row gap="2"&gt; size w:4 h:4 · 8px is space-2</p>
  <row gap="2">
    <column style(a)></column>
    <column style(b)></column>
    <column style(c)></column>
  </row>
</column>
