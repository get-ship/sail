<!--
  <badge> — L1 language element. Style value language only.
  Rest paint = muted when tone is omitted.
  Semantic paint = bound tone.
-->

<scope>
  props: {
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    variant: 'soft' | 'solid' | 'outline' = 'soft'
    size: 'md' | 'sm' | 'lg' = 'md'
    pill = false
  }

  style base = {display(inline-flex).size({ min-h: control-h-2xs }).pad({ x: 2, y: px }).border({ width: 1 }).radius(sm).type({ family: body, weight: semibold, leading: none, wrap: nowrap, size: xs })}

  style soft = {bg(muted).fg(muted-foreground).border({ color: border })}
  style solid = {bg(muted).fg(muted-foreground).border({ color: border }).elevation(xs)}
  style outline = {bg(transparent).fg(muted-foreground).border({ color: border })}

  style sizeMd = {size({ min-h: control-h-2xs }).pad({ x: 2 }).type({ size: xs })}
  style sizeSm = {size({ min-h: 4 }).pad({ x: 1_5 }).type({ size: xs })}
  style sizeLg = {size({ min-h: control-h-xs }).pad({ x: 2_5 }).type({ size: xs })}

  style solidTone = {bg(tone).fg(tone.foreground).border({ color: tone }).elevation(xs)}
  style softTone = {bg({ color: tone, mix: 15 }).border({ color: tone }).fg(tone)}
  style outlineTone = {bg(transparent).fg(tone).border({ color: tone })}
</scope>

<span
  style(base, {when: {hover: filter({ opacity: 90 }), focus: border({ width: 2, kind: outline })}, match: {variant: { soft: style(soft, softTone), solid: style(solid, solidTone), outline: style(outline, outlineTone) }, size: { md: sizeMd, sm: sizeSm, lg: sizeLg }, pill: {true: radius(full), false: radius(sm)}}, compound: [
      {tone: muted, variant: solid, bg(muted).fg(muted-foreground).border({ color: border })},
      {tone: muted, variant: soft, bg(muted).fg(muted-foreground).border({ color: border })},
      {tone: muted, variant: outline, fg(muted-foreground).border({ color: border })},
      {pill: true, pad({ x: 2_5 }).radius(full)},
    ]})
>
  <slot></slot>
</span>
