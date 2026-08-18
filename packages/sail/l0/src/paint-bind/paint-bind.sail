<!--
  Look: a painted box whose width ticks between 8 and 16 steps.
        No author var(). No .style.setProperty.
  Spelling: size w bind boxW · h 8 · controller signal boxW
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
    const boxW = signal(8)
    const boxPad = signal(2)
    mount(() => {
      const id = setInterval(() => {
        boxW.set(boxW() === 8 ? 16 : 8)
        boxPad.set(boxPad() === 2 ? 5 : 2)
      }, 800)
      return () => clearInterval(id)
    })
    return { boxW, boxPad }
  })
</controller>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>width ticks 8 → 16 → 8 from the boxW signal</p>
    <p slot="spell" style(spell)>size w: bind boxW · h: 8</p>
    <column style(box)></column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>pad x ticks 2 → 5 from the boxPad signal</p>
    <p slot="spell" style(spell)>pad x: bind boxPad</p>
    <column style(padBox)>
      <column style(inner)></column>
    </column>
  </l0-case>
</column>
