<!--
  <tab-list> — strip chrome. Sticky + variant Style. Orientation structural stamp from use:tabs.
  Paint: strip/rail from background / border
         (cssNames tab-list, border, muted).
-->

<scope>
  props: {
    sticky = false
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    variant: 'line' | 'pill' | 'enclosed' = 'line'
  }

  style base = {paint: border({ color: border, edge: l }).border({ color: border, edge: r }).border({ color: border, edge: t }).border({ color: border, width: 1, edge: b }).fill({ color: background })}
  // line + enclosed share the bottom-edge strip chrome.
  style variantStrip = {paint: border({ color: border, edge: l }).border({ color: border, edge: r }).border({ color: border, edge: t }).border({ color: border, width: 1, edge: b })}
  style pill = {box: space({ pad: 1 }), paint: border({ width: 0, edge: l }).border({ width: 0, edge: r }).border({ width: 0, edge: t }).border({ width: 0, edge: b }).radius(md).fill({ color: muted })}
</scope>

<row
  align="stretch"
  gap="1"
  style(base, {match: {
      variant: {
        line: variantStrip,
        pill: pill,
        enclosed: variantStrip,
      },
    }, compound: [
      {sticky: true, box: place({ position: sticky, top: 0, z: sticky })},
      {orientation: vertical, paint: border({ color: border, edge: s }).border({ color: border, edge: t }).border({ color: border, width: 0, edge: b }).border({ color: border, width: 1, edge: e })},
      {orientation: vertical, variant: pill, paint: border({ width: 0, edge: e })},
    ]})
>
  <slot></slot>
</row>
