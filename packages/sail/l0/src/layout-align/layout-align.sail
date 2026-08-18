<!--
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
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style rail = { size({ h: 16 }), bg(muted) }
  style a = { size({ w: 8, h: 4 }), bg(primary) }
  style b = { size({ w: 8, h: 8 }), bg(accent) }
  style c = { size({ w: 8, h: 4 }), bg(muted) }
  style stretchA = { size({ w: 8 }), bg(primary) }
  style stretchB = { size({ w: 8 }), bg(accent) }
  style stretchC = { size({ w: 8 }), bg(muted) }
  style baseA = { size({ w: 8 }), pad(2), bg(primary) }
  style baseB = { size({ w: 8 }), pad({ t: 5, x: 2, b: 1 }), bg(accent) }
  style baseC = { size({ w: 8 }), pad(1), bg(muted) }
</scope>
<column gap="6">
  <column gap="2">
    <p style(look)>three bars on a 16-step rail, align=end, 8px between</p>
    <p style(spell)>&lt;row align="end" gap="2"&gt;</p>
    <row align="end" gap="2" style(rail)>
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </row>
  </column>
  <column gap="2">
    <p style(look)>align=start</p>
    <p style(spell)>&lt;row align="start" gap="2"&gt;</p>
    <row align="start" gap="2" style(rail)>
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </row>
  </column>
  <column gap="2">
    <p style(look)>align=center</p>
    <p style(spell)>&lt;row align="center" gap="2"&gt;</p>
    <row align="center" gap="2" style(rail)>
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </row>
  </column>
  <column gap="2">
    <p style(look)>align=stretch</p>
    <p style(spell)>&lt;row align="stretch" gap="2"&gt;</p>
    <row align="stretch" gap="2" style(rail)>
      <column style(stretchA)></column>
      <column style(stretchB)></column>
      <column style(stretchC)></column>
    </row>
  </column>
  <column gap="2">
    <p style(look)>align=baseline</p>
    <p style(spell)>&lt;row align="baseline" gap="2"&gt;</p>
    <row align="baseline" gap="2" style(rail)>
      <column style(baseA)>Ay</column>
      <column style(baseB)>Hg</column>
      <column style(baseC)>x</column>
    </row>
  </column>
</column>
