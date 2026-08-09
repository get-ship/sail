<!--
  <chip> — filter/selection pill.
  Rest paint = component.chip tokens when tone is omitted.
  Semantic paint = bound tone group paths.
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
  // minH + padX own the box; padY(px) is a hairline so border does not kiss glyphs.
  // Vertical label centering is the paint-span floor in sail-base.css (items:center).
  style base = { box: display(inline-flex).minH(control-h-2xs).padX(2).padY(px), paint: group().appearance(none).cursor(pointer).select(none).tapHighlight(none).radius(full).borderWidth(1).border(solid).bg(chip).fg(chip.foreground).border(chip.border).transition(colors).duration(fast).ease(out), text: fontFamily(body).fontWeight(medium).leading(none).whitespace(nowrap).fontSize(xs) }
  style sizeMd = { box: minH(control-h-2xs).padX(2_5), text: fontSize(sm) }
  style sizeSm = { box: minH(control-h-2xs).padX(2), text: fontSize(xs) }
  style sizeLg = { box: minH(control-h-xs).padX(3), text: fontSize(sm) }
  style remove = { box: display(none).size(icon-size-xs).pad(0), paint: borderWidth(0).bg(transparent).fg(currentColor).cursor(pointer).radius(full).transition(colors).duration(fast).ease(out) }
  style leading = { box: display(contents) }
  style xmark = { box: display(block).size(full), paint: stroke(currentColor).fill(none) }
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
  <span
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
          solid: {
            paint: bg(tone).fg(tone.foreground).border(tone),
            when: { hover: { paint: opacity(90) } },
          },
          soft: {
            paint: colorMix(tone, 15).border(tone).fg(tone),
            when: { hover: { paint: colorMix(tone, 25) } },
          },
          outline: {
            paint: bg(transparent).fg(tone).border(tone),
            when: { hover: { paint: bg(hover-bg) } },
          },
          ghost: {
            paint: bg(transparent).border(transparent).fg(tone),
            when: { hover: { paint: bg(hover-bg) } },
          },
        },
      },
      compound: [
        { tone: muted, variant: solid, paint: bg(muted).fg(muted-foreground).border(border) },
        { tone: muted, variant: outline, paint: fg(muted-foreground).border(border) },
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
</row>
