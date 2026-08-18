<!--
  Registered CE is src/layout-gap/.
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
<column gap="6">
  <column gap="2">
    <p style(look)>three 4-step boxes (16px), 8px between, none stretch</p>
    <p style(spell)>&lt;row gap="2"&gt; size w:4 h:4 · 8px is space-2</p>
    <row gap="2">
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </row>
  </column>
  <column gap="2">
    <p style(look)>gap=0 — three faces, no space</p>
    <p style(spell)>&lt;row gap="0"&gt;</p>
    <row gap="0">
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </row>
  </column>
  <column gap="2">
    <p style(look)>gap=1 — 4px</p>
    <p style(spell)>&lt;row gap="1"&gt;</p>
    <row gap="1">
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </row>
  </column>
  <column gap="2">
    <p style(look)>gap=2_5 — 10px</p>
    <p style(spell)>&lt;row gap="2_5"&gt;</p>
    <row gap="2_5">
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </row>
  </column>
  <column gap="2">
    <p style(look)>gap=3 — 12px</p>
    <p style(spell)>&lt;row gap="3"&gt;</p>
    <row gap="3">
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </row>
  </column>
  <column gap="2">
    <p style(look)>gap=4 — 16px</p>
    <p style(spell)>&lt;row gap="4"&gt;</p>
    <row gap="4">
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </row>
  </column>
  <column gap="2">
    <p style(look)>gap=5 — 20px</p>
    <p style(spell)>&lt;row gap="5"&gt;</p>
    <row gap="5">
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </row>
  </column>
  <column gap="2">
    <p style(look)>gap=6 — 24px</p>
    <p style(spell)>&lt;row gap="6"&gt;</p>
    <row gap="6">
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </row>
  </column>
  <column gap="2">
    <p style(look)>gap=7 — 28px</p>
    <p style(spell)>&lt;row gap="7"&gt;</p>
    <row gap="7">
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </row>
  </column>
  <column gap="2">
    <p style(look)>gap=8 — 32px</p>
    <p style(spell)>&lt;row gap="8"&gt;</p>
    <row gap="8">
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </row>
  </column>
</column>
