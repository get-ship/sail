<!--
  <error-state> — L1 recovery surface. Style value language only.
  Rest chrome = component.error-state tokens when not brand/destructive.
  Destructive paint via semantic tokens (no tone-* channels).
-->

<scope>
  props: {
    variant: 'default' | 'inline' = 'default'
    size: 'md' | 'sm' | 'lg' = 'md'
    code = ''
  }

  style base = { box: padY(8).padX(4), paint: fg(error-state.foreground).borderWidth(1).border(error-state.border).radius(lg).bg(error-state).shadow(xs).group(), text: fontFamily(body).textAlign(center) }
  style sizeMd = { box: padY(8).padX(4) }
  style sizeSm = { box: padY(7) }
  style sizeLg = { box: padY(8).padX(6) }
  style variantDefault = { paint: borderWidth(1).border(error-state.border).radius(lg).bg(error-state).shadow(xs), text: textAlign(center) }
  // Inline: brand-destructive border + muted fill (intentional).
  style variantInline = { box: padY(3).padX(4), paint: borderWidth(1).border(destructive).radius(md).bg(muted).shadow(none), text: textAlign(start) }
  // Icon face: brand-destructive on muted wash (intentional, not pack rest).
  style icon = { box: self(center).size(control-h-xl), paint: radius(lg).bg(muted).fg(destructive).outline(1).outline(solid).outline(destructive).outlineOffset(ring-inset) }
  style glyph = { box: size(control-h-2xs) }
  style copy = { box: self(stretch) }
  // Strong title ink — free foreground (same path as empty-state title weight).
  style title = { paint: fg(foreground), text: fontFamily(heading).fontWeight(semibold).leading(tight).fontSize(md) }
  style desc = { box: maxW(panel-ch-w-md), paint: fg(error-state.foreground), text: leading(loose).fontSize(sm) }
  style codeBox = { box: padY(1).padX(2).mt(1), paint: fg(error-state.foreground).bg(error-state).borderWidth(1).border(error-state.border).radius(sm), text: fontFamily(mono).fontSize(xs) }
  style detailsToggle = { box: display(none).mt(1).padY(1).padX(2), paint: borderWidth(0).bg(transparent).fg(error-state.foreground).radius(sm).cursor(pointer), text: fontFamily(body).fontSize(xs) }
  style details = { box: display(none).mt(2).padY(2).padX(3).maxW(panel-ch-w-lg), paint: bg(error-state).radius(sm).fg(error-state.foreground), text: fontFamily(mono).fontSize(xs).whitespace(pre-wrap).break(words).textAlign(start) }
  style actions = { box: self(center) }
</scope>

<column
  align="stretch"
  justify="center"
  gap="3"
  use:recoverySurface
  style(base, {
    when: { forced-colors: { paint: border(CanvasText) } },
    match: {
      size: { md: sizeMd, sm: sizeSm, lg: sizeLg },
      variant: { default: variantDefault, inline: variantInline },
    },
  })
>
  <row
    inline
    nowrap
    align="center"
    justify="center"
    hidden="true"
    style(icon, {
      box: shrink(1),
      when: { forced-colors: { paint: bg(Canvas).fg(CanvasText) } },
    })
  >
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" style(glyph)>
      <circle cx="12" cy="12" r="9"></circle>
      <path d="M12 8v5"></path>
      <path d="M12 16h.01"></path>
    </svg>
  </row>
  <column align="stretch" gap="1" style(copy, { box: grow(1).minW(0) })>
    <div style(title)>
      <slot name="title">Something went wrong</slot>
    </div>
    <div style(desc, { when: { empty: { box: display(none) } } })>
      <slot></slot>
    </div>
    <div style(codeBox, { when: { empty: { box: display(none) } } })></div>
    <button
      type="button"
      anatomy
      expanded="false"
      style(detailsToggle, {
        when: {
          group-state-has-details: { box: display(inline-block) },
          hover: { paint: fg(foreground).bg(error-state) },
          focus: { paint: outline(solid).outline(2).outline(ring).outlineOffset(2) },
        },
      })
    >Show details</button>
    <div style(details, { when: { group-state-open: { box: display(block) } } })>
      <slot name="details"></slot>
    </div>
  </column>
  <row align="center" justify="center" gap="2" style(actions, { when: { empty: { box: display(none) } } })>
    <slot name="action"></slot>
  </row>
</column>
