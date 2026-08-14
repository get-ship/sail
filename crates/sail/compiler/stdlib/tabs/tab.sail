<!--
  <tab> — paint + variant/size Style match for pill/enclosed and pad scale.
  Paint: idle/selected from muted-foreground / foreground / muted
         (cssNames muted-foreground, foreground, muted).
-->

<scope>
  props: {
    name = ''
    disabled = false
    variant: 'line' | 'pill' | 'enclosed' = 'line'
    size: 'md' | 'sm' | 'lg' = 'md'
  }

  style base = {pad({ y: 2, x: 3 }).fg(muted-foreground).bg(transparent).border({ width: 0, edge: l }).border({ width: 0, edge: r }).border({ width: 0, edge: t }).border({ width: 2, edge: b, color: transparent }).radius({ t: sm }).cursor(pointer).select(none).tapHighlight(none).type({ family: body, weight: medium, leading: none, wrap: nowrap, size: sm }), animate: { transition: play(colors, fast) }}
  style sizeMd = {pad({ y: 2, x: 3 }).type({ size: sm })}
  style sizeSm = {pad({ y: 1_5, x: 2_5 }).type({ size: xs })}
  style sizeLg = {pad({ y: 2_5, x: 4 }).type({ size: base })}
  style pill = {border({ width: 0, edge: b }).radius(sm)}
  style enclosed = {border({ color: transparent, width: 1, edge: l }).border({ color: transparent, width: 1, edge: r }).border({ color: transparent, width: 1, edge: t }).border({ color: transparent, width: 0, edge: b }).radius({ t: md })}
</scope>

<button
  type="button"
  style(base, {when: {hover: fg(foreground), selected: fg(foreground).border({ color: primary, edge: b }).type({ weight: semibold }), focus: border({ style: solid, kind: outline, width: ring-width, color: ring, offset: ring-inset }), disabled: filter({ opacity: 50 }).cursor(not-allowed)}, match: {size: { md: sizeMd, sm: sizeSm, lg: sizeLg }, variant: {line: border({ color: transparent, edge: b }), pill: pill, enclosed: enclosed}}, compound: [
      {variant: pill, when: {selected: fg(foreground).bg(muted).elevation(xs).border({ color: transparent, edge: b })}},
      {variant: enclosed, when: {selected: fg(foreground).border({ color: border }).bg(muted).translateY(px)}},
    ]})
>
  <slot></slot>
</button>
