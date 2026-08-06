<!--
  <separator> — native <hr>. Style value language only.
  Line paint via separator-line + semantic tones (no tone channel).
-->

<scope>
  props: {
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    tone: 'default' | 'muted' | 'primary' | 'destructive' = 'default'
    variant: 'solid' | 'dashed' = 'solid'
  }

  const base = style.display(block).w(full).h(border-width).my(2).borderWidth(0).pad(0).bg(separator-line)
  const orientH = style.display(block).w(full).h(border-width).my(2).mx(0)
  const orientV = style.display(inline-block).self(stretch).w(border-width).h(full).mx(2).my(0)
  const solid = style.bg(separator-line).borderWidth(0).pad(0)
  const dashed = style.bg(transparent).borderWidth(0).pad(0)
</scope>

<hr>
  style([
  base,
  match(orientation, {
    horizontal: orientH,
    vertical: orientV,
  }),
  match(variant, {
    solid: solid,
    dashed: dashed,
  }),
  compound({ variant: solid, tone: default }, style.bg(separator-line)),
  compound({ variant: solid, tone: muted }, style.bg(border-subtle)),
  compound({ variant: solid, tone: primary }, style.bg(primary)),
  compound({ variant: solid, tone: destructive }, style.bg(destructive)),
  compound({ variant: dashed, orientation: horizontal, tone: default }, style.h(0).borderTWidth(1).border(dashed).borderT(separator-line)),
  compound({ variant: dashed, orientation: horizontal, tone: muted }, style.h(0).borderTWidth(1).border(dashed).borderT(border-subtle)),
  compound({ variant: dashed, orientation: horizontal, tone: primary }, style.h(0).borderTWidth(1).border(dashed).borderT(primary)),
  compound({ variant: dashed, orientation: horizontal, tone: destructive }, style.h(0).borderTWidth(1).border(dashed).borderT(destructive)),
  compound({ variant: dashed, orientation: vertical, tone: default }, style.w(0).borderLWidth(1).border(dashed).borderL(separator-line)),
  compound({ variant: dashed, orientation: vertical, tone: muted }, style.w(0).borderLWidth(1).border(dashed).borderL(border-subtle)),
  compound({ variant: dashed, orientation: vertical, tone: primary }, style.w(0).borderLWidth(1).border(dashed).borderL(primary)),
  compound({ variant: dashed, orientation: vertical, tone: destructive }, style.w(0).borderLWidth(1).border(dashed).borderL(destructive)),
])

