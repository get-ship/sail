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
  style onlyW = { size({ w: 16, h: 8 }), bg(primary) }
  style onlyH = { size({ w: 8, h: 16 }), bg(primary) }
  style minW = { size({ min-w: 16, h: 8 }), bg(accent) }
  style minH = { size({ w: 8, min-h: 16 }), bg(accent) }
  style maxW = { size({ max-w: 16, h: 8 }), bg(muted) }
  style maxH = { size({ w: 16, max-h: 8 }), bg(muted) }
  style square = { size({ w: 16, aspect: square }), bg(primary) }
  style video = { size({ w: 16, aspect: video }), bg(accent) }
  style auto = { size({ w: 16, h: 8, aspect: auto }), bg(muted) }
</scope>
<column gap="6">
  <column gap="2">
    <p style(look)>free width 7 (28px), height 8 (32px)</p>
    <p style(spell)>size w:7 h:8 · bg(accent)</p>
    <column style(box)></column>
  </column>
  <column gap="2">
    <p style(look)>w — 16-step bar</p>
    <p style(spell)>size w:16 h:8</p>
    <column style(onlyW)></column>
  </column>
  <column gap="2">
    <p style(look)>h — 16-step tall</p>
    <p style(spell)>size w:8 h:16</p>
    <column style(onlyH)></column>
  </column>
  <column gap="2">
    <p style(look)>min-w — at least 16 steps</p>
    <p style(spell)>size min-w:16 h:8</p>
    <column style(minW)></column>
  </column>
  <column gap="2">
    <p style(look)>min-h — at least 16 steps</p>
    <p style(spell)>size w:8 min-h:16</p>
    <column style(minH)></column>
  </column>
  <column gap="2">
    <p style(look)>max-w — at most 16 steps</p>
    <p style(spell)>size max-w:16 h:8</p>
    <column style(maxW)>max-w 16</column>
  </column>
  <column gap="2">
    <p style(look)>max-h — at most 8 steps</p>
    <p style(spell)>size w:16 max-h:8</p>
    <column style(maxH)>max-h 8</column>
  </column>
  <column gap="2">
    <p style(look)>aspect square</p>
    <p style(spell)>size w:16 aspect:square</p>
    <column style(square)></column>
  </column>
  <column gap="2">
    <p style(look)>aspect video</p>
    <p style(spell)>size w:16 aspect:video</p>
    <column style(video)></column>
  </column>
  <column gap="2">
    <p style(look)>aspect auto</p>
    <p style(spell)>size w:16 h:8 aspect:auto</p>
    <column style(auto)></column>
  </column>
</column>
