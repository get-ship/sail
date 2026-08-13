<!--
  <kbd> — mono keycap chip. Style product only.
  Paint: muted chip, mono type, doubled bottom edge (physical key affordance).
-->

<scope>
  props: {}

  style base = {box: flow({ display: inline-block }).space({ pad-y: px, pad-x: 1_5 }), paint: fill({ color: kbd }).border({ color: kbd.border, width: 1, edge: l }).border({ color: kbd.border, width: 1, edge: r }).border({ color: kbd.border, width: 1, edge: t }).border({ color: kbd.border, width: 2, edge: b }).radius(sm).ink({ color: kbd.foreground }).align(middle), text: type({ family: mono, size: xs, leading: none, wrap: nowrap })}
</scope>

<kbd style(base)>
  <slot></slot>
</kbd>
