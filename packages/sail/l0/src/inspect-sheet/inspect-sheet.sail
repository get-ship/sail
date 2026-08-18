<!--
  Look: two faces spend bg(surface). The sheet emits one .s-bg-surface rule.
  Spelling: two columns, style faceA / faceB — both bg(surface), pad 4 vs pad 8
  Owner: language (shared concern rules)
  Default: one rule per unique concern application
  Trap: none this slice — the sheet count is the proof.
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style faceA = { size({ w: 8, h: 8 }), pad(4), bg(surface), fg(muted), type(mono), radius(sm) }
  style faceB = { size({ w: 8, h: 8 }), pad(8), bg(surface), fg(muted), type(mono), radius(sm) }
</scope>
<controller>
  export default controller(() => {
    const count = signal('')
    mount((el) => {
      const tick = (): void => {
        const tag = el.querySelector('style')
        const css = tag?.textContent ?? ''
        let n = 0
        let from = 0
        while (from < css.length) {
          const i = css.indexOf('.s-bg-surface{', from)
          if (i < 0) break
          n += 1
          from = i + 1
        }
        count.set(`${n} .s-bg-surface rule`)
      }
      queueMicrotask(tick)
      return () => {
        count.set('')
      }
    })
    return { count }
  })
</controller>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>two faces share bg(surface) — one .s-bg-surface rule</p>
    <p slot="spell" style(spell)>bg(surface) · pad(4) vs pad(8)</p>
    <p slot="note" style(spell)>{count()}</p>
    <row gap="2">
      <column data-inspect-a style(faceA)>4</column>
      <column data-inspect-b style(faceB)>8</column>
    </row>
  </l0-case>
</column>
