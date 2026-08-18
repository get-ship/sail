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
  style a = { size({ h: 4 }), bg(primary) }
  style b = { size({ h: 4 }), bg(accent) }
  style c = { size({ h: 4 }), bg(muted) }
  style d = { size({ h: 4 }), bg(primary) }
</scope>
<grid cols="2" gap="2">
  <column style(a)></column>
  <column style(b)></column>
  <column style(c)></column>
  <column style(d)></column>
</grid>
