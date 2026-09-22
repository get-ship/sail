<scope>
  style box = { size({ w: bind(boxW), h: 24 }), bg(primary), radius(sm) }
</scope>
<controller>
  export default controller(() => {
    const boxW = signal(240)
    mount(() => {
      const id = setInterval(() => {
        boxW.set(boxW() === 240 ? 480 : 240)
      }, 800)
      return () => clearInterval(id)
    })
    return { boxW }
  })
</controller>
<column style(box)></column>
