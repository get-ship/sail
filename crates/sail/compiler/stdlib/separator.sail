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

  const base = style({ box: display(block).w(full).h(border-width).my(2).pad(0), paint: borderWidth(0).bg(separator-line) })
  const orientH = style({ box: display(block).w(full).h(border-width).my(2).mx(0) })
  const orientV = style({ box: display(inline-block).self(stretch).w(border-width).h(full).mx(2).my(0) })
  const solid = style({ box: pad(0), paint: bg(separator-line).borderWidth(0) })
  const dashed = style({ box: pad(0), paint: bg(transparent).borderWidth(0) })
</scope>

<hr style([
  base,
  match(orientation, {
    horizontal: orientH,
    vertical: orientV,
  }),
  match(variant, {
    solid: solid,
    dashed: dashed,
  }),
  compound({ variant: solid, tone: default }, style({ paint: bg(separator-line) })),
  compound({ variant: solid, tone: muted }, style({ paint: bg(border-subtle) })),
  compound({ variant: solid, tone: primary }, style({ paint: bg(primary) })),
  compound({ variant: solid, tone: destructive }, style({ paint: bg(destructive) })),
  compound({ variant: dashed, orientation: horizontal, tone: default }, style({ box: h(0), paint: borderTWidth(1).border(dashed).borderT(separator-line) })),
  compound({ variant: dashed, orientation: horizontal, tone: muted }, style({ box: h(0), paint: borderTWidth(1).border(dashed).borderT(border-subtle) })),
  compound({ variant: dashed, orientation: horizontal, tone: primary }, style({ box: h(0), paint: borderTWidth(1).border(dashed).borderT(primary) })),
  compound({ variant: dashed, orientation: horizontal, tone: destructive }, style({ box: h(0), paint: borderTWidth(1).border(dashed).borderT(destructive) })),
  compound({ variant: dashed, orientation: vertical, tone: default }, style({ box: w(0), paint: borderLWidth(1).border(dashed).borderL(separator-line) })),
  compound({ variant: dashed, orientation: vertical, tone: muted }, style({ box: w(0), paint: borderLWidth(1).border(dashed).borderL(border-subtle) })),
  compound({ variant: dashed, orientation: vertical, tone: primary }, style({ box: w(0), paint: borderLWidth(1).border(dashed).borderL(primary) })),
  compound({ variant: dashed, orientation: vertical, tone: destructive }, style({ box: w(0), paint: borderLWidth(1).border(dashed).borderL(destructive) })),
])>
