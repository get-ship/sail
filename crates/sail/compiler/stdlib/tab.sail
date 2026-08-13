<!--
  <tab> — paint + variant/size Style match for pill/enclosed and pad scale.
  Paint: idle/selected from component.tab tokens
         (cssNames tab.foreground, tab.selected-fg, tab.selected-bg).
-->

<scope>
  props: {
    name = ''
    disabled = false
    variant: 'line' | 'pill' | 'enclosed' = 'line'
    size: 'md' | 'sm' | 'lg' = 'md'
  }

  style base = {box: space({ pad-y: 2, pad-x: 3 }), paint: ink({ color: tab.foreground }).fill({ color: transparent }).border({ width: 0, edge: l }).border({ width: 0, edge: r }).border({ width: 0, edge: t }).border({ width: 2, edge: b, color: transparent }).radius({ t: sm }).motion({ property: colors, duration: fast, ease: out }).cursor(pointer).select(none).tapHighlight(none), text: type({ family: body, weight: medium, leading: none, wrap: nowrap, size: sm })}
  style sizeMd = {box: space({ pad-y: 2, pad-x: 3 }), text: type({ size: sm })}
  style sizeSm = {box: space({ pad-y: 1_5, pad-x: 2_5 }), text: type({ size: xs })}
  style sizeLg = {box: space({ pad-y: 2_5, pad-x: 4 }), text: type({ size: base })}
  style pill = {paint: border({ width: 0, edge: b }).radius(sm)}
  style enclosed = {paint: border({ color: transparent, width: 1, edge: l }).border({ color: transparent, width: 1, edge: r }).border({ color: transparent, width: 1, edge: t }).border({ color: transparent, width: 0, edge: b }).radius({ t: md })}
</scope>

<button
  type="button"
  style(base, {when: {hover: {paint: ink({ color: tab.selected-fg })}, selected: {paint: ink({ color: tab.selected-fg }).border({ color: primary, edge: b }), text: type({ weight: semibold })}, focus: {paint: border({ style: solid, kind: outline, width: ring-width, color: ring, offset: ring-inset })}, disabled: {paint: filter({ opacity: 50 }).cursor(not-allowed)}}, match: {size: { md: sizeMd, sm: sizeSm, lg: sizeLg }, variant: {line: {paint: border({ color: transparent, edge: b })}, pill: pill, enclosed: enclosed}}, compound: [
      {variant: pill, when: {selected: {paint: ink({ color: tab.selected-fg }).fill({ color: tab.selected-bg }).elevation(xs).border({ color: transparent, edge: b })}}},
      {variant: enclosed, when: {selected: {paint: ink({ color: tab.selected-fg }).border({ color: tab.border }).fill({ color: tab.selected-bg }).motion({ y: px })}}},
    ]})
>
  <slot></slot>
</button>
