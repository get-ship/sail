<!--
  <kbd> — mono keycap chip. Style product only.
  Paint: muted chip, mono type, doubled bottom edge (physical key affordance).
-->

<scope>
  props: {}

  style base = {box: flow({ display: inline-block }).space({ pad-y: px, pad-x: 1_5 }), paint: fill({ color: muted }).border({ color: border, width: 1, edge: l }).border({ color: border, width: 1, edge: r }).border({ color: border, width: 1, edge: t }).border({ color: border, width: 2, edge: b }).radius(sm).ink({ color: foreground }).align(middle), text: type({ family: mono, size: xs, leading: none, wrap: nowrap })}
</scope>

<kbd style(base)>
  <slot></slot>
</kbd>
