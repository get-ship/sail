<scope>
  style shell = { size({ w: 100%, min-h: 100vh }) }
  style stage = { size(grow), pad(5), overflow(auto), bg(surface) }
  style sub = { pad({ x: 5, y: 3 }), bg(muted) }
</scope>

<controller>
  type Link = { href: string; label: string }
  type Group = { id: string; label: string; href: string; members: Link[] }

  const groups: Group[] = [
    { id: 'home', label: 'L0', href: '/', members: [] },
    { id: 'poison', label: 'poison', href: '/poison', members: [] },
    { id: 'hosts', label: 'hosts', href: '/layout', members: [] },
    {
      id: 'space',
      label: 'space',
      href: '/layout/gap',
      members: [
        { href: '/layout/gap', label: 'gap' },
        { href: '/layout/pad', label: 'pad' },
        { href: '/layout/margin', label: 'margin' },
        { href: '/layout/box', label: 'box' },
      ],
    },
    {
      id: 'structure',
      label: 'structure',
      href: '/layout/participate',
      members: [
        { href: '/layout/participate', label: 'participate' },
        { href: '/layout/align', label: 'align' },
        { href: '/layout/cluster', label: 'cluster' },
        { href: '/layout/measure', label: 'measure' },
      ],
    },
    {
      id: 'flow',
      label: 'flow',
      href: '/layout/when',
      members: [
        { href: '/layout/when', label: 'when' },
        { href: '/layout/overflow', label: 'overflow' },
        { href: '/layout/absolute', label: 'absolute' },
        { href: '/layout/grid', label: 'grid' },
      ],
    },
    {
      id: 'paint',
      label: 'paint',
      href: '/paint/gradient',
      members: [
        { href: '/paint/gradient', label: 'gradient' },
        { href: '/paint/structured', label: 'structured' },
        { href: '/paint/bind', label: 'bind' },
      ],
    },
    { id: 'seeds', label: 'seeds', href: '/layout/seeds', members: [] },
  ]

  function groupFor(p: string): Group {
    if (p === '/') return groups[0]
    if (p === '/poison') return groups[1]
    if (p === '/layout') return groups[2]
    if (p === '/layout/seeds') return groups[7]
    for (const g of groups) {
      if (g.members.some((m) => m.href === p)) return g
    }
    return groups[0]
  }

  export default controller((ctx: { route?: { pathname?: () => string } }) => {
    const path = derived(() => ctx.route?.pathname?.() ?? '')
    const group = derived(() => groupFor(path()))
    const links = derived(() => group().members)
    return { path, group, links, groups }
  })
</controller>

<column style(shell)>
  <navbar placement="top" bordered>
    <a
      slot="brand"
      href="/"
      variant={path() === '/' ? 'soft' : 'ghost'}
      current={path() === '/' ? 'page' : null}
    >L0</a>
    <row slot="center" nowrap gap="1" align="center">
      <for {groups as g} key={g.id}>
        <a
          href={g.href}
          variant={group().id === g.id ? 'soft' : 'ghost'}
          current={group().id === g.id ? 'page' : null}
        >{g.label}</a>
      </for>
    </row>
    <theme-picker slot="actions" size="compact"></theme-picker>
  </navbar>
  <if {links().length > 0}>
    <row cluster gap="2" style(sub)>
      <for {links() as m} key={m.href}>
        <a
          href={m.href}
          variant={path() === m.href ? 'soft' : 'ghost'}
          current={path() === m.href ? 'page' : null}
        >{m.label}</a>
      </for>
    </row>
  </if>
  <column gap="6" style(stage)>
    <slot></slot>
  </column>
</column>
