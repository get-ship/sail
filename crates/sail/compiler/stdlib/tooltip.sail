<!--
  <tooltip> — hover/focus text bubble. use:tipDescribe supplies geometry vars.
  Style value language only.
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

<span use:tipDescribe sx={host}>
  <slot></slot>
  <span
    class="bubble"
    role="tooltip"
    sx={[
      bubble,
      when({
        group-hover: style.opacity(100),
        group-focus-within: style.opacity(100),
        after: arrow,
      }),
    ]}
  ></span>
</span>
