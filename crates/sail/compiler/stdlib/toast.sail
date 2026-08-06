<!--
  <toast> — auto-dismiss + close. Style value language only.
  Start stripe via borderSWidth + semantic tone tokens.
-->

<scope>
  props: {
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    duration = ''
  }

  const base = style({ box: padY(3).padX(4), paint: fg(toast-foreground).bg(toast).radius(md).shadow(md).borderWidth(1).border(toast-border).borderSWidth(3).borderS(foreground).pointerEvents(auto), text: fontFamily(body).fontSize(sm) })
  const close = style({ box: ms(auto), paint: fg(muted-foreground).bg(transparent).borderWidth(0).radius(sm).cursor(pointer) })
</scope>

<row align="start" gap="3" use:autoDismiss use:dismissControl>
  style([
  base,
  match(tone, {
    default: style({ paint: borderS(foreground) }),
    primary: style({ paint: borderS(primary) }),
    info: style({ paint: borderS(info) }),
    success: style({ paint: borderS(success) }),
    warning: style({ paint: borderS(warning) }),
    destructive: style({ paint: borderS(destructive) }),
    accent: style({ paint: borderS(accent) }),
    muted: style({ paint: borderS(muted-foreground) }),
  }),
])

  <slot></slot>
  <button data-toast-close type="button" anatomy label="Dismiss">
    style([ style({ box: shrink(0).grow(0) }), 
    close,
    when({
      hover: style({ paint: fg(foreground).bg(muted) }),
    }),
  ])
✕</button>
</row>
