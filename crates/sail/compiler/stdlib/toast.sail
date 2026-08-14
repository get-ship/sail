<!--
  <toast> — auto-dismiss + close. Style value language only.
  Start stripe rest = primary when tone is omitted.
  Semantic paint = bound tone group paths.
-->

<scope>
  props: {
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    duration = ''
    onDismiss: () => void
  }

  style base = {pad({ y: 3, x: 4 }).fg(foreground).bg(card).radius(md).elevation(md).border({ color: border, width: 1, edge: b }).border({ color: border, width: 1, edge: e }).border({ color: border, width: 1, edge: t }).border({ color: primary, width: 3, edge: s }).pointerEvents(auto).type({ family: body, size: sm })}
  style close = {margin({ s: auto }).fg(foreground).bg(transparent).border({ width: 0 }).radius(sm).filter({ opacity: 70 }).cursor(pointer)}
</scope>

<row
  align="start"
  gap="3"
  use:autoDismiss
  use:dismissControl
  style(base, {border({ color: tone, edge: s }), compound: [
      {tone: muted, border({ color: muted-foreground, edge: s })},
    ]})
>
  <slot></slot>
  <button
    data-toast-close
    type="button"
    anatomy
    label="Dismiss"
    style(close, {shrink(0).grow(0), when: {hover: fg(foreground).bg(card).filter({ opacity: 100 })}})
  >✕</button>
</row>
