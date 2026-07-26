<!--
  <alert> — L1 inline status banner. Style value language only.
  Stripe via when(before) + match(tone) semantic tokens.
-->

<scope>
  props: {
    tone: 'default' | 'muted' | 'info' | 'success' | 'warning' | 'destructive' = 'default'
    dismissible = false
  }

  const base = style.group().position(relative).overflow(hidden).pad(3).borderWidth(1).border(border).radius(md).bg(alert).fg(alert-foreground).fontFamily(body).fontSize(base).leading(normal).shadow(xs)
  const stripe = style.content(empty).position(absolute).left(0).top(0).bottom(0).w(1).bg(alert-stripe)
  const ico = style.size(control-h-xs).radius(md).bg(muted).fg(muted-foreground)
  const title = style.fontWeight(semibold)
  const body = style.fg(muted-foreground)
  const actions = style.mt(2)
  const close = style.display(none).size(control-h-xs).self(start).pad(0).borderWidth(0).radius(sm).bg(transparent).fg(muted-foreground).cursor(pointer).transition(colors).duration(fast).ease(out)
  const xmark = style.display(block).size(icon-size-sm).stroke(currentColor).fill(none)
</scope>

<row
  nowrap
  align="start"
  gap="3"
  use:liveRegion
  use:dismissControl
  role="status"
  aria-live="polite"
  aria-atomic="true"
  sx={[
    base,
    when({
      before: stripe,
      forced-colors: style.border(CanvasText),
    }),
    match(tone, {
      default: when({ before: style.bg(alert-stripe) }),
      muted: when({ before: style.bg(alert-stripe) }),
      info: when({ before: style.bg(info) }),
      success: when({ before: style.bg(success) }),
      warning: when({ before: style.bg(warning) }),
      destructive: when({ before: style.bg(destructive) }),
    }),
    when({
      forced-colors: when({
        before: style.bg(CanvasText),
      }),
    }),
  ]}
>
  <row inline nowrap align="center" justify="center" class="ico" shrink sx={[
    ico,
    match(tone, {
      default: style.bg(muted).fg(muted-foreground),
      muted: style.bg(muted).fg(muted-foreground),
      info: style.bg(muted).fg(info),
      success: style.bg(muted).fg(success),
      warning: style.bg(muted).fg(warning),
      destructive: style.bg(muted).fg(destructive),
    }),
    when({
      empty: style.display(none),
    }),
  ]}>
    <slot name="icon"></slot>
  </row>
  <column gap="1" class="content" grow min-w="0">
    <div class="title" sx={[
      title,
      when({
        empty: style.display(none),
      }),
    ]}>
      <slot name="title"></slot>
    </div>
    <div class="body" sx={body}>
      <slot></slot>
    </div>
    <row align="center" gap="2" class="actions" sx={[
      actions,
      when({
        empty: style.display(none),
      }),
    ]}>
      <slot name="actions"></slot>
    </row>
  </column>
  <button
    class="close"
    type="button"
    anatomy
    aria-label="Dismiss"
    shrink
    sx={[
      close,
      when({
        group-dismissible: style.display(inline-block),
        hover: style.bg(muted).fg(foreground),
        focus: style.outline(solid).outline(2).outline(ring).outlineOffset(2),
      }),
    ]}
  >
    <svg viewBox="0 0 10 10" aria-hidden="true" sx={xmark} stroke-width="1.75" stroke-linecap="round">
      <path d="M2 2 L8 8 M8 2 L2 8"></path>
    </svg>
  </button>
</row>
