<!--
  Look: four 4-step boxes in a 2-column grid, 8px gap
  Spelling: <grid cols="2" gap="2"> + four painted children
            (bg(primary) / bg(accent) / bg(muted) / bg(primary)). 8px is space-2.
  Owner: language (grid host)
  Default: min=md auto-fill
  Trap: fixtures/grid-cols-off-enum.bad.sail — <grid cols="99"> — `99` is not valid. allowed: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12
        Quoted below. Not compiled on this page.

        <grid cols="99">x</grid>
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style hug = { size(hug) }
  style a = { size({ w: 16, h: 8 }), bg(primary) }
  style b = { size({ w: 16, h: 8 }), bg(accent) }
  style c = { size({ w: 16, h: 8 }), bg(muted) }
  style d = { size({ w: 16, h: 8 }), bg(primary) }
</scope>
<column gap="3">
  <p style(look)>2-col hug grid, four 16×8 bars, 8px gap</p>
  <p style(spell)>&lt;grid cols="2" gap="2"&gt; size(hug)</p>
  <grid cols="2" gap="2" style(hug)>
    <column style(a)></column>
    <column style(b)></column>
    <column style(c)></column>
    <column style(d)></column>
  </grid>
</column>
