<scope>
  props: {
    kind: 'w' | 'pad' = 'w'
  }
  style wide = { size({ w: bind(boxW), h: 24 }), bg(primary), radius(sm) }
  style padded = { pad({ x: bind(boxPad) }), size({ w: 100%, h: 24 }), bg(accent), radius(sm) }
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
<column style({ match: { kind: { w: wide, pad: padded } } })>
  <if {kind() === 'pad'}>
    <column style(inner)></column>
  </if>
</column>
