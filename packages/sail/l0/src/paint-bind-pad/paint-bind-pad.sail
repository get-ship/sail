<scope>
  style padBox = { pad({ x: bind(boxPad) }), size({ w: 100%, h: 24 }), bg(accent), radius(sm) }
  style inner = { size({ w: 8, h: 8 }), bg(muted) }
</scope>
<controller>
  export default controller(() => {
    const boxPad = signal(16)
    mount(() => {
      const id = setInterval(() => {
        boxPad.set(boxPad() === 16 ? 48 : 16)
      }, 800)
      return () => clearInterval(id)
    })
    return { boxPad }
  })
</controller>
<column style(padBox)>
  <column style(inner)></column>
</column>
