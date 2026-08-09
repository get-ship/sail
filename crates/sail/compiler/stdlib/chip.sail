<!--
  <chip> — filter/selection pill.
  Rest paint = component.chip tokens when tone is omitted.
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

  // Default solid rides component.chip tokens. Brand tones only when tone= is set.
  style base = {
    box: display(inline-flex).minH(control-h-2xs).padX(2).padY(px).gap(1),
    paint: group().appearance(none).cursor(pointer).select(none).tapHighlight(none).radius(full).borderWidth(1).border(solid).bg(chip).fg(chip.foreground).border(chip.border).transition(colors).duration(fast).ease(out),
    text: fontFamily(body).fontWeight(medium).leading(none).whitespace(nowrap).fontSize(xs),
  }
  style sizeMd = { box: minH(control-h-2xs).padX(2_5), text: fontSize(sm) }
  style sizeSm = { box: minH(control-h-2xs).padX(2), text: fontSize(xs) }
  style sizeLg = { box: minH(control-h-xs).padX(3), text: fontSize(sm) }
  style remove = { box: display(none).size(icon-size-xs).pad(0), paint: borderWidth(0).bg(transparent).fg(currentColor).cursor(pointer).radius(full).transition(colors).duration(fast).ease(out) }
  style leading = { box: display(contents) }
  style xmark = { box: display(block).size(full), paint: stroke(currentColor).fill(none) }
</scope>

<span
  use:pressable
  use:pressed
  use:dismissible
  style(base, {
    when: {
      pressed: {
        paint: bg(primary).fg(primary-foreground).border(primary).shadow(xs),
        text: fontWeight(semibold),
      },
      disabled: {
        paint: cursor(not-allowed).bg(chip).border(border-subtle).fg(chip.foreground).saturate(65).shadow(none),
      },
      focus: {
        paint: outline(solid).outline(2).outline(ring).outlineOffset(2),
      },
    },
    match: {
      size: {
        md: sizeMd,
        sm: sizeSm,
        lg: sizeLg,
      },
      variant: {
        // Rest solid (no tone) keeps base chip tokens. Tone+solid is compound.
        solid: {
          paint: bg(chip).fg(chip.foreground).border(chip.border),
          when: { hover: { paint: opacity(90) } },
        },
        // Soft without tone: muted wash. Soft+tone: colorMix via --tone.
        soft: {
          paint: colorMix(muted, 15).border(border).fg(muted-foreground),
          when: { hover: { paint: colorMix(muted, 25) } },
        },
        outline: {
          paint: bg(transparent).fg(chip.foreground).border(chip.border),
          when: { hover: { paint: bg(hover-bg) } },
        },
        ghost: {
          paint: bg(transparent).border(transparent).fg(chip.foreground),
          when: { hover: { paint: bg(hover-bg) } },
        },
      },
    },
    compound: [
      // Tone + solid
      { tone: primary, variant: solid, paint: bg(primary).fg(primary-foreground).border(primary) },
      { tone: info, variant: solid, paint: bg(info).fg(info-foreground).border(info) },
      { tone: success, variant: solid, paint: bg(success).fg(success-foreground).border(success) },
      { tone: warning, variant: solid, paint: bg(warning).fg(warning-foreground).border(warning) },
      { tone: destructive, variant: solid, paint: bg(destructive).fg(destructive-foreground).border(destructive) },
      { tone: accent, variant: solid, paint: bg(accent).fg(accent-foreground).border(accent) },
      { tone: muted, variant: solid, paint: bg(muted).fg(muted-foreground).border(border) },
      // Tone + soft (wash)
      { tone: primary, variant: soft, paint: colorMix(primary, 15).border(primary).fg(primary), when: { hover: { paint: colorMix(primary, 25) } } },
      { tone: info, variant: soft, paint: colorMix(info, 15).border(info).fg(info), when: { hover: { paint: colorMix(info, 25) } } },
      { tone: success, variant: soft, paint: colorMix(success, 15).border(success).fg(success), when: { hover: { paint: colorMix(success, 25) } } },
      { tone: warning, variant: soft, paint: colorMix(warning, 15).border(warning).fg(warning), when: { hover: { paint: colorMix(warning, 25) } } },
      { tone: destructive, variant: soft, paint: colorMix(destructive, 15).border(destructive).fg(destructive), when: { hover: { paint: colorMix(destructive, 25) } } },
      { tone: accent, variant: soft, paint: colorMix(accent, 15).border(accent).fg(accent), when: { hover: { paint: colorMix(accent, 25) } } },
      { tone: muted, variant: soft, paint: colorMix(muted, 15).border(border).fg(muted-foreground), when: { hover: { paint: colorMix(muted, 25) } } },
      // Tone + outline / ghost
      { tone: primary, variant: outline, paint: bg(transparent).fg(primary).border(primary) },
      { tone: info, variant: outline, paint: bg(transparent).fg(info).border(info) },
      { tone: success, variant: outline, paint: bg(transparent).fg(success).border(success) },
      { tone: warning, variant: outline, paint: bg(transparent).fg(warning).border(warning) },
      { tone: destructive, variant: outline, paint: bg(transparent).fg(destructive).border(destructive) },
      { tone: accent, variant: outline, paint: bg(transparent).fg(accent).border(accent) },
      { tone: muted, variant: outline, paint: fg(muted-foreground).border(border) },
      { tone: primary, variant: ghost, paint: bg(transparent).border(transparent).fg(primary) },
      { tone: info, variant: ghost, paint: bg(transparent).border(transparent).fg(info) },
      { tone: success, variant: ghost, paint: bg(transparent).border(transparent).fg(success) },
      { tone: warning, variant: ghost, paint: bg(transparent).border(transparent).fg(warning) },
      { tone: destructive, variant: ghost, paint: bg(transparent).border(transparent).fg(destructive) },
      { tone: accent, variant: ghost, paint: bg(transparent).border(transparent).fg(accent) },
      { tone: muted, variant: ghost, paint: fg(muted-foreground) },
      { selected: true, paint: bg(primary).fg(primary-foreground).border(primary).shadow(xs) },
      { selected: true, tone: muted, paint: bg(primary).fg(primary-foreground).border(primary) },
      { selected: true, when: { hover: { paint: opacity(90) } } },
    ],
  })
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
    style(remove, {
      box: shrink(0),
      when: {
        group-removable: { box: display(inline-flex) },
        hover: { paint: bg(chip).fg(chip.foreground) },
        focus: { paint: outline(solid).outline(2).outline(ring).outlineOffset(2) },
      },
    })
  >
    <svg viewBox="0 0 10 10" hidden="true" stroke-width="1.75" stroke-linecap="round" style(xmark)>
      <path d="M2 2 L8 8 M8 2 L2 8"></path>
    </svg>
  </button>
</span>
