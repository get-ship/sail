<!--
  Look: the inspector line quotes .s-bg-surface and .s-pad-x4-y16
        and those names are on the specimen classList.
  Spelling: bg(surface) · pad({ x: 4, y: 16 })
  Owner: language (BUCKETS §2.6 class names)
  Default: no class
  Trap: none this slice — the live class list is the proof.
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style specimen = { size({ w: 16, h: 8 }), pad({ x: 4, y: 16 }), bg(surface), fg(muted), type(mono), radius(sm) }
</scope>
<controller>
  export default controller(() => {
    const live = signal('')
    const hit = signal('')
    mount((el) => {
      const spec = el.querySelector('[data-inspect-specimen]')
      const tick = (): void => {
        if (!spec) return
        const classes = Array.from(spec.classList).filter((c) => c.startsWith('s-'))
        live.set(classes.join(' '))
        const hasBg = classes.includes('s-bg-surface')
        const hasPad = classes.includes('s-pad-x4-y16')
        hit.set(hasBg && hasPad ? 'classList matches quoted names' : 'classList misses quoted names')
      }
      queueMicrotask(tick)
      return () => {
        live.set('')
        hit.set('')
      }
    })
    return { live, hit }
  })
</controller>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>class names read as vocabulary — .s-bg-surface · .s-pad-x4-y16</p>
    <p slot="spell" style(spell)>bg(surface) · pad x:4 y:16</p>
    <p slot="note" style(spell)>{hit()} · live {live()}</p>
    <column data-inspect-specimen style(specimen)>surface</column>
  </l0-case>
</column>
