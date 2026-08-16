<!--
  <toast> — auto-dismiss + close. Style value language only.
  tone defaults to info — a bare <toast> is a finished notice.
  Semantic paint = bound tone group paths.
-->

<scope>
  props: {
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'info'
    duration = ''
    onDismiss: () => void
  }

  style base = {pad({ y: 3, x: 4 }).fg(foreground).bg({ color: tone, mix: 15 }).radius(md).elevation(md).border({ color: border, width: 1, edge: b }).border({ color: border, width: 1, edge: e }).border({ color: border, width: 1, edge: t }).border({ color: tone, width: 3, edge: s }).pointerEvents(auto).type({ family: body, size: sm })}
  // Text ✕ in a square: inline-flex; inner row supplies align/justify center.
  style close = {display(inline-flex).margin({ s: auto }).size({ w: control-h-xs, h: control-h-xs }).pad(0).fg(foreground).bg(transparent).border({ width: 0 }).radius(sm).filter({ opacity: 70 }).cursor(pointer).type({ leading: none, align: center })}
  style closeGlyph = {size({ w: full, h: full })}
</scope>

<row
  align="start"
  gap="3"
  use:autoDismiss
  use:dismissControl
  style(base, {compound: [
      {tone: muted, border({ color: muted-foreground, edge: s }).bg(muted)},
    ]})
>
  <slot></slot>
  <button
    data-toast-close
    type="button"
    anatomy
    label="Dismiss"
    style(close, {shrink(0).grow(0), when: {hover: fg(foreground).bg(card).filter({ opacity: 100 })}})
  >
    <row inline nowrap align="center" justify="center" style(closeGlyph)>✕</row>
  </button>
</row>
