<!--
  <kbd> — mono keycap chip. Style product only.
  Paint: muted chip, mono type, doubled bottom edge (physical key affordance).
-->

<scope>
  props: {}

  style base = {display(inline-block).pad({ y: px, x: 1_5 }).bg(muted).border({ color: border, width: 1, edge: l }).border({ color: border, width: 1, edge: r }).border({ color: border, width: 1, edge: t }).border({ color: border, width: 2, edge: b }).radius(sm).fg(foreground).align(middle).type({ family: mono, size: xs, leading: none, wrap: nowrap })}
</scope>

<kbd role="group" style(base)>
  <slot></slot>
</kbd>
