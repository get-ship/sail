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
  style base = {box: place({ position: relative }).flow({ display: inline-flex }).size({ h: control-h-md }).space({ pad-x: 3, ms: -px }), paint: appearance(none).ink({ color: toggle.foreground }).fill({ color: toggle }).border({ color: toggle.border, width: 1, style: solid }).radius(none).motion({ property: colors, duration: fast, ease: out }).cursor(pointer).select(none).tapHighlight(none), text: type({ family: body, weight: medium, leading: none, wrap: nowrap, size: sm })}
  style sizeMd = {box: size({ h: control-h-md }).space({ pad-x: 3 }), text: type({ size: sm })}
  style sizeSm = {box: size({ h: control-h-sm }).space({ pad-x: 2_5 }), text: type({ size: xs })}
  style sizeLg = {box: size({ h: control-h-lg }).space({ pad-x: 4 }), text: type({ size: base })}
  style outline = {box: space({ ms: -px }), paint: radius(none)}
  style ghost = {box: space({ ms: 0, me: 1 }), paint: border({ color: transparent }).radius(md)}
  style segmented = {box: space({ ms: 0, me: 1 }), paint: border({ color: transparent }).radius(sm)}
</scope>

<button
  type="button"
  style(base, {when: {hover: {paint: fill({ color: hover-bg }).ink(foreground)}, pressed: {paint: fill({ color: primary }).ink({ color: primary-foreground }).border({ color: primary }), when: {// Selected must survive hover (not hover-bg wash / unselected look).
          hover: {paint: fill({ color: primary }).ink({ color: primary-foreground }).border({ color: primary }).filter({ opacity: 90 })}}, box: place({ z: 1 })}, disabled: {paint: filter({ opacity: 50 }).cursor(not-allowed)}, focus: {paint: border({ style: solid, kind: outline, width: 2, color: ring }), box: place({ z: 2 })}}, match: {
      size: { md: sizeMd, sm: sizeSm, lg: sizeLg },
      variant: { outline: outline, ghost: ghost, segmented: segmented },
    }, compound: [
      {orientation: vertical, variant: outline, box: space({ mt: -px, ms: 0, me: 0 }), paint: radius(none)},
      {variant: ghost, box: space({ ms: 0, mt: 0, me: 1 }), paint: border({ color: transparent }).radius(md)},
      {variant: segmented, box: space({ ms: 0, mt: 0, me: 1 }), paint: border({ color: transparent }).radius(sm)},
      {orientation: vertical, variant: ghost, box: space({ ms: 0, mt: 0, me: 0, mb: 1 }), paint: border({ color: transparent }).radius(md)},
      {orientation: vertical, variant: segmented, box: space({ ms: 0, mt: 0, me: 0, mb: 1 }), paint: border({ color: transparent }).radius(sm)},
      {orientation: horizontal, variant: outline, when: {first: {box: space({ ms: 0 }), paint: radius({ l: md })}, last: {paint: radius({ r: md })}}},
      {orientation: vertical, variant: outline, when: {first: {box: space({ mt: 0 }), paint: radius({ t: md })}, last: {paint: radius({ b: md })}}},
      {variant: ghost, when: {last: {box: space({ me: 0 })}}},
      {variant: segmented, when: {last: {box: space({ me: 0 })}}},
      {orientation: vertical, variant: ghost, when: {last: {box: space({ mb: 0 })}}},
      {orientation: vertical, variant: segmented, when: {last: {box: space({ mb: 0 })}}},
    ]})
>
  <slot></slot>
</button>
