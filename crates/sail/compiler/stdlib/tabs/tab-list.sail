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

  style base = {border({ color: border, edge: l }).border({ color: border, edge: r }).border({ color: border, edge: t }).border({ color: border, width: 1, edge: b }).bg(background)}
  // line + enclosed share the bottom-edge strip chrome.
  style variantStrip = {border({ color: border, edge: l }).border({ color: border, edge: r }).border({ color: border, edge: t }).border({ color: border, width: 1, edge: b })}
  style pill = {pad(1).border({ width: 0, edge: l }).border({ width: 0, edge: r }).border({ width: 0, edge: t }).border({ width: 0, edge: b }).radius(md).bg(muted)}
  style horiz = {dir(row)}
  style vert = {dir(column).shrink(0)}
</scope>

<row
  align="stretch"
  gap="1"
  style(base, {match: { variant: { line: variantStrip, pill: pill, enclosed: variantStrip }, orientation: { horizontal: horiz, vertical: vert } }, compound: [
      {sticky: true, position({ scheme: sticky, t: 0, z: sticky })},
      {orientation: vertical, border({ color: border, edge: s }).border({ color: border, edge: t }).border({ color: border, width: 0, edge: b }).border({ color: border, width: 1, edge: e })},
      {orientation: vertical, variant: pill, border({ width: 0, edge: e })},
    ]})
>
  <slot></slot>
</row>
