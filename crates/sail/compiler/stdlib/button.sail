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
  }

  // border(solid) kills UA button outset so freeform demos use design-system chrome.
  // No base bg(transparent) or border(transparent): unlayered product atomics
  // (.s-bgtransparent / .s-bordertransparent) beat layered variant fills and
  // outline strokes. Variants own fill + border color; ghost is the transparent pair.
  // Cursor is NOT on base: base cursor(pointer) + when-disabled cursor(not-allowed) both
  // stamp classes and pointer was winning in the cascade. Match on the disabled prop so
  // only one cursor utility is active.
  style base = {box: flow({ display: inline-flex }).place({ position: relative }), paint: appearance(none).radius(md).border({ width: control-stroke, style: solid }).isolate().controlFrame().select(none), text: type({ weight: medium, leading: none, wrap: nowrap }), when: {hover: {paint: controlLift()}, active: {paint: press()}}}
  style cursorOn = {paint: cursor(pointer)}
  style cursorOff = {paint: cursor(not-allowed).filter({ opacity: 50 })}

  style sizeXs = {box: size({ min-h: control-h-2xs }).space({ pad-x: 2 }), text: type({ size: xs })}
  style sizeSm = {box: size({ min-h: control-h-sm }).space({ pad-x: 2_5 }), text: type({ size: sm })}
  style sizeMd = {box: size({ min-h: control-h-md }).space({ pad-x: 3 }), text: type({ size: base })}
  style sizeLg = {box: size({ min-h: control-h-lg }).space({ pad-x: 4 }), text: type({ size: md })}
  // Square icon hit targets — explicit w/h (not minW: parent .sail-col zeros min-width).
  style sizeIcon = {box: size({ w: control-h-md, h: control-h-md }).space({ pad: 0, pad-x: 0 }), text: type({ size: base })}
  style sizeIconSm = {box: size({ w: control-h-sm, h: control-h-sm }).space({ pad: 0, pad-x: 0 }), text: type({ size: sm })}
  style sizeIconLg = {box: size({ w: control-h-lg, h: control-h-lg }).space({ pad: 0, pad-x: 0 }), text: type({ size: md })}

  // Untoned invert is not a product default (tone defaults to primary).
  // These still exist so an explicit empty tone does not go naked.
  style vSolid = {paint: fill({ color: foreground }).border({ color: foreground }).ink({ color: background }).elevation(xs), when: {hover: {paint: filter({ opacity: 90 }).elevation(sm)}}}
  style vSolidTone = {paint: fill({ color: tone }).border({ color: tone }).ink({ color: tone.foreground }).elevation(xs), when: {hover: {paint: filter({ opacity: 90 }).elevation(sm)}}}
  style vSoft = {paint: fill({ color: hover-bg }).border({ color: border }).ink(button), when: {hover: {paint: filter({ opacity: 90 })}}}
  style vSoftTone = {paint: fill({ color: tone, mix: 15 }).border({ color: tone }).ink({ color: tone }), when: {hover: {paint: fill({ color: tone, mix: 25 })}}}
  style vOutline = {paint: fill({ color: transparent }).border({ color: foreground }).ink(button), when: {hover: {paint: fill({ color: hover-bg })}, disabled: {paint: fill({ color: transparent })}}}
  style vOutlineTone = {paint: fill({ color: transparent }).border({ color: tone }).ink({ color: tone }), when: {hover: {paint: fill({ color: hover-bg })}, disabled: {paint: fill({ color: transparent })}}}
  style vGhost = {paint: fill({ color: transparent }).border({ color: transparent }).ink(button), when: {hover: {paint: fill({ color: hover-bg })}, disabled: {paint: fill({ color: transparent })}}}
  style vGhostTone = {paint: fill({ color: transparent }).border({ color: transparent }).ink({ color: tone }), when: {hover: {paint: fill({ color: hover-bg })}, disabled: {paint: fill({ color: transparent })}}}
</scope>

<button
  style(base, {match: {
      disabled: {
        true: cursorOff,
        false: cursorOn,
      },
      variant: {
        // default ≡ solid — one fill model, tone always applies when set.
        default: style(vSolid, vSolidTone),
        solid: style(vSolid, vSolidTone),
        soft: style(vSoft, vSoftTone),
        outline: style(vOutline, vOutlineTone),
        ghost: style(vGhost, vGhostTone),
      },
      size: {
        xs: sizeXs,
        sm: sizeSm,
        md: sizeMd,
        lg: sizeLg,
        icon: sizeIcon,
        iconSm: sizeIconSm,
        iconLg: sizeIconLg,
      },
    }, compound: [
      {tone: muted, variant: default, paint: fill({ color: muted-foreground }).border({ color: muted-foreground }).ink({ color: background })},
      {tone: muted, variant: solid, paint: fill({ color: muted-foreground }).border({ color: muted-foreground }).ink({ color: background })},
      {tone: muted, variant: soft, paint: fill(muted).border({ color: border-strong }).ink(foreground)},
      {tone: muted, variant: outline, paint: border({ color: muted-foreground }).ink({ color: muted-foreground })},
      {tone: muted, variant: ghost, paint: ink({ color: muted-foreground })},
    ]})
>
  <slot></slot>
</button>