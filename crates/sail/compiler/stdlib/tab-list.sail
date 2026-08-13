<!--
  <tab-list> — strip chrome. Sticky + variant Style. Orientation structural stamp from use:tabs.
  Paint: strip/rail from component.tab-list tokens
         (cssNames tab-list, tab-list.border, tab-list.rail).
-->

<scope>
  props: {
    sticky = false
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    variant: 'line' | 'pill' | 'enclosed' = 'line'
  }

  style base = {paint: border({ color: tab-list.border, edge: l }).border({ color: tab-list.border, edge: r }).border({ color: tab-list.border, edge: t }).border({ color: tab-list.border, width: 1, edge: b }).fill({ color: tab-list })}
  // line + enclosed share the bottom-edge strip chrome.
  style variantStrip = {paint: border({ color: tab-list.border, edge: l }).border({ color: tab-list.border, edge: r }).border({ color: tab-list.border, edge: t }).border({ color: tab-list.border, width: 1, edge: b })}
  style pill = {box: space({ pad: 1 }), paint: border({ width: 0, edge: l }).border({ width: 0, edge: r }).border({ width: 0, edge: t }).border({ width: 0, edge: b }).radius(md).fill({ color: tab-list.rail })}
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
      {orientation: vertical, paint: border({ color: tab-list.border, edge: s }).border({ color: tab-list.border, edge: t }).border({ color: tab-list.border, width: 0, edge: b }).border({ color: tab-list.border, width: 1, edge: e })},
      {orientation: vertical, variant: pill, paint: border({ width: 0, edge: e })},
    ]})
>
  <slot></slot>
</row>
