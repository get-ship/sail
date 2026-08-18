<!--
  Look: a painted box whose width ticks 32px → 64px (free px, not space steps).
        Height stays space-8 (32px). No author var(). No .style.setProperty.
  Spelling: size w bind boxW · h 8 · controller signal boxW is 32 | 64 px
  Owner: language (bind is the geometry channel)
  Default: hug
  Trap: none this slice — bind already parses; the cell is the channel proof.
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style box = { size({ w: bind(boxW), h: 8 }), bg(primary), radius(sm) }
  style padBox = { pad({ x: bind(boxPad) }), size({ w: 16, h: 8 }), bg(accent), radius(sm) }
  style inner = { size({ w: 4, h: 4 }), bg(muted) }
</scope>
<controller>
  export default controller(() => {
    const boxW = signal(32)
    const boxPad = signal(8)
    mount(() => {
      const id = setInterval(() => {
        boxW.set(boxW() === 32 ? 64 : 32)
        boxPad.set(boxPad() === 8 ? 20 : 8)
      }, 800)
      return () => clearInterval(id)
    })
    return { boxW, boxPad }
  })
</controller>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>width ticks 32px → 64px from the boxW signal. h is space-8 = 32px</p>
    <p slot="spell" style(spell)>size w: bind boxW (32|64 px) · h: 8</p>
    <column style(box)></column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>pad x ticks 8px → 20px from the boxPad signal</p>
    <p slot="spell" style(spell)>pad x: bind boxPad (8|20 px)</p>
    <column style(padBox)>
      <column style(inner)></column>
    </column>
  </l0-case>
</column>
