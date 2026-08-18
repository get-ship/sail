<!--
  Registered CE is src/layout-box/.
  Look: one painted box of free width 7
  Spelling: style box = { size({ w: 7 }), bg(accent) }
  Owner: free
  Default: hug content
  Trap: none this slice
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style box = { size({ w: 7, h: 8 }), bg(accent) }
</scope>
<column gap="3">
  <p style(look)>free width 7 (28px), height 8 (32px)</p>
  <p style(spell)>size w:7 h:8 · bg(accent)</p>
  <column style(box)></column>
</column>
