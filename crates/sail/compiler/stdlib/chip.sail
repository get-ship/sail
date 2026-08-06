<!--
  <chip> — filter/selection pill.
-->

<scope>
  props: {
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    variant: 'solid' | 'soft' | 'outline' | 'ghost' = 'solid'
    size: 'md' | 'sm' | 'lg' = 'md'
    selected = false
    removable = false
    disabled = false
  }

  // Default solid rides component.chip tokens (muted secondary — not primary).
  // Primary/info/… tones only when tone= is set. gap-1 keeps the dismiss mark
  // tight to the label (shadcn Badge density, not a padded button).
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
  style([
    base,
    match(size, {
      md: sizeMd,
      sm: sizeSm,
      lg: sizeLg,
    }),
    match(variant, {
      solid: match(tone, {
        default: style({ paint: bg(chip).fg(chip-foreground).border(chip-border) }),
        primary: style({ paint: bg(primary).fg(primary-foreground).border(primary) }),
        info: style({ paint: bg(info).fg(info-foreground).border(info) }),
        success: style({ paint: bg(success).fg(success-foreground).border(success) }),
        warning: style({ paint: bg(warning).fg(warning-foreground).border(warning) }),
        destructive: style({ paint: bg(destructive).fg(destructive-foreground).border(destructive) }),
        accent: style({ paint: bg(accent).fg(accent-foreground).border(accent) }),
        muted: style({ paint: bg(muted).fg(muted-foreground).border(border) }),
      }),
      // soft/outline/ghost key on variant so freeform hosts without tone= still paint.
      soft: style({ paint: bg(muted).fg(muted-foreground).border(border) }),
      outline: style({ paint: bg(transparent).fg(foreground).border(border) }),
      ghost: style({ paint: bg(transparent).fg(foreground).border(transparent) }),
    }),
    compound({ variant: soft, tone: primary }, style({ paint: fg(primary) })),
    compound({ variant: soft, tone: info }, style({ paint: fg(info) })),
    compound({ variant: soft, tone: success }, style({ paint: fg(success) })),
    compound({ variant: soft, tone: warning }, style({ paint: fg(warning) })),
    compound({ variant: soft, tone: destructive }, style({ paint: fg(destructive) })),
    compound({ variant: soft, tone: accent }, style({ paint: fg(accent) })),
    compound({ variant: outline, tone: primary }, style({ paint: fg(primary).border(primary) })),
    compound({ variant: outline, tone: info }, style({ paint: fg(info).border(info) })),
    compound({ variant: outline, tone: success }, style({ paint: fg(success).border(success) })),
    compound({ variant: outline, tone: warning }, style({ paint: fg(warning).border(warning) })),
    compound({ variant: outline, tone: destructive }, style({ paint: fg(destructive).border(destructive) })),
    compound({ variant: outline, tone: accent }, style({ paint: fg(accent).border(accent) })),
    compound({ variant: outline, tone: muted }, style({ paint: fg(muted-foreground) })),
    compound({ variant: ghost, tone: primary }, style({ paint: fg(primary) })),
    compound({ variant: ghost, tone: info }, style({ paint: fg(info) })),
    compound({ variant: ghost, tone: success }, style({ paint: fg(success) })),
    compound({ variant: ghost, tone: warning }, style({ paint: fg(warning) })),
    compound({ variant: ghost, tone: destructive }, style({ paint: fg(destructive) })),
    compound({ variant: ghost, tone: accent }, style({ paint: fg(accent) })),
    compound({ variant: ghost, tone: muted }, style({ paint: fg(muted-foreground) })),
    compound({ selected: true }, style({ paint: bg(primary).fg(primary-foreground).border(primary).shadow(xs) })),
    compound({ selected: true, tone: muted }, style({ paint: bg(primary).fg(primary-foreground).border(primary) })),
    when({
      pressed: style({ paint: bg(primary).fg(primary-foreground).border(primary).shadow(xs), text: fontWeight(semibold) }),
      disabled: style({ paint: cursor(not-allowed).bg(muted).border(border-subtle).fg(muted-foreground).saturate(65).shadow(none) }),
      focus: style({ paint: outline(solid).outline(2).outline(ring).outlineOffset(2) }),
    }),
    compound({ variant: solid, tone: default }, when({
      hover: style({ paint: opacity(90) }),
    })),
    compound({ variant: solid, tone: primary }, when({
      hover: style({ paint: opacity(90) }),
    })),
    compound({ variant: solid, tone: info }, when({
      hover: style({ paint: opacity(90) }),
    })),
    compound({ variant: solid, tone: success }, when({
      hover: style({ paint: opacity(90) }),
    })),
    compound({ variant: solid, tone: warning }, when({
      hover: style({ paint: opacity(90) }),
    })),
    compound({ variant: solid, tone: destructive }, when({
      hover: style({ paint: opacity(90) }),
    })),
    compound({ variant: solid, tone: accent }, when({
      hover: style({ paint: opacity(90) }),
    })),
    compound({ variant: solid, tone: muted }, when({
      hover: style({ paint: opacity(90) }),
    })),
    compound({ selected: true }, when({
      hover: style({ paint: opacity(90) }),
    })),
    compound({ variant: soft }, when({
      hover: style({ paint: opacity(90) }),
    })),
    compound({ variant: outline }, when({
      hover: style({ paint: bg(hover-bg) }),
    })),
    compound({ variant: ghost }, when({
      hover: style({ paint: bg(hover-bg) }),
    })),
  ])

  <span>
    style(leading)

    <slot name="leading"></slot>
  </span>
  <slot></slot>
  <button
    type="button"
    anatomy
    data-dismiss
    label="Remove"
    tabindex="-1"
    shrink="0"

  >
    style([
      remove,
      when({
        group-removable: style({ box: display(inline-flex) }),
        hover: style({ paint: bg(muted).fg(foreground) }),
        focus: style({ paint: outline(solid).outline(2).outline(ring).outlineOffset(2) }),
      }),
    ])

    <svg viewBox="0 0 10 10" hidden="true" stroke-width="1.75" stroke-linecap="round">
      style(xmark)

      <path d="M2 2 L8 8 M8 2 L2 8"></path>
    </svg>
  </button>
</row>
