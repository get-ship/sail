<!--
  <empty-state> — L1 centered empty placeholder.
  Style value language only. size/variant ordinary props.
  tone defaults to muted — paints the icon slot (empty icon + tone is a no-op).
  Semantic paint = bound tone (icon fg).
-->

<scope>
  props: {
    variant: 'default' | 'inline' | 'placeholder' | 'connecting' = 'default'
    size: 'md' | 'sm' | 'lg' = 'md'
    tone: 'primary' | 'info' | 'success' | 'warning' | 'danger' | 'accent' | 'muted' = 'muted'
    compact = false
  }

  style base = {pad({ y: 16, x: 4 }).fg(muted-foreground).border({ color: border, width: 1 }).radius(lg).bg(muted).elevation(xs).type({ family: body, align: center })}
  style sizeMd = {pad({ y: 16, x: 4 })}
  style sizeSm = {pad({ y: 7, x: 4 })}
  style sizeLg = {pad({ y: 24, x: 6 })}
  style variantDefault = {border({ color: border, width: 1 }).radius(lg).bg(muted).elevation(xs).type({ align: center })}
  style variantInline = {pad({ y: 3, x: 4 }).border({ color: border, width: 1, style: dashed }).radius(md).bg(muted).elevation(none).type({ align: start })}
  // placeholder + connecting share bare (borderless/transparent) chrome.
  style variantBare = {pad({ y: 6 }).border({ width: 0 }).bg(transparent).radius(none).elevation(none).type({ align: center })}
  // Style cannot set gap (layout attr only) — compact is pad + icon.
  style compactOn = {pad({ y: 4, x: 3 })}
  style icon = {self(center).size(12).radius(lg).bg(muted).fg(muted-foreground)}
  style copy = {self(center).size({ max-w: 38ch, w: 100% })}
  style title = {fg(foreground).type({ family: heading, weight: semibold, leading: tight, size: md, align: center })}
  style desc = {size({ max-w: 38ch }).fg(muted-foreground).type({ leading: normal, size: sm, align: center })}
  style actions = {self(center)}
</scope>

<column
  align="center"
  justify="center"
  gap="3"
  use:variantLive
  style(base, {when: {forced-colors: border({ color: CanvasText })}, match: { size: { md: sizeMd, sm: sizeSm, lg: sizeLg }, variant: { default: variantDefault, inline: variantInline, placeholder: variantBare, connecting: variantBare }, compact: { true: compactOn } }})
>
  <row
    inline
    nowrap
    align="center"
    justify="center"
    hidden="true"
    style(icon, {shrink(1).bg(muted).fg(tone), when: {empty: display(none), forced-colors: bg(Canvas).fg(CanvasText)}, compound: [{tone: muted, bg(muted).fg(muted-foreground)}, {compact: true, size(8)}]})
  >
    <slot name="icon"></slot>
  </row>
  <column align="center" gap="1" style(copy, {size({ min-w: 0 })})>
    <div style(title, {when: {empty: display(none)}})>
      <slot name="title"></slot>
    </div>
    <div style(desc, {when: {empty: display(none)}})>
      <slot></slot>
    </div>
  </column>
  <row align="center" justify="center" gap="2" style(actions, {when: {empty: display(none)}})>
    <slot name="action"></slot>
  </row>
</column>
