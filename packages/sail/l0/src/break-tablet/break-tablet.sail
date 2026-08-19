<!--
  Theme: mobile 390 only. tablet arm compiles away.
  Sibling breakpoints.json is the 1-bp table.
-->
<scope>
  style box = { size({ w: 16, h: 16 }), bg(primary), when: { tablet: bg(accent) } }
</scope>
<column style(box)></column>
