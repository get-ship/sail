<!--
  <error-state> — L1 recovery surface. Style value language only.
  Rest chrome = muted / destructive when not brand/destructive.
  Destructive paint via semantic tokens (no tone-* channels).
-->

<scope>
  props: {
    variant: 'default' | 'inline' = 'default'
    size: 'md' | 'sm' | 'lg' = 'md'
    code = ''
  }

  style base = {pad({ y: 8, x: 4 }).fg(destructive).border({ color: destructive, width: 1 }).radius(lg).bg(muted).elevation(xs).group().type({ family: body, align: center })}
  style sizeMd = {pad({ y: 8, x: 4 })}
  style sizeSm = {pad({ y: 7 })}
  style sizeLg = {pad({ y: 8, x: 6 })}
  style variantDefault = {border({ color: destructive, width: 1 }).radius(lg).bg(muted).elevation(xs).type({ align: center })}
  // Inline: brand-destructive border + muted fill (intentional).
  style variantInline = {pad({ y: 3, x: 4 }).border({ color: destructive, width: 1 }).radius(md).bg(muted).elevation(none).type({ align: start })}
  // Icon face: brand-destructive on muted wash (intentional, not pack rest).
  style icon = {self(center).size(control-h-xl).radius(lg).bg(muted).fg(destructive).border({ width: 1, kind: outline, style: solid, color: destructive, offset: ring-inset })}
  style glyph = {size(control-h-2xs)}
  style copy = {self(center).size({ max-w: panel-ch-w-md, w: full })}
  // Strong title ink — free foreground (same path as empty-state title weight).
  style title = {fg(foreground).type({ family: heading, weight: semibold, leading: tight, size: md, align: center })}
  style desc = {size({ max-w: panel-ch-w-md }).fg(destructive).type({ leading: loose, size: sm, align: center })}
  style codeBox = {pad({ y: 1, x: 2 }).margin({ t: 1 }).fg(destructive).bg(muted).border({ color: destructive, width: 1 }).radius(sm).type({ family: mono, size: xs })}
  style detailsToggle = {display(none).pad({ y: 1, x: 2 }).margin({ t: 1 }).border({ width: 0 }).bg(transparent).fg(destructive).radius(sm).cursor(pointer).type({ family: body, size: xs })}
  style details = {display(none).pad({ y: 2, x: 3 }).margin({ t: 2 }).size({ max-w: panel-ch-w-lg }).bg(muted).radius(sm).fg(destructive).type({ family: mono, size: xs, wrap: words, align: start })}
  style actions = {self(center)}
</scope>

<column
  align="center"
  justify="center"
  gap="3"
  use:recoverySurface
  style(base, {when: {forced-colors: border({ color: CanvasText })}, match: { size: { md: sizeMd, sm: sizeSm, lg: sizeLg }, variant: { default: variantDefault, inline: variantInline } }})
>
  <row
    inline
    nowrap
    align="center"
    justify="center"
    hidden="true"
    style(icon, {shrink(1), when: {forced-colors: bg(Canvas).fg(CanvasText)}})
  >
    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" style(glyph)>
      <circle cx="12" cy="12" r="9"></circle>
      <path d="M12 8v5"></path>
      <path d="M12 16h.01"></path>
    </svg>
  </row>
  <column align="center" gap="1" style(copy, {size({ min-w: 0 })})>
    <div style(title)>
      <slot name="title">Something went wrong</slot>
    </div>
    <div style(desc, {when: {empty: display(none)}})>
      <slot></slot>
    </div>
    <div style(codeBox, {when: {empty: display(none)}})></div>
    <button
      type="button"
      anatomy
      expanded="false"
      style(detailsToggle, {when: {group-state-has-details: display(inline-block), hover: fg(foreground).bg(muted), focus: border({ style: solid, kind: outline, width: 2, color: ring, offset: 2 })}})
    >Show details</button>
    <div style(details, {when: {group-state-open: display(block)}})>
      <slot name="details"></slot>
    </div>
  </column>
  <row align="center" justify="center" gap="2" style(actions, {when: {empty: display(none)}})>
    <slot name="action"></slot>
  </row>
</column>
