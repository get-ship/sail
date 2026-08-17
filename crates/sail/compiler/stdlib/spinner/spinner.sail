<!--
  <spinner> — wait mark. Default variant is ring.
  Every variant spins (animate(spin) on the mark). dots/bars are three
  marks on the host box; the group rotates. Size is the icon ladder.
-->

<scope>
  props: {
    size: 'md' | 'xs' | 'sm' | 'lg' = 'md'
    variant: 'ring' | 'dots' | 'bars' = 'ring'
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' = 'info'
    centered = false
  }

  // Host owns the box. A child size(grow) of an unsized inline-flex collapses.
  // group() so children can when({ group-variant-* }) — child match { variant }
  // keys the child's own attr, so all three marks used to paint at once.
  style host = {display(inline-flex).self(start).align(middle).fg(tone).size(icon-size-lg).group()}
  style hide = {display(none)}
  style sizeXs = {size(icon-size-sm)}
  style sizeSm = {size(icon-size-md)}
  style sizeMd = {size(icon-size-lg)}
  style sizeLg = {size(icon-size-xl)}

  style ring = {display(block).size(grow).border({ color: border-subtle, width: control-stroke, style: solid, edge: b }).border({ color: border-subtle, width: control-stroke, style: solid, edge: l }).border({ color: border-subtle, width: control-stroke, style: solid, edge: t }).border({ color: currentColor, width: control-stroke, style: solid, edge: r }).radius(full).animate(spin)}

  style dotsHost = {display(none)}
  style dotsOn = {display(block).position(relative).size(grow).animate(spin)}
  style dot = {size(1_5).radius(full).bg(currentColor)}
  style dotA = {position({ scheme: absolute, t: 0, l: 0, r: 0 }).margin({ x: auto })}
  style dotB = {position({ scheme: absolute, b: 0, l: 0 })}
  style dotC = {position({ scheme: absolute, b: 0, r: 0 })}

  style barsHost = {display(none)}
  style barsOn = {display(block).position(relative).size(grow).animate(spin)}
  style bar = {size({ w: 1, h: 2 }).radius(sm).bg(currentColor)}
  style barA = {position({ scheme: absolute, t: 0, l: 0, r: 0 }).margin({ x: auto })}
  style barB = {position({ scheme: absolute, b: 0, l: 0 })}
  style barC = {position({ scheme: absolute, b: 0, r: 0 })}
</scope>

<span
  style(host, {match: { size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg } }, compound: [{centered: true, display(flex).margin({ y: 8, x: auto })}]})
>
  <span
    style(ring, {when: { group-variant-dots: hide, group-variant-bars: hide }})
  ></span>
  <span style(dotsHost, {when: { group-variant-dots: dotsOn }})>
    <span style(dot, dotA)></span>
    <span style(dot, dotB)></span>
    <span style(dot, dotC)></span>
  </span>
  <span style(barsHost, {when: { group-variant-bars: barsOn }})>
    <span style(bar, barA)></span>
    <span style(bar, barB)></span>
    <span style(bar, barC)></span>
  </span>
</span>
