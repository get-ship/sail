<!--
  Registered CE is src/layout-margin/.
  Look: two painted boxes; the second sits 4 steps below the first
  Spelling: style box = { margin({ t: 4 }), bg(primary) }
  Owner: theme scale + free edges
  Default: no margin
  Trap: fixtures/margin-off-scale.bad.sail — Style error: 3.5 is not a space step: did you mean 3 | 4 — or is this an exception: raw(3.5)?
        Quoted below. Not compiled on this page (CI: pages compile;
        trap fixtures assert the error).

        <column style({ margin({ t: 3.5 }) })>x</column>
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style first = { size({ w: 16, h: 8 }), bg(accent) }
  style box = { margin({ t: 4 }), size({ w: 16, h: 8 }), bg(primary) }
  style bar = { size({ w: 8, h: 8 }), bg(accent) }
  style m1 = { margin(1), size({ w: 8, h: 8 }), bg(primary) }
  style m2 = { margin(2), size({ w: 8, h: 8 }), bg(primary) }
  style m2_5 = { margin(2_5), size({ w: 8, h: 8 }), bg(primary) }
  style m3 = { margin(3), size({ w: 8, h: 8 }), bg(primary) }
  style m4 = { margin(4), size({ w: 8, h: 8 }), bg(primary) }
  style m5 = { margin(5), size({ w: 8, h: 8 }), bg(primary) }
  style m6 = { margin(6), size({ w: 8, h: 8 }), bg(primary) }
  style m7 = { margin(7), size({ w: 8, h: 8 }), bg(primary) }
  style m8 = { margin(8), size({ w: 8, h: 8 }), bg(primary) }
  style edgeX = { margin({ x: 4 }), size({ w: 8, h: 8 }), bg(primary) }
  style edgeY = { margin({ y: 4 }), size({ w: 8, h: 8 }), bg(primary) }
  style edgeT = { margin({ t: 4 }), size({ w: 8, h: 8 }), bg(primary) }
  style edgeR = { margin({ r: 4 }), size({ w: 8, h: 8 }), bg(primary) }
  style edgeB = { margin({ b: 4 }), size({ w: 8, h: 8 }), bg(primary) }
  style edgeL = { margin({ l: 4 }), size({ w: 8, h: 8 }), bg(primary) }
</scope>
<column gap="6">
  <column gap="2">
    <p style(look)>two 16×8 bars; second is 4 steps (16px) below the first</p>
    <p style(spell)>margin t:4 · size w:16 h:8</p>
    <column>
      <column style(first)></column>
      <column style(box)></column>
    </column>
  </column>
  <column gap="2">
    <p style(look)>step list — margin(N) after a face</p>
    <p style(spell)>margin(1) … margin(8) · margin(2_5)</p>
    <row>
      <column style(bar)></column>
      <column style(m1)></column>
      <column style(bar)></column>
      <column style(m2)></column>
      <column style(bar)></column>
      <column style(m2_5)></column>
      <column style(bar)></column>
      <column style(m3)></column>
      <column style(bar)></column>
      <column style(m4)></column>
      <column style(bar)></column>
      <column style(m5)></column>
      <column style(bar)></column>
      <column style(m6)></column>
      <column style(bar)></column>
      <column style(m7)></column>
      <column style(bar)></column>
      <column style(m8)></column>
    </row>
  </column>
  <column gap="2">
    <p style(look)>edges — x y t r b l at step 4</p>
    <p style(spell)>margin x / y / t / r / b / l at step 4</p>
    <row>
      <column style(bar)></column>
      <column style(edgeX)></column>
      <column style(bar)></column>
      <column style(edgeY)></column>
      <column style(bar)></column>
      <column style(edgeT)></column>
      <column style(bar)></column>
      <column style(edgeR)></column>
      <column style(bar)></column>
      <column style(edgeB)></column>
      <column style(bar)></column>
      <column style(edgeL)></column>
    </row>
  </column>
</column>
