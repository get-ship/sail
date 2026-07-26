<!--
  <chip> — L1 interactive filter/selection pill.
  Style value language only. tone/size/variant ordinary props.
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

  const base = style.fontFamily(body).fontWeight(medium).leading(none).align(middle).whitespace(nowrap).cursor(pointer).select(none).tapHighlight(none).radius(full).borderWidth(1).minH(control-h-xs).padX(2_5).fontSize(sm).bg(chip).fg(chip-foreground).border(chip-border).shadow(xs).transition(colors).duration(fast).ease(out)
  const sizeMd = style.minH(control-h-xs).padX(2_5).fontSize(sm)
  const sizeSm = style.minH(control-h-2xs).padX(2).fontSize(xs)
  const sizeLg = style.minH(control-h-md).padX(3).fontSize(base)
  const remove = style.size(icon-size-md).pad(0).borderWidth(0).bg(transparent).fg(currentColor).cursor(pointer).radius(full).transition(colors).duration(fast).ease(out)
  const leading = style.display(contents)
  const xmark = style.display(block).size(full).stroke(currentColor).fill(none)
</scope>

<row
  inline
  nowrap
  align="center"
  gap="2"
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
        default: style.bg(chip).fg(chip-foreground).border(chip-border).shadow(xs),
        primary: style.bg(primary).fg(primary-foreground).border(primary).shadow(xs),
        info: style.bg(info).fg(info-foreground).border(info).shadow(xs),
        success: style.bg(success).fg(success-foreground).border(success).shadow(xs),
        warning: style.bg(warning).fg(warning-foreground).border(warning).shadow(xs),
        destructive: style.bg(destructive).fg(destructive-foreground).border(destructive).shadow(xs),
        accent: style.bg(accent).fg(accent-foreground).border(accent).shadow(xs),
        muted: style.bg(muted).fg(muted-foreground).border(border).shadow(xs),
      }),
      soft: match(tone, {
        default: style.bg(muted).fg(muted-foreground).border(border).shadow(none),
        primary: style.bg(muted).fg(primary).border(border).shadow(none),
        info: style.bg(muted).fg(info).border(border).shadow(none),
        success: style.bg(muted).fg(success).border(border).shadow(none),
        warning: style.bg(muted).fg(warning).border(border).shadow(none),
        destructive: style.bg(muted).fg(destructive).border(border).shadow(none),
        accent: style.bg(muted).fg(accent).border(border).shadow(none),
        muted: style.bg(muted).fg(muted-foreground).border(border).shadow(none),
      }),
      outline: match(tone, {
        default: style.bg(transparent).fg(foreground).border(border).shadow(none),
        primary: style.bg(transparent).fg(primary).border(primary).shadow(none),
        info: style.bg(transparent).fg(info).border(info).shadow(none),
        success: style.bg(transparent).fg(success).border(success).shadow(none),
        warning: style.bg(transparent).fg(warning).border(warning).shadow(none),
        destructive: style.bg(transparent).fg(destructive).border(destructive).shadow(none),
        accent: style.bg(transparent).fg(accent).border(accent).shadow(none),
        muted: style.bg(transparent).fg(muted-foreground).border(border).shadow(none),
      }),
      ghost: match(tone, {
        default: style.bg(transparent).fg(foreground).border(transparent).shadow(none),
        primary: style.bg(transparent).fg(primary).border(transparent).shadow(none),
        info: style.bg(transparent).fg(info).border(transparent).shadow(none),
        success: style.bg(transparent).fg(success).border(transparent).shadow(none),
        warning: style.bg(transparent).fg(warning).border(transparent).shadow(none),
        destructive: style.bg(transparent).fg(destructive).border(transparent).shadow(none),
        accent: style.bg(transparent).fg(accent).border(transparent).shadow(none),
        muted: style.bg(transparent).fg(muted-foreground).border(transparent).shadow(none),
      }),
    }),
    compound({ selected: true, tone: muted }, style.bg(primary).fg(primary-foreground).border(primary)),
    when({
      hover: style.bg(muted),
      pressed: style.bg(primary).fg(primary-foreground).border(primary).fontWeight(semibold).shadow(xs),
      disabled: style.cursor(not-allowed).bg(muted).border(border-subtle).fg(muted-foreground).saturate(65).shadow(none),
      focus: style.outline(solid).outline(2).outline(ring).outlineOffset(2),
    }),
    compound({ variant: soft }, when({
      hover: style.bg(muted).border(foreground),
    })),
    compound({ variant: outline }, when({
      hover: style.bg(muted),
    })),
    compound({ variant: ghost }, when({
      hover: style.bg(muted),
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
    aria-label="Remove"
    tabindex="-1"
    shrink="0"
    sx={remove}
  >
    <svg viewBox="0 0 10 10" aria-hidden="true" sx={xmark} stroke-width="1.75" stroke-linecap="round">
      <path d="M2 2 L8 8 M8 2 L2 8"></path>
    </svg>
  </button>
</row>
