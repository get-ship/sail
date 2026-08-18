<scope>
  props: {}
  style band = { pad(5), bg(muted), radius(md) }
  style well = { size(grow), pad(6), bg(surface), radius(sm) }
  style nums = { size({ w: 48 }), pad({ l: 4 }) }
  style meta = { type(mono), fg(muted) }
</scope>

<controller>
  export default controller(() => {
    const boxList = signal<{ k: string; t: string }[]>([])
    let well: Element | null = null
    const tick = (): void => {
      if (!well) return
      const wr = well.getBoundingClientRect()
      const out: { k: string; t: string }[] = []
      const walk = (n: Element): void => {
        for (const c of Array.from(n.children)) {
          if (c.localName === 'p') {
            walk(c)
            continue
          }
          const r = c.getBoundingClientRect()
          const bg = getComputedStyle(c).backgroundColor
          const painted =
            bg !== 'rgba(0, 0, 0, 0)' && bg !== 'transparent' && r.width > 0 && r.height > 0
          const fillsWell = r.width >= wr.width * 0.95 && r.height >= wr.height * 0.95
          if (painted && !fillsWell) {
            out.push({
              k: String(out.length),
              t: `${c.localName} ${Math.round(r.width)}×${Math.round(r.height)} +${Math.round(r.x - wr.x)},${Math.round(r.y - wr.y)}`,
            })
          }
          walk(c)
        }
      }
      walk(well)
      boxList.set(out)
    }
    mount((el) => {
      well = el.querySelector('[data-l0-well]')
      queueMicrotask(tick)
      if (!well || typeof ResizeObserver === 'undefined') return
      const ro = new ResizeObserver(tick)
      ro.observe(well)
      return () => {
        ro.disconnect()
        well = null
      }
    })
    return { boxList }
  })
</controller>

<column gap="4" style(band)>
  <column gap="1">
    <slot name="look"></slot>
    <slot name="spell"></slot>
    <slot name="note"></slot>
  </column>
  <row gap="4" align="start">
    <column data-l0-well style(well)>
      <slot></slot>
    </column>
    <column gap="1" style(nums)>
      <for {boxList() as line} key={line.k}>
        <p style(meta)>{line.t}</p>
      </for>
    </column>
  </row>
</column>
