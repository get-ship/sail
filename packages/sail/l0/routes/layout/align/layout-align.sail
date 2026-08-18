<!--
  Registered CE is src/layout-align/.
  Look: three 4-step boxes in a row, aligned to the end, none stretch
  Spelling: <row align="end" gap="2"> + three painted children
            (bg(primary) / bg(accent) / bg(muted)). 8px is space-2.
  Owner: language (align attr)
  Default: row align is center
  Trap: fixtures/align-off-enum.bad.sail — <row align="middle"> — `middle` is not valid. allowed: start, center, end, stretch, baseline
        Quoted below. Not compiled on this page.

        <row align="middle">x</row>
-->
<scope>
  style a = { size({ w: 4, h: 4 }), bg(primary) }
  style b = { size({ w: 4, h: 4 }), bg(accent) }
  style c = { size({ w: 4, h: 4 }), bg(muted) }
</scope>
<row align="end" gap="2">
  <column style(a)></column>
  <column style(b)></column>
  <column style(c)></column>
</row>
