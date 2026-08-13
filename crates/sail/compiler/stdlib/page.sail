<!--
  <page> — size match owns cell geometry (author size on each page).
  Paint from component.page tokens (page.foreground, hover-bg, current-*).
-->

<scope>
  props: {
    value = ''
    disabled = false
    size: 'md' | 'sm' | 'lg' = 'md'
  }

  style base = {box: space({ pad-x: 2 }), paint: ink({ color: page.foreground }).fill({ color: transparent }).radius(md).border({ color: transparent, width: 1 }).motion({ property: colors, duration: fast, ease: out }).cursor(pointer).select(none), text: type({ family: body, weight: medium })}
  style sizeMd = {box: size({ min-w: control-h-md, h: control-h-md }), text: type({ size: sm })}
  style sizeSm = {box: size({ min-w: control-h-sm, h: control-h-sm }), text: type({ size: xs })}
  style sizeLg = {box: size({ min-w: control-h-lg, h: control-h-lg }), text: type({ size: base })}
</scope>

<button
  type="button"
  style(base, {when: {hover: {paint: fill({ color: hover-bg })}, current: {paint: fill({ color: page.current-bg }).ink({ color: page.current-fg }).border({ color: page.current-bg })}, disabled: {paint: filter({ opacity: 50 }).cursor(not-allowed)}, focus: {paint: border({ style: solid, kind: outline, width: 2, color: ring })}}, match: { size: { md: sizeMd, sm: sizeSm, lg: sizeLg } }})
>
  <slot></slot>
</button>
