<!--
  <a> — native <a>. Dual mode:
    • bare (no variant attr) = text link (sail-base a.sail-a; freeform skips
      stamping variant/tone so every href stays inline).
    • variant set = button-link chrome (inline-flex control).
  default ≠ solid: rest uses background / foreground / border.
  solid rest = inverted free; solid + tone = filled tone.
  soft / outline / ghost rest → button.* ; tone always paints when set.
  No size axis (use button for sized CTAs). disabled paint needs
  aria-disabled on anchors (native disabled is not a real <a> state).
-->

<scope>
  props: {
    // Type surface defaults; freeform stamping skips variant/tone (bare link).
    variant: 'default' | 'solid' | 'soft' | 'outline' | 'ghost' = 'default'
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    disabled = false
  }

  // Button-link chrome only when variant is present (attr-gated match arms).
  // display(inline-flex) so freeform never leaves the label top-aligned in a block box.
  style base = {box: flow({ display: inline-flex }).size({ min-h: control-h-md }).space({ pad-x: 3 }), paint: appearance(none).radius(md).border({ color: transparent, width: control-stroke, style: solid }).select(none).cursor(pointer), text: type({ weight: medium, leading: none, size: base, wrap: nowrap, decoration: none })}
  // default rest = themeable button surface (not solid invert).
  style vDefault = {paint: fill(button).border({ color: border }).ink(button).elevation(xs), when: {hover: {paint: fill({ color: hover-bg }).border({ color: border-strong })}}}
  style vDefaultTone = {paint: fill({ color: tone, mix: 15 }).border({ color: tone }).ink({ color: tone }), when: {hover: {paint: fill({ color: tone, mix: 25 })}}}
  style vSolid = {paint: fill({ color: foreground }).border({ color: foreground }).ink({ color: background }).elevation(xs), when: {hover: {paint: filter({ opacity: 90 }).elevation(sm)}}}
  style vSolidTone = {paint: fill({ color: tone }).border({ color: tone }).ink({ color: tone.foreground }).elevation(xs), when: {hover: {paint: filter({ opacity: 90 }).elevation(sm)}}}
  style vSoft = {paint: fill({ color: hover-bg }).border({ color: border }).ink(button), when: {hover: {paint: filter({ opacity: 90 })}}}
  style vSoftTone = {paint: fill({ color: tone, mix: 15 }).border({ color: tone }).ink({ color: tone }), when: {hover: {paint: fill({ color: tone, mix: 25 })}}}
  style vOutline = {paint: fill({ color: transparent }).border({ color: foreground }).ink(button), when: {hover: {paint: fill({ color: hover-bg })}, disabled: {paint: fill({ color: transparent })}}}
  style vOutlineTone = {paint: fill({ color: transparent }).border({ color: tone }).ink({ color: tone }), when: {hover: {paint: fill({ color: hover-bg })}, disabled: {paint: fill({ color: transparent })}}}
  style vGhost = {paint: fill({ color: transparent }).border({ color: transparent }).ink(button), when: {hover: {paint: fill({ color: hover-bg })}, disabled: {paint: fill({ color: transparent })}}}
  style vGhostTone = {paint: fill({ color: transparent }).ink({ color: tone }), when: {hover: {paint: fill({ color: hover-bg })}, disabled: {paint: fill({ color: transparent })}}}
</scope>

<a
  style(base, {when: {disabled: {paint: filter({ opacity: 50 }).cursor(not-allowed)}}, match: {
      variant: {
        // default rest = button.*; + tone = soft-wash brand (not solid invert).
        default: style(vDefault, vDefaultTone),
        solid: style(vSolid, vSolidTone),
        soft: style(vSoft, vSoftTone),
        outline: style(vOutline, vOutlineTone),
        ghost: style(vGhost, vGhostTone),
      },
    }, compound: [
      {tone: muted, variant: default, paint: fill(muted).border({ color: border }).ink({ color: muted-foreground })},
      {tone: muted, variant: solid, paint: fill({ color: muted-foreground }).border({ color: muted-foreground }).ink({ color: background })},
      {tone: muted, variant: soft, paint: fill(muted).border({ color: border }).ink({ color: muted-foreground })},
      {tone: muted, variant: outline, paint: border({ color: muted-foreground }).ink({ color: muted-foreground })},
      {tone: muted, variant: ghost, paint: ink({ color: muted-foreground })},
    ]})
>
  <slot></slot>
</a>
