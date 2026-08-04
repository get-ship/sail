<!--
  <toast> — auto-dismiss + close. Style value language only.
  Start stripe via borderSWidth + semantic tone tokens.
-->

<scope>
  props: {
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    duration = ''
  }

  const base = style.fontFamily(body).fontSize(sm).fg(toast-foreground).bg(toast).radius(md).shadow(md).padY(3).padX(4).borderWidth(1).border(toast-border).borderSWidth(3).borderS(foreground).pointerEvents(auto)
  const close = style.ms(auto).fg(muted-foreground).bg(transparent).borderWidth(0).radius(sm).cursor(pointer)
</scope>

<row align="start" gap="3" use:autoDismiss use:dismissControl sx={[
  base,
  match(tone, {
    default: style.borderS(foreground),
    primary: style.borderS(primary),
    info: style.borderS(info),
    success: style.borderS(success),
    warning: style.borderS(warning),
    destructive: style.borderS(destructive),
    accent: style.borderS(accent),
    muted: style.borderS(muted-foreground),
  }),
]}>
  <slot></slot>
  <button data-toast-close type="button" anatomy label="Dismiss" shrink="0" grow="0" sx={[
    close,
    when({
      hover: style.fg(foreground).bg(muted),
    }),
  ]}>✕</button>
</row>
