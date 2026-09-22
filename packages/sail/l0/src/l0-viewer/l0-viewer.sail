<scope>
  style shell = { size({ w: 100%, h: 100vh }), overflow(hidden) }
  style main = { size(grow), overflow(auto), bg(surface) }
  style sub = { pad({ x: 5, y: 3 }), bg(muted) }
  style stage = { pad(5) }
</scope>

<controller>
  type Link = { href: string; label: string }
  type Group = { id: string; label: string; href: string; members: Link[] }

  const groups: Group[] = [
    { id: 'hosts', label: 'Hosts', href: '/', members: [] },
    { id: 'traps', label: 'Traps', href: '/poison', members: [] },
    {
      id: 'spacing',
      label: 'Spacing',
      href: '/layout/gap',
      members: [
        { href: '/layout/gap', label: 'Gap' },
        { href: '/layout/pad', label: 'Pad' },
        { href: '/layout/margin', label: 'Margin' },
        { href: '/layout/box', label: 'Box' },
      ],
    },
    {
      id: 'fit',
      label: 'Fit',
      href: '/layout/participate',
      members: [
        { href: '/layout/participate', label: 'Hug / grow' },
        { href: '/layout/align', label: 'Align' },
        { href: '/layout/cluster', label: 'Cluster' },
        { href: '/layout/measure', label: 'Measure' },
      ],
    },
    {
      id: 'flow',
      label: 'Flow',
      href: '/layout/when',
      members: [
        { href: '/layout/when', label: 'When' },
        { href: '/layout/overflow', label: 'Overflow' },
        { href: '/layout/absolute', label: 'Absolute' },
        { href: '/layout/grid', label: 'Grid' },
      ],
    },
    {
      id: 'paint',
      label: 'Paint',
      href: '/paint/bg',
      members: [
        { href: '/paint/bg', label: 'Bg' },
        { href: '/paint/fg', label: 'Fg' },
        { href: '/paint/border', label: 'Border' },
        { href: '/paint/radius', label: 'Radius' },
        { href: '/paint/elevation', label: 'Elevation' },
        { href: '/paint/filter', label: 'Filter' },
        { href: '/paint/type', label: 'Type' },
        { href: '/paint/gradient', label: 'Gradient' },
        { href: '/paint/structured', label: 'Structured' },
        { href: '/paint/bind', label: 'Bind' },
      ],
    },
    { id: 'seeds', label: 'Patterns', href: '/layout/seeds', members: [] },
    {
      id: 'inspect',
      label: 'Inspect',
      href: '/inspect/names',
      members: [
        { href: '/inspect/names', label: 'Names' },
        { href: '/inspect/sheet', label: 'Sheet' },
      ],
    },
    {
      id: 'break',
      label: 'Breakpoints',
      href: '/break/wide',
      members: [
        { href: '/break/wide', label: 'Wide' },
        { href: '/break/tablet', label: 'Tablet' },
        { href: '/break/token', label: 'Token' },
      ],
    },
    {
      id: 'ramp',
      label: 'Type ramp',
      href: '/ramp/heading',
      members: [
        { href: '/ramp/heading', label: 'Headings' },
      ],
    },
  ]

  function groupFor(p: string): Group {
    const hosts = groups[0]
    const traps = groups[1]
    const seeds = groups[6]
    if (!hosts) throw new Error('hosts')
    if (p === '/' || p === '/layout') return hosts
    if (p === '/poison') return traps ?? hosts
    if (p === '/layout/seeds') return seeds ?? hosts
    for (const g of groups) {
      if (g.href === p || g.members.some((m) => m.href === p)) return g
    }
    return hosts
  }

  export default controller((ctx: { route?: { pathname?: () => string } }) => {
    const path = derived(() => ctx.route?.pathname?.() ?? '')
    const group = derived(() => groupFor(path()))
    const links = derived(() => group().members)
    return { path, group, links, groups }
  })
</controller>

<row nowrap align="stretch" style(shell)>
  <navbar placement="rail" bordered>
    <a slot="brand" href="/" variant="ghost">L0</a>
    <column slot="center" gap="1">
      <for {groups as g} key={g.id}>
        <a
          href={g.href}
          variant={group().id === g.id ? 'soft' : 'ghost'}
          current={path() === g.href && links().length === 0 ? 'page' : null}
        >{g.label}</a>
      </for>
    </column>
    <theme-picker slot="actions" size="compact"></theme-picker>
  </navbar>
  <column style(main)>
    <if {links().length > 0}>
      <row cluster gap="2" style(sub)>
        <for {links() as m} key={m.href}>
          <a
            href={m.href}
            variant={path() === m.href ? 'soft' : 'text'}
            current={path() === m.href ? 'page' : null}
          >{m.label}</a>
        </for>
      </row>
    </if>
    <column gap="6" style(stage)>
      <slot name="route"></slot>
    </column>
  </column>
</row>
