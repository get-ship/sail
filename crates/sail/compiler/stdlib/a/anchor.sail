<!--
  <a> — native <a> (docs name: anchor). Dual mode:
    • bare / variant=text = no chrome. Wrap a card, span, or label as a link.
      Freeform skips stamping variant/tone so every href stays inline.
    • variant default|solid|soft|outline|ghost = button-link chrome.
  default ≠ solid: rest uses background / foreground / border.
  solid rest = inverted free; solid + tone = filled tone.
  soft / outline / ghost rest → button.* ; tone always paints when set.
  No size axis (use button for sized CTAs). disabled paint needs
  aria-disabled on anchors (native disabled is not a real <a> state).
-->

<scope>
  props: {
    // Type surface defaults; freeform stamping skips variant/tone (bare link).
    variant: 'text' | 'default' | 'solid' | 'soft' | 'outline' | 'ghost' = 'text'
    // Exception: do not default tone — freeform <a href> must stay a text link.
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    disabled = false
  }

  // text / bare = no chrome — wrap a card, span, or label as a link.
  // Button-link chrome only when variant is a button family value.
  style base = {display(inline).appearance(none).cursor(pointer).type({ family: body, weight: medium, size: base })}
  style linkText = {display(inline).size({ min-h: 0 }).pad(0).bg(transparent).elevation(none).fg(primary).type({ decoration: underline }), when: {hover: fg(primary).filter({ opacity: 80 })}}
  style linkTextTone = {fg(tone), when: {hover: fg(tone).filter({ opacity: 80 })}}
  // default rest = themeable button surface (not solid invert).
  style vDefault = {display(inline-flex).size({ min-h: control-h-md }).pad({ x: 3 }).bg(background).border({ color: border, width: control-stroke, style: solid }).radius(md).fg(foreground).elevation(xs).type({ leading: none, wrap: nowrap, decoration: none }), when: {hover: bg(hover-bg).border({ color: border-strong })}}
  style vDefaultTone = {bg({ color: tone, mix: 15 }).border({ color: tone }).fg(tone), when: {hover: bg({ color: tone, mix: 25 })}}
  style vSolid = {display(inline-flex).size({ min-h: control-h-md }).pad({ x: 3 }).radius(md).select(none).bg(foreground).border({ color: foreground, width: control-stroke, style: solid }).fg(background).elevation(xs).type({ leading: none, wrap: nowrap, decoration: none }), when: {hover: filter({ opacity: 90 }).elevation(sm)}}
  style vSolidTone = {bg(tone).border({ color: tone }).fg(tone.foreground).elevation(xs), when: {hover: filter({ opacity: 90 }).elevation(sm)}}
  style vSoft = {display(inline-flex).size({ min-h: control-h-md }).pad({ x: 3 }).radius(md).select(none).bg(hover-bg).border({ color: border, width: control-stroke, style: solid }).fg(foreground).type({ leading: none, wrap: nowrap, decoration: none }), when: {hover: filter({ opacity: 90 })}}
  style vSoftTone = {bg({ color: tone, mix: 15 }).border({ color: tone }).fg(tone), when: {hover: bg({ color: tone, mix: 25 })}}
  style vOutline = {display(inline-flex).size({ min-h: control-h-md }).pad({ x: 3 }).radius(md).select(none).bg(transparent).border({ color: foreground, width: control-stroke, style: solid }).fg(foreground).type({ leading: none, wrap: nowrap, decoration: none }), when: {hover: bg(hover-bg), disabled: bg(transparent)}}
  style vOutlineTone = {bg(transparent).border({ color: tone }).fg(tone), when: {hover: bg(hover-bg), disabled: bg(transparent)}}
  style vGhost = {display(inline-flex).size({ min-h: control-h-md }).pad({ x: 3 }).radius(md).select(none).bg(transparent).border({ color: transparent, width: control-stroke, style: solid }).fg(foreground).type({ leading: none, wrap: nowrap, decoration: none }), when: {hover: bg(hover-bg), disabled: bg(transparent)}}
  style vGhostTone = {bg(transparent).fg(tone), when: {hover: bg(hover-bg), disabled: bg(transparent)}}
</scope>

<a
  style(base, {when: {disabled: filter({ opacity: 50 }).cursor(not-allowed)}, match: { variant: { text: style(linkText, linkTextTone), default: style(vDefault, vDefaultTone), solid: style(vSolid, vSolidTone), soft: style(vSoft, vSoftTone), outline: style(vOutline, vOutlineTone), ghost: style(vGhost, vGhostTone) } }, compound: [
      {tone: muted, variant: text, fg(muted-foreground)},
      {tone: muted, variant: default, bg(muted).border({ color: border }).fg(muted-foreground)},
      {tone: muted, variant: solid, bg(muted-foreground).border({ color: muted-foreground }).fg(background)},
      {tone: muted, variant: soft, bg(muted).border({ color: border }).fg(muted-foreground)},
      {tone: muted, variant: outline, border({ color: muted-foreground }).fg(muted-foreground)},
      {tone: muted, variant: ghost, fg(muted-foreground)},
    ]})
>
  <slot></slot>
</a>
