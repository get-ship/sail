<!--
  <chip> — filter/selection pill.
  Rest paint = muted when tone is omitted.
  Semantic paint = bound tone group paths.

  Host is a text-bearing <span> (text channel + Style match). Size/variant/
  selected attrs and paint classes must share one node or match CSS never
  fires. Layout: inline-flex + gap (no outer row wrapper).
-->

<scope>
  props: {
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    variant: 'solid' | 'soft' | 'outline' | 'ghost' = 'solid'
    size: 'md' | 'sm' | 'lg' = 'md'
    selected = false
    removable = false
    disabled = false
    onRemove: () => void
  }

  // Default solid rides muted. Brand tones only when tone= is set.
  style base = {box: flow({ display: inline-flex }).size({ min-h: control-h-2xs }).space({ pad-x: 2, pad-y: px }), paint: group().appearance(none).cursor(pointer).select(none).tapHighlight(none).radius(full).border({ color: border, width: 1, style: solid }).fill({ color: muted }).ink({ color: muted-foreground }).motion({ property: colors, duration: fast, ease: out }), text: type({ family: body, weight: medium, leading: none, wrap: nowrap, size: xs })}
  style sizeMd = {box: size({ min-h: control-h-2xs }).space({ pad-x: 2_5 }), text: type({ size: sm })}
  style sizeSm = {box: size({ min-h: control-h-2xs }).space({ pad-x: 2 }), text: type({ size: xs })}
  style sizeLg = {box: size({ min-h: control-h-xs }).space({ pad-x: 3 }), text: type({ size: sm })}
  style remove = {box: flow({ display: none }).size(icon-size-xs).space({ pad: 0 }), paint: border({ width: 0 }).fill({ color: transparent }).ink({ color: currentColor }).radius(full).motion({ property: colors, duration: fast, ease: out }).cursor(pointer)}
  style leading = {box: flow({ display: contents })}
  style xmark = {box: flow({ display: block }).size(full)}
</scope>

<span
  use:pressable
  use:pressed
  use:dismissible
  style(base, {when: {pressed: {paint: fill({ color: primary }).ink({ color: primary-foreground }).border({ color: primary }).elevation(xs), text: type({ weight: semibold })}, disabled: {paint: cursor(not-allowed).fill({ color: muted }).border({ color: border-subtle }).ink({ color: muted-foreground }).filter({ saturate: 65 }).elevation(none)}, focus: {paint: border({ style: solid, kind: outline, width: 2, color: ring, offset: 2 })}}, match: {size: {
        md: sizeMd,
        sm: sizeSm,
        lg: sizeLg,
      }, variant: {// Rest solid (no tone) keeps muted. Tone+solid is compound.
        solid: {paint: fill({ color: muted }).ink({ color: muted-foreground }).border({ color: border }), when: {hover: {paint: filter({ opacity: 90 })}}}, // Soft without tone: muted. Soft+tone: wash compounds below.
        soft: {paint: fill({ color: muted }).ink({ color: muted-foreground }).border({ color: border }), when: {hover: {paint: filter({ opacity: 90 })}}}, outline: {paint: fill({ color: transparent }).ink({ color: muted-foreground }).border({ color: border }), when: {hover: {paint: fill({ color: hover-bg })}}}, ghost: {paint: fill({ color: transparent }).border({ color: transparent }).ink({ color: muted-foreground }), when: {hover: {paint: fill({ color: hover-bg })}}}}}, compound: [
      // Tone + solid
      {tone: primary, variant: solid, paint: fill({ color: primary }).ink({ color: primary-foreground }).border({ color: primary })},
      {tone: info, variant: solid, paint: fill({ color: info }).ink({ color: info-foreground }).border({ color: info })},
      {tone: success, variant: solid, paint: fill({ color: success }).ink({ color: success-foreground }).border({ color: success })},
      {tone: warning, variant: solid, paint: fill({ color: warning }).ink({ color: warning-foreground }).border({ color: warning })},
      {tone: destructive, variant: solid, paint: fill({ color: destructive }).ink({ color: destructive-foreground }).border({ color: destructive })},
      {tone: accent, variant: solid, paint: fill({ color: accent }).ink({ color: accent-foreground }).border({ color: accent })},
      {tone: muted, variant: solid, paint: fill(muted).ink({ color: muted-foreground }).border({ color: border })},
      // Tone + soft (wash)
      {tone: primary, variant: soft, paint: fill({ color: primary, mix: 15 }).border({ color: primary }).ink({ color: primary }), when: {hover: {paint: fill({ color: primary, mix: 25 })}}},
      {tone: info, variant: soft, paint: fill({ color: info, mix: 15 }).border({ color: info }).ink({ color: info }), when: {hover: {paint: fill({ color: info, mix: 25 })}}},
      {tone: success, variant: soft, paint: fill({ color: success, mix: 15 }).border({ color: success }).ink({ color: success }), when: {hover: {paint: fill({ color: success, mix: 25 })}}},
      {tone: warning, variant: soft, paint: fill({ color: warning, mix: 15 }).border({ color: warning }).ink({ color: warning }), when: {hover: {paint: fill({ color: warning, mix: 25 })}}},
      {tone: destructive, variant: soft, paint: fill({ color: destructive, mix: 15 }).border({ color: destructive }).ink({ color: destructive }), when: {hover: {paint: fill({ color: destructive, mix: 25 })}}},
      {tone: accent, variant: soft, paint: fill({ color: accent, mix: 15 }).border({ color: accent }).ink({ color: accent }), when: {hover: {paint: fill({ color: accent, mix: 25 })}}},
      {tone: muted, variant: soft, paint: fill({ color: muted, mix: 15 }).border({ color: border }).ink({ color: muted-foreground }), when: {hover: {paint: fill({ color: muted, mix: 25 })}}},
      // Tone + outline / ghost
      {tone: primary, variant: outline, paint: fill({ color: transparent }).ink({ color: primary }).border({ color: primary })},
      {tone: info, variant: outline, paint: fill({ color: transparent }).ink({ color: info }).border({ color: info })},
      {tone: success, variant: outline, paint: fill({ color: transparent }).ink({ color: success }).border({ color: success })},
      {tone: warning, variant: outline, paint: fill({ color: transparent }).ink({ color: warning }).border({ color: warning })},
      {tone: destructive, variant: outline, paint: fill({ color: transparent }).ink({ color: destructive }).border({ color: destructive })},
      {tone: accent, variant: outline, paint: fill({ color: transparent }).ink({ color: accent }).border({ color: accent })},
      {tone: muted, variant: outline, paint: ink({ color: muted-foreground }).border({ color: border })},
      {tone: primary, variant: ghost, paint: fill({ color: transparent }).border({ color: transparent }).ink({ color: primary })},
      {tone: info, variant: ghost, paint: fill({ color: transparent }).border({ color: transparent }).ink({ color: info })},
      {tone: success, variant: ghost, paint: fill({ color: transparent }).border({ color: transparent }).ink({ color: success })},
      {tone: warning, variant: ghost, paint: fill({ color: transparent }).border({ color: transparent }).ink({ color: warning })},
      {tone: destructive, variant: ghost, paint: fill({ color: transparent }).border({ color: transparent }).ink({ color: destructive })},
      {tone: accent, variant: ghost, paint: fill({ color: transparent }).border({ color: transparent }).ink({ color: accent })},
      {tone: muted, variant: ghost, paint: ink({ color: muted-foreground })},
      // Selected beats tone×variant (same/higher attr count so it can win cascade).
      {selected: true, paint: fill({ color: primary }).ink({ color: primary-foreground }).border({ color: primary }).elevation(xs)},
      {selected: true, variant: solid, paint: fill({ color: primary }).ink({ color: primary-foreground }).border({ color: primary }).elevation(xs)},
      {selected: true, variant: soft, paint: fill({ color: primary }).ink({ color: primary-foreground }).border({ color: primary }).elevation(xs)},
      {selected: true, variant: outline, paint: fill({ color: primary }).ink({ color: primary-foreground }).border({ color: primary }).elevation(xs)},
      {selected: true, variant: ghost, paint: fill({ color: primary }).ink({ color: primary-foreground }).border({ color: primary }).elevation(xs)},
      {selected: true, when: {hover: {paint: filter({ opacity: 90 })}}},
    ]})
>
  <span style(leading)>
    <slot name="leading"></slot>
  </span>
  <slot></slot>
  <button
    type="button"
    anatomy
    data-dismiss
    label="Remove"
    tabindex="-1"
    style(remove, {box: flow({ shrink: 0 }), match: {removable: {true: {box: flow({ display: inline-flex })}, false: {box: flow({ display: none })}}}, when: {hover: {paint: fill({ color: muted }).ink({ color: muted-foreground })}, focus: {paint: border({ style: solid, kind: outline, width: 2, color: ring, offset: 2 })}}})
  >
    <svg viewBox="0 0 10 10" stroke-width="1.75" stroke-linecap="round" stroke="currentColor" fill="none" style(xmark)>
      <path d="M2 2 L8 8 M8 2 L2 8"></path>
    </svg>
  </button>
</span>
