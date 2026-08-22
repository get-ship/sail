<!--
  <tabs> — nowrap row. Horizontal is dir(column): rail on top, panel
  under. Vertical is dir(row): rail on the start edge, panel beside.

  Nested API:
    <tabs>
      <tab title="Overview">panel…</tab>
    </tabs>

  Look: this host owns layout. Each <tab> owns its Style. Slots, not :part.
  State: this file's <controller> (active + keys + select). No DOM.
  Child tabs take `active` / `select` as props from this instance (signal
  cell, not a snapshot). Listen: use:hotkeys={keys} on the host.
-->
<scope>
  props: {
    active = ''
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    variant: 'line' | 'pill' | 'enclosed' = 'line'
    onTabChange: (detail: { name: string }) => void
  }

  style host = {size({ min-w: 0 })}
  style horiz = {dir(column)}
  style vert = {dir(row)}
</scope>

<controller>
export default controller((init) => {
  const names = Array.isArray(init.names) ? init.names : []
  const active = signal(String(init.active || names[0] || ''))

  function select(name) {
    if (!name || name === active()) return
    active.set(name)
    const cb = init.onTabChange
    if (typeof cb === 'function') cb({ name })
  }
  function move(delta) {
    if (!names.length) return
    let i = names.indexOf(active())
    if (i < 0) i = 0
    const next = names[(i + delta + names.length) % names.length]
    if (next) select(next)
  }
  const keys = {
    ArrowRight: () => move(1),
    ArrowLeft: () => move(-1),
    ArrowDown: () => move(1),
    ArrowUp: () => move(-1),
    Home: () => { if (names[0]) select(names[0]) },
    End: () => { const n = names[names.length - 1]; if (n) select(n) },
  }
  return { active, keys, select }
})
</controller>

<row nowrap gap="3" use:hotkeys={keys} style(host, {match: { orientation: { horizontal: horiz, vertical: vert } }})>
  <slot></slot>
</row>
