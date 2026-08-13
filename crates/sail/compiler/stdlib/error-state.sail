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

  style base = {box: space({ pad-y: 8, pad-x: 4 }), paint: ink({ color: error-state.foreground }).border({ color: error-state.border, width: 1 }).radius(lg).fill({ color: error-state }).elevation(xs).group(), text: type({ family: body, align: center })}
  style sizeMd = {box: space({ pad-y: 8, pad-x: 4 })}
  style sizeSm = {box: space({ pad-y: 7 })}
  style sizeLg = {box: space({ pad-y: 8, pad-x: 6 })}
  style variantDefault = {paint: border({ color: error-state.border, width: 1 }).radius(lg).fill({ color: error-state }).elevation(xs), text: type({ align: center })}
  // Inline: brand-destructive border + muted fill (intentional).
  style variantInline = {box: space({ pad-y: 3, pad-x: 4 }), paint: border({ color: destructive, width: 1 }).radius(md).fill(muted).elevation(none), text: type({ align: start })}
  // Icon face: brand-destructive on muted wash (intentional, not pack rest).
  style icon = {box: flow({ self: center }).size(control-h-xl), paint: radius(lg).fill(muted).ink({ color: destructive }).border({ width: 1, kind: outline, style: solid, color: destructive, offset: ring-inset })}
  style glyph = {box: size(control-h-2xs)}
  style copy = {box: flow({ self: stretch })}
  // Strong title ink — free foreground (same path as empty-state title weight).
  style title = {paint: ink(foreground), text: type({ family: heading, weight: semibold, leading: tight, size: md })}
  style desc = {box: size({ max-w: panel-ch-w-md }), paint: ink({ color: error-state.foreground }), text: type({ leading: loose, size: sm })}
  style codeBox = {box: space({ pad-y: 1, pad-x: 2, mt: 1 }), paint: ink({ color: error-state.foreground }).fill({ color: error-state }).border({ color: error-state.border, width: 1 }).radius(sm), text: type({ family: mono, size: xs })}
  style detailsToggle = {box: flow({ display: none }).space({ mt: 1, pad-y: 1, pad-x: 2 }), paint: border({ width: 0 }).fill({ color: transparent }).ink({ color: error-state.foreground }).radius(sm).cursor(pointer), text: type({ family: body, size: xs })}
  style details = {box: flow({ display: none }).space({ mt: 2, pad-y: 2, pad-x: 3 }).size({ max-w: panel-ch-w-lg }), paint: fill({ color: error-state }).radius(sm).ink({ color: error-state.foreground }), text: type({ family: mono, size: xs, wrap: words, align: start })}
  style actions = {box: flow({ self: center })}
</scope>

<column
  align="stretch"
  justify="center"
  gap="3"
  use:recoverySurface
  style(base, {when: {forced-colors: {paint: border({ color: CanvasText })}}, match: {
      size: { md: sizeMd, sm: sizeSm, lg: sizeLg },
      variant: { default: variantDefault, inline: variantInline },
    }})
>
  <row
    inline
    nowrap
    align="center"
    justify="center"
    hidden="true"
    style(icon, {box: flow({ shrink: 1 }), when: {forced-colors: {paint: fill({ color: Canvas }).ink({ color: CanvasText })}}})
  >
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" style(glyph)>
      <circle cx="12" cy="12" r="9"></circle>
      <path d="M12 8v5"></path>
      <path d="M12 16h.01"></path>
    </svg>
  </row>
  <column align="stretch" gap="1" style(copy, {box: flow({ grow: 1 }).size({ min-w: 0 })})>
    <div style(title)>
      <slot name="title">Something went wrong</slot>
    </div>
    <div style(desc, {when: {empty: {box: flow({ display: none })}}})>
      <slot></slot>
    </div>
    <div style(codeBox, {when: {empty: {box: flow({ display: none })}}})></div>
    <button
      type="button"
      anatomy
      expanded="false"
      style(detailsToggle, {when: {group-state-has-details: {box: flow({ display: inline-block })}, hover: {paint: ink(foreground).fill({ color: error-state })}, focus: {paint: border({ style: solid, kind: outline, width: 2, color: ring, offset: 2 })}}})
    >Show details</button>
    <div style(details, {when: {group-state-open: {box: flow({ display: block })}}})>
      <slot name="details"></slot>
    </div>
  </column>
  <row align="center" justify="center" gap="2" style(actions, {when: {empty: {box: flow({ display: none })}}})>
    <slot name="action"></slot>
  </row>
</column>
