<!--
  <tooltip> — hover/focus text bubble. use:tipDescribe supplies per-instance
  anchor geometry vars (placement offsets + arrow insets), content text, and
  Escape. Style value language only.

  Paint (Style product only):
    host sx   — style({ paint: group() }) (sail-grp ancestor for the bubble show) +
                position(relative) positioning context
    bubble sx — absolute + (--tt-*) geometry via bind (the vars tipDescribe
                publishes, same bridge as use:anchor) + opacity show under
                group hover/focus-within + after: arrow
    Hide is instant (no opacity transition) so tooltips never linger after
    leave; tipDescribe also forces hide on mouseleave/focusout.
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

  style host = { box: position(relative), paint: group() }
  // No opacity transition — linger after leave was a real product bug.
  style bubble = { box: position(absolute).top(bind(ttTop)).bottom(bind(ttBottom)).left(bind(ttLeft)).right(bind(ttRight)).padY(1).padX(2), paint: translateX(bind(ttTx)).z(popover).opacity(0).bg(tooltip).fg(tooltip.foreground).radius(md).shadow(md).pointerEvents(none), text: fontFamily(body).fontSize(xs).fontWeight(medium).whitespace(nowrap) }
  style arrow = { box: position(absolute).w(2).h(2).top(bind(ttArrowTop)).bottom(bind(ttArrowBottom)).left(bind(ttArrowLeft)).right(bind(ttArrowRight)).ms(bind(ttArrowMl)).mt(bind(ttArrowMt)), paint: content(empty).bg(tooltip).rotate(45) }
</scope>

<span use:tipDescribe style(host)>
  <slot></slot>
  <!-- tooltip role stamped by use:tipDescribe (closed part anatomy) -->
  <span data-tooltip-bubble style(bubble, { when: { group-hover: { paint: opacity(100) }, group-focus-within: { paint: opacity(100) }, after: arrow } })></span>
</span>
