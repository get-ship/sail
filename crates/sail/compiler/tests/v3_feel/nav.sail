<scope>
  style topbar = {
    position({ scheme: sticky, t: 0, z: header }).size(full).pad({ x: 4 })
      .bg(background).border({ color: border-subtle, edge: bottom }),
    animate: { transition: play(colors, fast) },
  }
</scope>

<navbar style(topbar)>
  <row gap="1" align="center">
    <a href="/">sail</a>
    <a href="/docs" style(pad({ x: 3, y: 2 }).radius(md).type(label), when: { hover: bg(surface-2), current: bg(primary).fg(primary-foreground) })>Docs</a>
    <a href="/components" style(pad({ x: 3, y: 2 }).radius(md).type(label), when: { hover: bg(surface-2), current: bg(primary).fg(primary-foreground) })>Components</a>
    <a href="/playground" style(pad({ x: 3, y: 2 }).radius(md).type(label), when: { hover: bg(surface-2), current: bg(primary).fg(primary-foreground) })>Playground</a>
    <row gap="2" style(margin({ l: auto }))>
      <theme-picker />
      <button size="sm">Get started</button>
    </row>
  </row>
</navbar>
