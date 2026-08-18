<!--
  One public chrome. Navbar rail + leftover stage + box readout.
  Routes stay Style-less wrappers. Cells stay painted boxes.
-->
<scope>
  style shell = { size({ w: 100%, h: 100% }) }
  style stage = { size(grow), pad(5), overflow(auto), bg(surface) }
  style title = { type(heading) }
  style meta = { type(mono), fg(muted) }
  style railLabel = { type(label), fg(muted) }
</scope>

<controller>
  function titleFor(p: string): string {
    if (p === '/') return 'L0 gauntlet'
    if (p === '/poison') return 'poison'
    if (p === '/layout') return 'hosts'
    if (p === '/layout/gap') return 'gap'
    if (p === '/layout/pad') return 'pad'
    if (p === '/layout/margin') return 'margin'
    if (p === '/layout/box') return 'box'
    if (p === '/layout/participate') return 'participate'
    if (p === '/layout/align') return 'align'
    if (p === '/layout/cluster') return 'cluster'
    if (p === '/layout/measure') return 'measure'
    if (p === '/layout/when') return 'when'
    if (p === '/layout/overflow') return 'overflow'
    if (p === '/layout/absolute') return 'absolute'
    if (p === '/layout/grid') return 'grid'
    if (p === '/layout/seeds') return 'seeds'
    return 'L0'
  }

  export default controller((ctx: { route?: { pathname?: () => string } }, host) => {
    const path = derived(() => ctx.route?.pathname?.() ?? '')
    const title = derived(() => titleFor(path()))
    const boxList = signal<{ k: string; t: string }[]>([])
    const tick = (): void => {
      if (!host) return
      const root = host.querySelector('[data-l0-cells]')
      if (!root) return
      const out: { k: string; t: string }[] = []
      const walk = (n: Element): void => {
        for (const c of Array.from(n.children)) {
          if (c.localName === 'p') continue
          const r = c.getBoundingClientRect()
          const bg = getComputedStyle(c).backgroundColor
          const painted =
            bg !== 'rgba(0, 0, 0, 0)' && bg !== 'transparent' && r.width > 0 && r.height > 0
          if (painted) {
            out.push({
              k: String(out.length),
              t: `${c.localName} ${Math.round(r.width)}×${Math.round(r.height)} @${Math.round(r.x)},${Math.round(r.y)}`,
            })
          }
          walk(c)
        }
      }
      walk(root)
      boxList.set(out)
    }
    queueMicrotask(tick)
    if (host && typeof ResizeObserver !== 'undefined') {
      const root = host.querySelector('[data-l0-cells]')
      new ResizeObserver(tick).observe(root ?? host)
    }
    return { path, title, boxList }
  })
</controller>

<row nowrap align="stretch" style(shell)>
  <navbar placement="rail" bordered expanded>
    <a
      slot="brand"
      href="/"
      variant={path() === '/' ? 'soft' : 'ghost'}
      current={path() === '/' ? 'page' : null}
    >L0</a>
    <column slot="center" gap="1">
      <a href="/" variant={path() === '/' ? 'soft' : 'ghost'} current={path() === '/' ? 'page' : null}>L0</a>
      <a href="/poison" variant={path() === '/poison' ? 'soft' : 'ghost'} current={path() === '/poison' ? 'page' : null}>poison</a>
      <a href="/layout" variant={path() === '/layout' ? 'soft' : 'ghost'} current={path() === '/layout' ? 'page' : null}>hosts</a>
      <separator></separator>
      <a href="/layout/gap" variant={path() === '/layout/gap' ? 'soft' : 'ghost'} current={path() === '/layout/gap' ? 'page' : null}>gap</a>
      <a href="/layout/pad" variant={path() === '/layout/pad' ? 'soft' : 'ghost'} current={path() === '/layout/pad' ? 'page' : null}>pad</a>
      <a href="/layout/margin" variant={path() === '/layout/margin' ? 'soft' : 'ghost'} current={path() === '/layout/margin' ? 'page' : null}>margin</a>
      <a href="/layout/box" variant={path() === '/layout/box' ? 'soft' : 'ghost'} current={path() === '/layout/box' ? 'page' : null}>box</a>
      <separator></separator>
      <a href="/layout/participate" variant={path() === '/layout/participate' ? 'soft' : 'ghost'} current={path() === '/layout/participate' ? 'page' : null}>participate</a>
      <a href="/layout/align" variant={path() === '/layout/align' ? 'soft' : 'ghost'} current={path() === '/layout/align' ? 'page' : null}>align</a>
      <a href="/layout/cluster" variant={path() === '/layout/cluster' ? 'soft' : 'ghost'} current={path() === '/layout/cluster' ? 'page' : null}>cluster</a>
      <a href="/layout/measure" variant={path() === '/layout/measure' ? 'soft' : 'ghost'} current={path() === '/layout/measure' ? 'page' : null}>measure</a>
      <separator></separator>
      <a href="/layout/when" variant={path() === '/layout/when' ? 'soft' : 'ghost'} current={path() === '/layout/when' ? 'page' : null}>when</a>
      <a href="/layout/overflow" variant={path() === '/layout/overflow' ? 'soft' : 'ghost'} current={path() === '/layout/overflow' ? 'page' : null}>overflow</a>
      <a href="/layout/absolute" variant={path() === '/layout/absolute' ? 'soft' : 'ghost'} current={path() === '/layout/absolute' ? 'page' : null}>absolute</a>
      <a href="/layout/grid" variant={path() === '/layout/grid' ? 'soft' : 'ghost'} current={path() === '/layout/grid' ? 'page' : null}>grid</a>
      <separator></separator>
      <a href="/layout/seeds" variant={path() === '/layout/seeds' ? 'soft' : 'ghost'} current={path() === '/layout/seeds' ? 'page' : null}>seeds</a>
    </column>
    <theme-picker slot="actions" size="compact"></theme-picker>
  </navbar>
  <column gap="5" style(stage)>
    <p style(title)>{title()}</p>
    <column gap="6" data-l0-cells>
      <slot></slot>
    </column>
    <column gap="1">
      <p style(railLabel)>boxes</p>
      <for {boxList() as line} key={line.k}>
        <p style(meta)>{line.t}</p>
      </for>
    </column>
  </column>
</row>
