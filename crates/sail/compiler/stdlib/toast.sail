<!--
  <toast> — auto-dismiss + close. Style value language only.
  Start stripe rest = toast.stripe when tone is omitted.
  Semantic paint = bound tone group paths. Authors omit tone for rest (never tone="default").
-->

<scope>
  props: {
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    duration = ''
  
    onDismiss: () => void
}

  const base = style({ box: padY(3).padX(4), paint: fg(toast.foreground).bg(toast).radius(md).shadow(md).borderWidth(1).border(toast.border).borderSWidth(3).borderS(toast.stripe).pointerEvents(auto), text: fontFamily(body).fontSize(sm) })
  const close = style({ box: ms(auto), paint: fg(toast.foreground).bg(transparent).borderWidth(0).radius(sm).cursor(pointer).opacity(70) })
</scope>

<row align="start" gap="3" use:autoDismiss use:dismissControl style(base, { paint: borderS(tone), compound: [ { tone: muted, paint: borderS(muted-foreground) } ] })>
  <slot></slot>
  <button data-toast-close type="button" anatomy label="Dismiss" style(close, { box: shrink(0).grow(0), when: { hover: { paint: fg(toast.foreground).bg(toast).opacity(100) } } })>✕</button>
</row>
