<!--
  <toggle> — pressed-button cell for <toggle-group> only (standalone died).
  First/last join + variant/orientation/size Style.
  Binary form control → use <switch>. Action button → use <button>.
  Selected = primary tokens (not raw fg/bg invert); pressed+hover keeps identity.
-->

<scope>
  props: {
    value = ''
    disabled = false
    variant: 'outline' | 'ghost' | 'segmented' = 'outline'
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    size: 'md' | 'sm' | 'lg' = 'md'
  }

  // border(solid) is required — bare <button> UA is border-style:outset; without
  // it the outline join paints as separate browser chrome (not a segment).
  style base = {position(relative).display(inline-flex).size({ h: control-h-md }).pad({ x: 3 }).margin({ s: -px }).appearance(none).fg(foreground).bg(background).border({ color: border, width: 1, style: solid }).radius(none).cursor(pointer).select(none).tapHighlight(none).type({ family: body, weight: medium, leading: none, wrap: nowrap, size: sm }), animate: { transition: play(colors, fast) }}
  style sizeMd = {size({ h: control-h-md }).pad({ x: 3 }).type({ size: sm })}
  style sizeSm = {size({ h: control-h-sm }).pad({ x: 2_5 }).type({ size: xs })}
  style sizeLg = {size({ h: control-h-lg }).pad({ x: 4 }).type({ size: base })}
  style outline = {margin({ s: -px }).radius(none)}
  style ghost = {margin({ s: 0, e: 1 }).border({ color: transparent }).radius(md)}
  style segmented = {margin({ s: 0, e: 1 }).border({ color: transparent }).radius(sm)}
  // Vertical cells share the column width (widest label). Match orientation —
  // do not require variant= on the cell (docs omit it; default is outline).
  style vertCell = {size({ w: 100% }).self(stretch)}
</scope>

<button
  type="button"
  style(base, {when: {hover: bg(hover-bg).fg(foreground), pressed: {bg(primary).fg(primary-foreground).border({ color: primary }), when: {hover: bg(primary).fg(primary-foreground).border({ color: primary }).filter({ opacity: 90 })}, position({ z: 1 })}, disabled: filter({ opacity: 50 }).cursor(not-allowed), focus: border({ style: solid, kind: outline, width: 2, color: ring }).position({ z: 2 })}, match: { size: { md: sizeMd, sm: sizeSm, lg: sizeLg }, variant: { outline: outline, ghost: ghost, segmented: segmented }, orientation: { vertical: vertCell } }, compound: [
      {orientation: vertical, variant: outline, margin({ t: -px, s: 0, e: 0 }).radius(none).size({ w: 100% }).self(stretch)},
      {variant: ghost, margin({ s: 0, t: 0, e: 1 }).border({ color: transparent }).radius(md)},
      {variant: segmented, margin({ s: 0, t: 0, e: 1 }).border({ color: transparent }).radius(sm)},
      {orientation: vertical, variant: ghost, margin({ s: 0, t: 0, e: 0, b: 1 }).border({ color: transparent }).radius(md).size({ w: 100% }).self(stretch)},
      {orientation: vertical, variant: segmented, margin({ s: 0, t: 0, e: 0, b: 1 }).border({ color: transparent }).radius(sm).size({ w: 100% }).self(stretch)},
      {orientation: horizontal, variant: outline, when: {first: margin({ s: 0 }).radius({ l: md }), last: radius({ r: md })}},
      {orientation: vertical, variant: outline, when: {first: margin({ t: 0 }).radius({ t: md }), last: radius({ b: md })}},
      {variant: ghost, when: {last: margin({ e: 0 })}},
      {variant: segmented, when: {last: margin({ e: 0 })}},
      {orientation: vertical, variant: ghost, when: {last: margin({ b: 0 })}},
      {orientation: vertical, variant: segmented, when: {last: margin({ b: 0 })}},
    ]})
>
  <slot></slot>
</button>
