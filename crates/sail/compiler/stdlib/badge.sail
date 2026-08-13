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

  style base = {box: flow({ display: inline-flex }).size({ min-h: control-h-2xs }).space({ pad-x: 2, pad-y: px }), paint: border({ width: 1 }).radius(sm), text: type({ family: body, weight: semibold, leading: none, wrap: nowrap, size: xs })}

  style soft = {paint: fill({ color: muted }).ink({ color: muted-foreground }).border({ color: border })}
  style solid = {paint: fill({ color: muted }).ink({ color: muted-foreground }).border({ color: border }).elevation(xs)}
  style outline = {paint: fill({ color: transparent }).ink({ color: muted-foreground }).border({ color: border })}

  style sizeMd = {box: size({ min-h: control-h-2xs }).space({ pad-x: 2 }), text: type({ size: xs })}
  style sizeSm = {box: size({ min-h: 4 }).space({ pad-x: 1_5 }), text: type({ size: xs })}
  style sizeLg = {box: size({ min-h: control-h-xs }).space({ pad-x: 2_5 }), text: type({ size: xs })}

  style solidTone = {paint: fill({ color: tone }).ink({ color: tone.foreground }).border({ color: tone }).elevation(xs)}
  style softTone = {paint: fill({ color: tone, mix: 15 }).border({ color: tone }).ink({ color: tone })}
  style outlineTone = {paint: fill({ color: transparent }).ink({ color: tone }).border({ color: tone })}
</scope>

<span
  style(base, {when: {hover: {paint: filter({ opacity: 90 })}, focus: {paint: border({ width: 2, kind: outline })}}, match: {variant: {
        soft: style(soft, softTone),
        solid: style(solid, solidTone),
        outline: style(outline, outlineTone),
      }, size: {
        md: sizeMd,
        sm: sizeSm,
        lg: sizeLg,
      }, pill: {true: {paint: radius(full)}, false: {paint: radius(sm)}}}, compound: [
      {tone: muted, variant: solid, paint: fill(muted).ink({ color: muted-foreground }).border({ color: border })},
      {tone: muted, variant: soft, paint: fill(muted).ink({ color: muted-foreground }).border({ color: border })},
      {tone: muted, variant: outline, paint: ink({ color: muted-foreground }).border({ color: border })},
      {pill: true, box: space({ pad-x: 2_5 }), paint: radius(full)},
    ]})
>
  <slot></slot>
</span>
