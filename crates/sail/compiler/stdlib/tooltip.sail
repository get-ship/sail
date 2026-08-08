<!--
  <tooltip> — hover/focus text bubble. use:tipDescribe supplies per-instance
  anchor geometry vars (placement offsets + arrow insets), content text, and
  Escape. Style value language only.
-->

<scope>
  props: {
    placement: 'top' | 'bottom' | 'left' | 'right' = 'top'
    content = ''
  }

  style host = { box: position(relative), paint: group() }
  // No opacity transition — linger after leave was a real product bug.
  style bubble = {
    box: position(absolute).top(bind(ttTop)).bottom(bind(ttBottom)).left(bind(ttLeft)).right(bind(ttRight)).padY(1).padX(2),
    paint: translateX(bind(ttTx)).z(popover).opacity(0).bg(tooltip).fg(tooltip.foreground).radius(md).shadow(md).pointerEvents(none),
    text: fontFamily(body).fontSize(xs).fontWeight(medium).whitespace(nowrap),
  }
  style arrow = {
    box: position(absolute).w(2).h(2).top(bind(ttArrowTop)).bottom(bind(ttArrowBottom)).left(bind(ttArrowLeft)).right(bind(ttArrowRight)).ms(bind(ttArrowMl)).mt(bind(ttArrowMt)),
    paint: content(empty).bg(tooltip).rotate(45),
  }
</scope>

<span use:tipDescribe style(host)>
  <slot></slot>
  <!-- tooltip role stamped by use:tipDescribe (closed part anatomy) -->
  <span
    data-tooltip-bubble
    style(bubble, {
      when: {
        group-hover: { paint: opacity(100) },
        group-focus-within: { paint: opacity(100) },
        after: arrow,
      },
    })
  ></span>
</span>
