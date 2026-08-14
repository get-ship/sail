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
  style base = {display(inline-flex).size({ min-h: control-h-md }).pad({ x: 3 }).appearance(none).radius(md).border({ color: transparent, width: control-stroke, style: solid }).select(none).cursor(pointer).type({ weight: medium, leading: none, size: base, wrap: nowrap, decoration: none })}
  // default rest = themeable button surface (not solid invert).
  style vDefault = {bg(button).border({ color: border }).fg(button).elevation(xs), when: {hover: bg(hover-bg).border({ color: border-strong })}}
  style vDefaultTone = {bg({ color: tone, mix: 15 }).border({ color: tone }).fg(tone), when: {hover: bg({ color: tone, mix: 25 })}}
  style vSolid = {bg(foreground).border({ color: foreground }).fg(background).elevation(xs), when: {hover: filter({ opacity: 90 }).elevation(sm)}}
  style vSolidTone = {bg(tone).border({ color: tone }).fg(tone.foreground).elevation(xs), when: {hover: filter({ opacity: 90 }).elevation(sm)}}
  style vSoft = {bg(hover-bg).border({ color: border }).fg(button), when: {hover: filter({ opacity: 90 })}}
  style vSoftTone = {bg({ color: tone, mix: 15 }).border({ color: tone }).fg(tone), when: {hover: bg({ color: tone, mix: 25 })}}
  style vOutline = {bg(transparent).border({ color: foreground }).fg(button), when: {hover: bg(hover-bg), disabled: bg(transparent)}}
  style vOutlineTone = {bg(transparent).border({ color: tone }).fg(tone), when: {hover: bg(hover-bg), disabled: bg(transparent)}}
  style vGhost = {bg(transparent).border({ color: transparent }).fg(button), when: {hover: bg(hover-bg), disabled: bg(transparent)}}
  style vGhostTone = {bg(transparent).fg(tone), when: {hover: bg(hover-bg), disabled: bg(transparent)}}
</scope>

<a
  style(base, {when: {disabled: filter({ opacity: 50 }).cursor(not-allowed)}, match: { variant: { default: style(vDefault, vDefaultTone), solid: style(vSolid, vSolidTone), soft: style(vSoft, vSoftTone), outline: style(vOutline, vOutlineTone), ghost: style(vGhost, vGhostTone) } }, compound: [
      {tone: muted, variant: default, bg(muted).border({ color: border }).fg(muted-foreground)},
      {tone: muted, variant: solid, bg(muted-foreground).border({ color: muted-foreground }).fg(background)},
      {tone: muted, variant: soft, bg(muted).border({ color: border }).fg(muted-foreground)},
      {tone: muted, variant: outline, border({ color: muted-foreground }).fg(muted-foreground)},
      {tone: muted, variant: ghost, fg(muted-foreground)},
    ]})
>
  <slot></slot>
</a>
