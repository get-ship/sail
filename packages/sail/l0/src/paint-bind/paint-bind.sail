<!--
  Look: a painted box whose width ticks 240px → 480px (free px, not space steps).
        Height is space-24. No author var(). No .style.setProperty.
  Spelling: size w bind boxW · h 24 · controller signal boxW is 240 | 480 px
  Owner: language (bind is the geometry channel)
  Default: hug
  Trap: none this slice — bind already parses; the cell is the channel proof.
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style box = { size({ w: bind(boxW), h: 24 }), bg(primary), radius(sm) }
  style padBox = { pad({ x: bind(boxPad) }), size({ w: 100%, h: 24 }), bg(accent), radius(sm) }
  style inner = { size({ w: 8, h: 8 }), bg(muted) }
</scope>
<controller>
  export default controller(() => {
    const boxW = signal(240)
    const boxPad = signal(16)
    mount(() => {
      const id = setInterval(() => {
        boxW.set(boxW() === 240 ? 480 : 240)
        boxPad.set(boxPad() === 16 ? 48 : 16)
      }, 800)
      return () => clearInterval(id)
    })
    return { boxW, boxPad }
  })
</controller>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>width ticks 240px → 480px from the boxW signal. h is space-24</p>
    <p slot="spell" style(spell)>size w: bind boxW (240|480 px) · h: 24</p>
    <column style(box)></column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>pad x ticks 16px → 48px from the boxPad signal</p>
    <p slot="spell" style(spell)>pad x: bind boxPad (16|48 px)</p>
    <column style(padBox)>
      <column style(inner)></column>
    </column>
  </l0-case>
</column>
