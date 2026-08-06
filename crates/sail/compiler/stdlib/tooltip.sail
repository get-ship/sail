<!--
  <tooltip> — hover/focus text bubble. use:tipDescribe supplies per-instance
  anchor geometry vars (placement offsets + arrow insets), content text, and
  Escape. Style value language only.

  Paint (Style product only):
    host sx   — style.group() (sail-grp ancestor for the bubble show) +
                position(relative) positioning context
    bubble sx — absolute + (--tt-*) geometry via bind (the vars tipDescribe
                publishes, same bridge as use:anchor) + opacity show under
                group hover/focus-within + after: arrow
  No contracts PE matrix.

  Legacy utility-atom spellings this Style paint replaces (pinned by
  sx_host_paint_probe::tooltip_geometry_and_show_on_sx_no_contracts_pe):
    host   group relative
    bubble top-(--tt-top) translate-x-(--tt-tx)
           group-hover:opacity-100 group-focus-within:opacity-100
    arrow  after:content-empty after:rotate-45
-->

<scope>
  props: {
    placement: 'top' | 'bottom' | 'left' | 'right' = 'top'
    content = ''
  }

  const host = style.group().position(relative)
  const bubble = style.position(absolute).top(bind(ttTop)).bottom(bind(ttBottom)).left(bind(ttLeft)).right(bind(ttRight)).translateX(bind(ttTx)).z(popover).opacity(0).transition(opacity).duration(fast).ease(out).padY(1).padX(2).bg(tooltip).fg(tooltip-foreground).fontFamily(body).fontSize(xs).fontWeight(medium).radius(md).shadow(md).whitespace(nowrap).pointerEvents(none)
  const arrow = style.content(empty).position(absolute).w(2).h(2).bg(tooltip).rotate(45).top(bind(ttArrowTop)).bottom(bind(ttArrowBottom)).left(bind(ttArrowLeft)).right(bind(ttArrowRight)).ms(bind(ttArrowMl)).mt(bind(ttArrowMt))
</scope>

<span use:tipDescribe>
  style(host)

  <slot></slot>
  <span
    role="tooltip"

  >
    style([
      bubble,
      when({
        group-hover: style.opacity(100),
        group-focus-within: style.opacity(100),
        after: arrow,
      }),
    ])
</span>
</span>
