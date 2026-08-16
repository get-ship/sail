<!--
  <button> — native <button>. L1 platform control. Style value language only.
  `default` is an alias of `solid` (prop default solid). tone defaults to
  primary — a bare <button> is a finished control, never an untoned invert.
  Explicit tone always paints. Sizes: xs–lg text; icon / iconSm / iconLg squares.
-->

<scope>
  props: {// default ≡ solid (kept as enum value so existing markup doesn't break).
    variant: 'default' | 'solid' | 'soft' | 'outline' | 'ghost' = 'solid'
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'primary'
    // icon / iconSm / iconLg = square hit targets (no hyphen — Style match keys are idents).
    size: 'xs' | 'sm' | 'md' | 'lg' | 'icon' | 'iconSm' | 'iconLg' = 'md'
    disabled = false
    loading = false
    pill = false
  }

  // border(solid) kills UA button outset so freeform demos use design-system chrome.
  // No base bg(transparent) or border(transparent): unlayered product atomics
  // (.s-bgtransparent / .s-bordertransparent) beat layered variant fills and
  // outline strokes. Variants own fill + border color; ghost is the transparent pair.
  // Cursor is NOT on base: base cursor(pointer) + when-disabled cursor(not-allowed) both
  // stamp classes and pointer was winning in the cascade. Match on the disabled prop so
  // only one cursor utility is active.
  // w/min-w max: a column stretches items and `.sail-col>*` zeros min-width.
  // A control hugs its label; authors who want a block button set size({ w: full }).
  // Icon+label pack is the inner <row cluster> — not Style gap(), not host CSS.
  style base = {display(inline-flex).position(relative).appearance(none).radius(md).border({ width: control-stroke, style: solid }).isolate().controlFrame().select(none).size({ w: max, min-w: max }).type({ weight: medium, leading: none, wrap: nowrap }), when: {hover: controlLift(), active: press()}, animate: { transition: play(control, fast) }}
  style pill = {radius(full)}
  style cursorOn = {cursor(pointer)}
  style cursorOff = {cursor(not-allowed).filter({ opacity: 50 })}

  style sizeXs = {size({ min-h: control-h-2xs }).pad({ x: 2 }).type({ size: xs })}
  style sizeSm = {size({ min-h: control-h-sm }).pad({ x: 2_5 }).type({ size: sm })}
  style sizeMd = {size({ min-h: control-h-md }).pad({ x: 3 }).type({ size: base })}
  style sizeLg = {size({ min-h: control-h-lg }).pad({ x: 4 }).type({ size: md })}
  // Square icon hit targets — explicit w/h (not minW: parent .sail-col zeros min-width).
  style sizeIcon = {size({ w: control-h-md, h: control-h-md }).pad({ all: 0, x: 0 }).type({ size: base })}
  style sizeIconSm = {size({ w: control-h-sm, h: control-h-sm }).pad({ all: 0, x: 0 }).type({ size: sm })}
  style sizeIconLg = {size({ w: control-h-lg, h: control-h-lg }).pad({ all: 0, x: 0 }).type({ size: md })}

  // Untoned invert is not a product default (tone defaults to primary).
  // These still exist so an explicit empty tone does not go naked.
  style vSolid = {bg(foreground).border({ color: foreground }).fg(background).elevation(xs), when: {hover: filter({ opacity: 90 }).elevation(sm)}}
  style vSolidTone = {bg(tone).border({ color: tone }).fg(tone.foreground).elevation(xs), when: {hover: filter({ opacity: 90 }).elevation(sm)}}
  style vSoft = {bg(hover-bg).border({ color: border }).fg(foreground), when: {hover: filter({ opacity: 90 })}}
  style vSoftTone = {bg({ color: tone, mix: 15 }).border({ color: tone }).fg(tone), when: {hover: bg({ color: tone, mix: 25 })}}
  style vOutline = {bg(transparent).border({ color: foreground }).fg(foreground), when: {hover: bg(hover-bg), disabled: bg(transparent)}}
  style vOutlineTone = {bg(transparent).border({ color: tone }).fg(tone), when: {hover: bg(hover-bg), disabled: bg(transparent)}}
  style vGhost = {bg(transparent).border({ color: transparent }).fg(foreground), when: {hover: bg(hover-bg), disabled: bg(transparent)}}
  style vGhostTone = {bg(transparent).border({ color: transparent }).fg(tone), when: {hover: bg(hover-bg), disabled: bg(transparent)}}
</scope>

<button
  style(base, {match: { disabled: { true: cursorOff, false: cursorOn }, variant: { default: style(vSolid, vSolidTone), solid: style(vSolid, vSolidTone), soft: style(vSoft, vSoftTone), outline: style(vOutline, vOutlineTone), ghost: style(vGhost, vGhostTone) }, size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg, icon: sizeIcon, iconSm: sizeIconSm, iconLg: sizeIconLg }, pill: { true: pill } }, compound: [
      {loading: true, cursor(wait).pointerEvents(none)},
      {tone: muted, variant: default, bg(muted-foreground).border({ color: muted-foreground }).fg(background)},
      {tone: muted, variant: solid, bg(muted-foreground).border({ color: muted-foreground }).fg(background)},
      {tone: muted, variant: soft, bg(muted).border({ color: border-strong }).fg(foreground)},
      {tone: muted, variant: outline, border({ color: muted-foreground }).fg(muted-foreground)},
      {tone: muted, variant: ghost, fg(muted-foreground)},
    ]})
>
  <row cluster gap="2">
    <slot></slot>
  </row>
</button>