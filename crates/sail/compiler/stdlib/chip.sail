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
  const base = style.group().display(inline-flex).appearance(none).fontFamily(body).fontWeight(medium).leading(none).whitespace(nowrap).cursor(pointer).select(none).tapHighlight(none).radius(full).borderWidth(1).border(solid).minH(control-h-2xs).padX(2).padY(px).fontSize(xs).bg(chip).fg(chip-foreground).border(chip-border).transition(colors).duration(fast).ease(out)
  const sizeMd = style.minH(control-h-2xs).padX(2_5).fontSize(sm)
  const sizeSm = style.minH(control-h-2xs).padX(2).fontSize(xs)
  const sizeLg = style.minH(control-h-xs).padX(3).fontSize(sm)
  const remove = style.display(none).size(icon-size-xs).pad(0).borderWidth(0).bg(transparent).fg(currentColor).cursor(pointer).radius(full).transition(colors).duration(fast).ease(out)
  const leading = style.display(contents)
  const xmark = style.display(block).size(full).stroke(currentColor).fill(none)
</scope>

<row
  inline
  nowrap
  align="center"
  gap="1"
  use:pressable
  use:pressed
  use:dismissible
  sx={[
    base,
    match(size, {
      md: sizeMd,
      sm: sizeSm,
      lg: sizeLg,
    }),
    match(variant, {
      solid: match(tone, {
        default: style.bg(chip).fg(chip-foreground).border(chip-border),
        primary: style.bg(primary).fg(primary-foreground).border(primary),
        info: style.bg(info).fg(info-foreground).border(info),
        success: style.bg(success).fg(success-foreground).border(success),
        warning: style.bg(warning).fg(warning-foreground).border(warning),
        destructive: style.bg(destructive).fg(destructive-foreground).border(destructive),
        accent: style.bg(accent).fg(accent-foreground).border(accent),
        muted: style.bg(muted).fg(muted-foreground).border(border),
      }),
      // soft/outline/ghost key on variant so freeform hosts without tone= still paint.
      soft: style.bg(muted).fg(muted-foreground).border(border),
      outline: style.bg(transparent).fg(foreground).border(border),
      ghost: style.bg(transparent).fg(foreground).border(transparent),
    }),
    compound({ variant: soft, tone: primary }, style.fg(primary)),
    compound({ variant: soft, tone: info }, style.fg(info)),
    compound({ variant: soft, tone: success }, style.fg(success)),
    compound({ variant: soft, tone: warning }, style.fg(warning)),
    compound({ variant: soft, tone: destructive }, style.fg(destructive)),
    compound({ variant: soft, tone: accent }, style.fg(accent)),
    compound({ variant: outline, tone: primary }, style.fg(primary).border(primary)),
    compound({ variant: outline, tone: info }, style.fg(info).border(info)),
    compound({ variant: outline, tone: success }, style.fg(success).border(success)),
    compound({ variant: outline, tone: warning }, style.fg(warning).border(warning)),
    compound({ variant: outline, tone: destructive }, style.fg(destructive).border(destructive)),
    compound({ variant: outline, tone: accent }, style.fg(accent).border(accent)),
    compound({ variant: outline, tone: muted }, style.fg(muted-foreground)),
    compound({ variant: ghost, tone: primary }, style.fg(primary)),
    compound({ variant: ghost, tone: info }, style.fg(info)),
    compound({ variant: ghost, tone: success }, style.fg(success)),
    compound({ variant: ghost, tone: warning }, style.fg(warning)),
    compound({ variant: ghost, tone: destructive }, style.fg(destructive)),
    compound({ variant: ghost, tone: accent }, style.fg(accent)),
    compound({ variant: ghost, tone: muted }, style.fg(muted-foreground)),
    compound({ selected: true }, style.bg(primary).fg(primary-foreground).border(primary).shadow(xs)),
    compound({ selected: true, tone: muted }, style.bg(primary).fg(primary-foreground).border(primary)),
    when({
      pressed: style.bg(primary).fg(primary-foreground).border(primary).fontWeight(semibold).shadow(xs),
      disabled: style.cursor(not-allowed).bg(muted).border(border-subtle).fg(muted-foreground).saturate(65).shadow(none),
      focus: style.outline(solid).outline(2).outline(ring).outlineOffset(2),
    }),
    compound({ variant: solid, tone: default }, when({
      hover: style.opacity(90),
    })),
    compound({ variant: solid, tone: primary }, when({
      hover: style.opacity(90),
    })),
    compound({ variant: solid, tone: info }, when({
      hover: style.opacity(90),
    })),
    compound({ variant: solid, tone: success }, when({
      hover: style.opacity(90),
    })),
    compound({ variant: solid, tone: warning }, when({
      hover: style.opacity(90),
    })),
    compound({ variant: solid, tone: destructive }, when({
      hover: style.opacity(90),
    })),
    compound({ variant: solid, tone: accent }, when({
      hover: style.opacity(90),
    })),
    compound({ variant: solid, tone: muted }, when({
      hover: style.opacity(90),
    })),
    compound({ selected: true }, when({
      hover: style.opacity(90),
    })),
    compound({ variant: soft }, when({
      hover: style.opacity(90),
    })),
    compound({ variant: outline }, when({
      hover: style.bg(hover-bg),
    })),
    compound({ variant: ghost }, when({
      hover: style.bg(hover-bg),
    })),
  ]}
>
  <span sx={leading}>
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
    sx={[
      remove,
      when({
        group-removable: style.display(inline-flex),
        hover: style.bg(muted).fg(foreground),
        focus: style.outline(solid).outline(2).outline(ring).outlineOffset(2),
      }),
    ]}
  >
    <svg viewBox="0 0 10 10" hidden="true" sx={xmark} stroke-width="1.75" stroke-linecap="round">
      <path d="M2 2 L8 8 M8 2 L2 8"></path>
    </svg>
  </button>
</row>
