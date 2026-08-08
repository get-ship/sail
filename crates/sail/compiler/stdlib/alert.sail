<!--
  <alert> — L1 inline status banner. Style value language only.
  Stripe / icon rest = alert tokens when tone is omitted.
  Semantic paint = bound tone group paths. Authors omit tone for rest (never tone="default").
  Host subset: muted | info | success | warning | destructive (no primary/accent product use).
-->

<scope>
  props: {
    tone: 'muted' | 'info' | 'success' | 'warning' | 'destructive'
    dismissible = false
    onDismiss: () => void
  }

  style base = { box: position(relative).overflow(hidden).pad(3), paint: group().borderWidth(1).border(alert.border).radius(md).bg(alert).fg(alert.foreground).shadow(xs), text: fontFamily(body).fontSize(base).leading(normal) }
  // ::before stripe geometry (rest fill alert.stripe; host attaches tone via when.before).
  style stripe = { box: position(absolute).left(0).top(0).bottom(0).w(1), paint: content(empty).bg(alert.stripe) }
  style ico = { box: size(control-h-xs), paint: radius(md).bg(alert).fg(alert.stripe) }
  style title = { text: fontWeight(semibold) }
  style body = { paint: fg(alert.stripe) }
  style actions = { box: mt(2) }
  style close = { box: display(none).size(control-h-xs).self(start).pad(0), paint: borderWidth(0).radius(sm).bg(transparent).fg(alert.stripe).cursor(pointer).transition(colors).duration(fast).ease(out) }
  style xmark = { box: display(block).size(icon-size-sm), paint: stroke(currentColor).fill(none) }
  style growCol = { box: grow(1).minW(0) }
</scope>

<row
  nowrap
  align="start"
  gap="3"
  use:liveRegion
  use:dismissControl
  style(base, {
    when: {
      before: style(stripe, { paint: bg(tone) }),
      forced-colors: {
        paint: border(CanvasText),
        when: { before: { paint: bg(CanvasText) } },
      },
    },
    compound: [
      { tone: muted, when: { before: { paint: bg(alert.stripe) } } },
    ],
  })
>
  <row
    inline
    nowrap
    align="center"
    justify="center"
    style(ico, {
      box: shrink(1),
      paint: fg(tone),
      when: { empty: { box: display(none) } },
      compound: [{ tone: muted, paint: bg(alert).fg(alert.stripe) }],
    })
  >
    <slot name="icon"></slot>
  </row>
  <column gap="1" style(growCol)>
    <div style(title, { when: { empty: { box: display(none) } } })>
      <slot name="title"></slot>
    </div>
    <div style(body)>
      <slot></slot>
    </div>
    <row align="center" gap="2" style(actions, { when: { empty: { box: display(none) } } })>
      <slot name="actions"></slot>
    </row>
  </column>
  <button
    data-dismiss
    type="button"
    anatomy
    label="Dismiss"
    style(close, {
      box: shrink(1),
      when: {
        group-dismissible: { box: display(inline-block) },
        hover: { paint: bg(alert).fg(alert.foreground) },
        focus: { paint: outline(solid).outline(2).outline(ring).outlineOffset(2) },
      },
    })
  >
    <svg viewBox="0 0 10 10" hidden="true" stroke-width="1.75" stroke-linecap="round" style(xmark)>
      <path d="M2 2 L8 8 M8 2 L2 8"></path>
    </svg>
  </button>
</row>
