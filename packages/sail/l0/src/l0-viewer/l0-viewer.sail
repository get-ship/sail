<scope>
  style shell = { pad({ x: 6, y: 6 }) }
  style stage = { pad(6), bg(muted) }
  style meta = { type(mono), fg(muted) }
</scope>

<controller>
  export default controller((_s, host) => {
    const boxes = signal('')
    const tick = (): void => {
      if (!host) return
      const out: string[] = []
      const walk = (n: Element): void => {
        for (const c of Array.from(n.children)) {
          if (c.localName === 'theme-picker' || c.localName === 'p') {
            walk(c)
            continue
          }
          const r = c.getBoundingClientRect()
          const bg = getComputedStyle(c).backgroundColor
          const painted =
            bg !== 'rgba(0, 0, 0, 0)' && bg !== 'transparent' && r.width > 0 && r.height > 0
          if (painted) {
            out.push(
              `${c.localName} ${Math.round(r.width)}×${Math.round(r.height)} @${Math.round(r.x)},${Math.round(r.y)}`,
            )
          }
          walk(c)
        }
      }
      walk(host)
      boxes.set(out.slice(0, 16).join('  '))
    }
    queueMicrotask(tick)
    if (host && typeof ResizeObserver !== 'undefined') {
      new ResizeObserver(tick).observe(host)
    }
    return { boxes }
  })
</controller>

<column gap="4" style(shell)>
  <theme-picker chrome="inline" size="compact"></theme-picker>
  <column gap="4" style(stage)>
    <center>
      <slot></slot>
    </center>
  </column>
  <p style(meta)>{boxes()}</p>
</column>
