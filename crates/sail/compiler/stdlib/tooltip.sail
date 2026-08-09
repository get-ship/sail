<!--
  <tooltip> — hover/focus text bubble. use:tipDescribe supplies per-instance
  anchor geometry vars (placement offsets + arrow insets), content text, and
  Escape. Style value language only.
  Arrow is a border-colored caret (not a rotated square) so one edge is not a
  double/broken seam against the bubble fill + shadow.
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
    paint: translateX(bind(ttTx)).z(popover).opacity(0).bg(tooltip).fg(tooltip.foreground).radius(md).shadow(md).pointerEvents(none).borderWidth(0),
    text: fontFamily(body).fontSize(xs).fontWeight(medium).whitespace(nowrap),
  }
  // Zero-box border triangle; match placement points the filled edge outward.
  style arrowBox = {
    box: position(absolute).w(0).h(0).top(bind(ttArrowTop)).bottom(bind(ttArrowBottom)).left(bind(ttArrowLeft)).right(bind(ttArrowRight)).ms(bind(ttArrowMl)).mt(bind(ttArrowMt)),
    paint: content(empty).bg(transparent).border(solid).borderWidth(0),
  }
  // placement=top → caret on bubble bottom, points down (border-top filled).
  // Border widths are px scale (0|1|2|3|4|8) — 4px sides ≈ prior diamond size.
  style arrowFromTop = {
    paint: borderLWidth(4).borderRWidth(4).borderTWidth(4).borderBWidth(0).borderL(transparent).borderR(transparent).borderT(tooltip).borderB(transparent),
  }
  // placement=bottom → caret on bubble top, points up.
  style arrowFromBottom = {
    paint: borderLWidth(4).borderRWidth(4).borderBWidth(4).borderTWidth(0).borderL(transparent).borderR(transparent).borderB(tooltip).borderT(transparent),
  }
  // placement=left → caret on bubble right, points right.
  style arrowFromLeft = {
    paint: borderTWidth(4).borderBWidth(4).borderLWidth(4).borderRWidth(0).borderT(transparent).borderB(transparent).borderL(tooltip).borderR(transparent),
  }
  // placement=right → caret on bubble left, points left.
  style arrowFromRight = {
    paint: borderTWidth(4).borderBWidth(4).borderRWidth(4).borderLWidth(0).borderT(transparent).borderB(transparent).borderR(tooltip).borderL(transparent),
  }
</scope>

<span use:tipDescribe style(host)>
  <slot></slot>
  <!-- tooltip role stamped by use:tipDescribe (closed part anatomy) -->
  <span
    data-tooltip-bubble
    style(bubble, {
      match: {
        placement: {
          top: { when: { after: style(arrowBox, arrowFromTop) } },
          bottom: { when: { after: style(arrowBox, arrowFromBottom) } },
          left: { when: { after: style(arrowBox, arrowFromLeft) } },
          right: { when: { after: style(arrowBox, arrowFromRight) } },
        },
      },
      when: {
        group-hover: { paint: opacity(100) },
        group-focus-within: { paint: opacity(100) },
      },
    })
  ></span>
</span>
