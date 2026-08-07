<!--
  <chip> — filter/selection pill.

  Rest paint = component.chip tokens when tone is omitted.
  Semantic paint = bound tone group paths (bg(tone), fg(tone.foreground), …).
  Authors never write tone="default" — omit the attribute for rest.
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

  // Default solid rides component.chip tokens (muted secondary — not primary).
  // Brand tones only when tone= is set. gap-1 keeps the dismiss mark tight.
  const base = style({ box: display(inline-flex).minH(control-h-2xs).padX(2).padY(px), paint: group().appearance(none).cursor(pointer).select(none).tapHighlight(none).radius(full).borderWidth(1).border(solid).bg(chip).fg(chip-foreground).border(chip-border).transition(colors).duration(fast).ease(out), text: fontFamily(body).fontWeight(medium).leading(none).whitespace(nowrap).fontSize(xs) })
  const sizeMd = style({ box: minH(control-h-2xs).padX(2_5), text: fontSize(sm) })
  const sizeSm = style({ box: minH(control-h-2xs).padX(2), text: fontSize(xs) })
  const sizeLg = style({ box: minH(control-h-xs).padX(3), text: fontSize(sm) })
  const remove = style({ box: display(none).size(icon-size-xs).pad(0), paint: borderWidth(0).bg(transparent).fg(currentColor).cursor(pointer).radius(full).transition(colors).duration(fast).ease(out) })
  const leading = style({ box: display(contents) })
  const xmark = style({ box: display(block).size(full), paint: stroke(currentColor).fill(none) })
</scope>

<row
  inline
  nowrap
  align="center"
  gap="1"
  use:pressable
  use:pressed
  use:dismissible

>
  <span style([
    base,
    match(size, {
      md: sizeMd,
      sm: sizeSm,
      lg: sizeLg,
    }),
    // Bound tone expands over Tone groups with [tone='…'] gates.
    // Omit tone → only base/component chip paint (no tone attr rules match).
    match(variant, {
      solid: style({ paint: bg(tone).fg(tone.foreground).border(tone) }).when(hover, { paint: opacity(90) }),
      soft: style({ paint: fg(tone) }).when(hover, { paint: opacity(90) }),
      outline: style({ paint: bg(transparent).fg(tone).border(tone) }).when(hover, { paint: bg(hover-bg) }),
      ghost: style({ paint: bg(transparent).border(transparent).fg(tone) }).when(hover, { paint: bg(hover-bg) }),
    }),
    // Muted solid uses border (not muted) — not pure group-member paint.
    compound({ variant: solid, tone: muted }, style({ paint: bg(muted).fg(muted-foreground).border(border) })),
    compound({ variant: outline, tone: muted }, style({ paint: fg(muted-foreground).border(border) })),
    compound({ variant: ghost, tone: muted }, style({ paint: fg(muted-foreground) })),
    compound({ selected: true }, style({ paint: bg(primary).fg(primary-foreground).border(primary).shadow(xs) })),
    compound({ selected: true, tone: muted }, style({ paint: bg(primary).fg(primary-foreground).border(primary) })),
    when({
      pressed: style({ paint: bg(primary).fg(primary-foreground).border(primary).shadow(xs), text: fontWeight(semibold) }),
      disabled: style({ paint: cursor(not-allowed).bg(chip).border(border-subtle).fg(chip-foreground).saturate(65).shadow(none) }),
      focus: style({ paint: outline(solid).outline(2).outline(ring).outlineOffset(2) }),
    }),
    compound({ selected: true }, when({
      hover: style({ paint: opacity(90) }),
    })),
  ])>    <span style(leading)>
    <slot name="leading"></slot>
  </span>
  <slot></slot>
  <button type="button" anatomy data-dismiss label="Remove" tabindex="-1" style([ style({ box: shrink(0) }), 
      remove,
      when({
        group-removable: style({ box: display(inline-flex) }),
        hover: style({ paint: bg(chip).fg(chip-foreground) }),
        focus: style({ paint: outline(solid).outline(2).outline(ring).outlineOffset(2) }),
      }),
    ])>
    <svg viewBox="0 0 10 10" hidden="true" stroke-width="1.75" stroke-linecap="round" style(xmark)>
      <path d="M2 2 L8 8 M8 2 L2 8"></path>
    </svg>
  </button>
  </span>
</row>
