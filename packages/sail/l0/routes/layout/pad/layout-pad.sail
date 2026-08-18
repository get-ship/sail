<!--
  Registered CE is src/layout-pad/.
  Look: one painted box with 4-step horizontal inset (pad({ x: 4 }))
  Spelling: style box = { pad({ x: 4 }), bg(primary) }
  Owner: theme scale + free edges
  Default: no inset
  Trap: fixtures/pad-off-scale.bad.sail — Style error: 3.5 is not a space step: did you mean 3 | 4 — or is this an exception: raw(3.5)?
        Quoted below. Not compiled on this page (CI: pages compile;
        trap fixtures assert the error).

        <column style({ pad({ x: 3.5 }) })>x</column>
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style box = { pad({ x: 4 }), size({ w: 16, h: 8 }), bg(primary) }
  style pad1 = { pad(1), size({ w: 24, h: 24 }), bg(primary) }
  style pad2 = { pad(2), size({ w: 24, h: 24 }), bg(primary) }
  style pad2_5 = { pad(2_5), size({ w: 24, h: 24 }), bg(primary) }
  style pad3 = { pad(3), size({ w: 24, h: 24 }), bg(primary) }
  style pad4 = { pad(4), size({ w: 24, h: 24 }), bg(primary) }
  style pad5 = { pad(5), size({ w: 24, h: 24 }), bg(primary) }
  style pad6 = { pad(6), size({ w: 24, h: 24 }), bg(primary) }
  style pad7 = { pad(7), size({ w: 24, h: 24 }), bg(primary) }
  style pad8 = { pad(8), size({ w: 24, h: 24 }), bg(primary) }
  style edgeX = { pad({ x: 4 }), size({ w: 16, h: 8 }), bg(accent) }
  style edgeY = { pad({ y: 4 }), size({ w: 16, h: 8 }), bg(accent) }
  style edgeT = { pad({ t: 4 }), size({ w: 16, h: 8 }), bg(accent) }
  style edgeR = { pad({ r: 4 }), size({ w: 16, h: 8 }), bg(accent) }
  style edgeB = { pad({ b: 4 }), size({ w: 16, h: 8 }), bg(accent) }
  style edgeL = { pad({ l: 4 }), size({ w: 16, h: 8 }), bg(accent) }
  style inner = { size({ w: 4, h: 4 }), bg(muted) }
</scope>
<column gap="6">
  <column gap="2">
    <p style(look)>16-step bar, 4-step (16px) inset on x</p>
    <p style(spell)>pad x:4 · size w:16 h:8 · bg(primary)</p>
    <column style(box)>x</column>
  </column>
  <column gap="2">
    <p style(look)>step list — pad(N) on one face, same host</p>
    <p style(spell)>pad(1) … pad(8) · pad(2_5)</p>
    <row gap="2">
      <column style(pad1)><column style(inner)></column></column>
      <column style(pad2)><column style(inner)></column></column>
      <column style(pad2_5)><column style(inner)></column></column>
      <column style(pad3)><column style(inner)></column></column>
      <column style(pad4)><column style(inner)></column></column>
      <column style(pad5)><column style(inner)></column></column>
      <column style(pad6)><column style(inner)></column></column>
      <column style(pad7)><column style(inner)></column></column>
      <column style(pad8)><column style(inner)></column></column>
    </row>
  </column>
  <column gap="2">
    <p style(look)>edges — x y t r b l at step 4</p>
    <p style(spell)>pad x / y / t / r / b / l at step 4</p>
    <row gap="2">
      <column style(edgeX)>x</column>
      <column style(edgeY)>y</column>
      <column style(edgeT)>t</column>
      <column style(edgeR)>r</column>
      <column style(edgeB)>b</column>
      <column style(edgeL)>l</column>
    </row>
  </column>
</column>
