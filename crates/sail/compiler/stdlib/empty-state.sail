<!--
  <empty-state> — L1 centered empty placeholder.
  Style value language only. size/variant ordinary props.
  Icon rest = muted when tone is omitted.
  Semantic paint = bound tone (icon fg).
-->

<scope>
  props: {
    variant: 'default' | 'inline' | 'placeholder' | 'connecting' = 'default'
    size: 'md' | 'sm' | 'lg' = 'md'
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
  }

  style base = {box: space({ pad-y: 16, pad-x: 4 }), paint: ink({ color: muted-foreground }).border({ color: border, width: 1 }).radius(lg).fill({ color: muted }).elevation(xs), text: type({ family: body, align: center })}
  style sizeMd = {box: space({ pad-y: 16, pad-x: 4 })}
  style sizeSm = {box: space({ pad-y: 7, pad-x: 4 })}
  style sizeLg = {box: space({ pad-y: 24, pad-x: 6 })}
  style variantDefault = {paint: border({ color: border, width: 1 }).radius(lg).fill({ color: muted }).elevation(xs), text: type({ align: center })}
  style variantInline = {box: space({ pad-y: 3, pad-x: 4 }), paint: border({ color: border, width: 1, style: dashed }).radius(md).fill({ color: muted }).elevation(none), text: type({ align: start })}
  // placeholder + connecting share bare (borderless/transparent) chrome.
  style variantBare = {box: space({ pad-y: 6 }), paint: border({ width: 0 }).fill({ color: transparent }).radius(none).elevation(none), text: type({ align: center })}
  style icon = {box: flow({ self: center }).size(12), paint: radius(lg).fill({ color: muted }).ink({ color: muted-foreground })}
  style copy = {box: flow({ self: stretch })}
  style title = {paint: ink({ color: foreground }), text: type({ family: heading, weight: semibold, leading: tight, size: md })}
  style desc = {box: size({ max-w: panel-ch-w-md }), paint: ink({ color: muted-foreground }), text: type({ leading: loose, size: sm })}
  style actions = {box: flow({ self: center })}
</scope>

<column
  align="stretch"
  justify="center"
  gap="3"
  use:variantLive
  style(base, {when: {forced-colors: {paint: border({ color: CanvasText })}}, match: {
      size: { md: sizeMd, sm: sizeSm, lg: sizeLg },
      variant: {
        default: variantDefault,
        inline: variantInline,
        placeholder: variantBare,
        connecting: variantBare,
      },
    }})
>
  <row
    inline
    nowrap
    align="center"
    justify="center"
    hidden="true"
    style(icon, {box: flow({ shrink: 1 }), paint: fill({ color: muted }).ink({ color: tone }), when: {empty: {box: flow({ display: none })}, forced-colors: {paint: fill({ color: Canvas }).ink({ color: CanvasText })}}, compound: [{tone: muted, paint: fill({ color: muted }).ink({ color: muted-foreground })}]})
  >
    <slot name="icon"></slot>
  </row>
  <column align="stretch" gap="1" style(copy, {box: flow({ grow: 1 }).size({ min-w: 0 })})>
    <div style(title, {when: {empty: {box: flow({ display: none })}}})>
      <slot name="title"></slot>
    </div>
    <div style(desc, {when: {empty: {box: flow({ display: none })}}})>
      <slot></slot>
    </div>
  </column>
  <row align="center" justify="center" gap="2" style(actions, {when: {empty: {box: flow({ display: none })}}})>
    <slot name="action"></slot>
  </row>
</column>
