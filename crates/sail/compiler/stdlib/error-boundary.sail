<!--
  <error-boundary> — group-caught swap on Style when. Fallback chrome from
  component.error-boundary tokens.
-->
<scope>
  props: {
    message = ''
  }

  const root = style.display(contents).group()
  const content = style.display(contents)
  const fallback = style.display(none)
  const alert = style.padY(4).padX(5).bg(error-boundary).borderWidth(1).border(error-boundary-border).radius(md).shadow(xs).fg(error-boundary-foreground).fontFamily(body).fontSize(sm)
  const icon = style.shrink(0).size(icon-size-lg).fg(error-boundary-icon).mt(1)
  const body = style.minW(0)
  const message = style.fg(error-boundary-message-foreground).leading(normal)
  const retry = style.self(start).bg(error-boundary-retry-bg).borderWidth(1).border(border).radius(sm).padY(1).padX(3).fontFamily(body).fontSize(xs).fontWeight(semibold).fg(error-boundary-retry-foreground).shadow(xs).cursor(pointer)
</scope>
<div use:catchRender sx={root}>
  <div class="content" sx={[
    content,
    when({
      group-caught: style.display(none),
    }),
  ]}><slot></slot></div>
  <div class="fallback" sx={[
    fallback,
    when({
      group-caught: style.display(contents),
    }),
  ]}>
    <row align="start" gap="3" role="alert" sx={alert}>
      <slot name="fallback">
        <svg class="fallback-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" sx={icon}>
          <path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z"></path>
          <path d="M12 9v4"></path>
          <path d="M12 17h.01"></path>
        </svg>
        <column gap="1" class="fallback-body" sx={body}>
          <div class="message" sx={message}>Something went wrong</div>
          <button class="retry" type="button" anatomy sx={[
            retry,
            when({
              hover: style.bg(muted),
              focus: style.outline(solid).outline(2).outline(ring).outlineOffset(2),
            }),
          ]}>Retry</button>
        </column>
      </slot>
    </row>
  </div>
</div>
