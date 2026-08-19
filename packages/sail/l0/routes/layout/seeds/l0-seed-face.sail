<scope>
  props: {
    kind: 'box4' | 'box4a' | 'box4m' | 'leftover' | 'rail' | 'fluid' | 'header' | 'footer' | 'nav' | 'main' | 'aside' | 'pct-parent' | 'pct-child' | 'span-a' | 'span-b' | 'span-c' | 'overlap-a' | 'overlap-b' | 'fill' | 'aspect-grow' = 'box4'
  }
  style box4 = { size({ w: 8, h: 8 }), bg(primary) }
  style box4a = { size({ w: 8, h: 8 }), bg(accent) }
  style box4m = { size({ w: 8, h: 8 }), bg(muted) }
  style leftover = { size(grow), pad(3), bg(accent) }
  style rail = { size({ w: 16 }), pad(3), bg(primary) }
  style fluid = { size(grow), pad(3), bg(accent) }
  style header = { size({ h: 8 }), pad(3), bg(primary) }
  style footer = { size({ h: 8 }), pad(3), bg(muted) }
  style nav = { size({ w: 16 }), pad(3), bg(primary) }
  style main = { size(grow), pad(3), bg(accent) }
  style aside = { size({ w: 16 }), pad(3), bg(muted) }
  style pctParent = { size({ w: 50% }), pad(3), bg(muted) }
  style pctChild = { size({ w: 50% }), pad(3), bg(primary) }
  style spanA = { pad(3), bg(primary) }
  style spanB = { pad(3), bg(accent) }
  style spanC = { pad(3), bg(muted) }
  style overlapA = { size({ w: 16, h: 16 }), bg(primary) }
  style overlapB = { size({ w: 16, h: 16 }), bg(accent) }
  style fill = { size({ w: 100% }) }
  style aspectGrow = { size(grow), size({ aspect: square }), bg(primary) }
</scope>
<column style({ match: { kind: { box4: box4, box4a: box4a, box4m: box4m, leftover: leftover, rail: rail, fluid: fluid, header: header, footer: footer, nav: nav, main: main, aside: aside, 'pct-parent': pctParent, 'pct-child': pctChild, 'span-a': spanA, 'span-b': spanB, 'span-c': spanC, 'overlap-a': overlapA, 'overlap-b': overlapB, fill: fill, 'aspect-grow': aspectGrow } } })>
  <slot></slot>
</column>
