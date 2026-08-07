<!--
  <separator> — native <hr>. Style value language only.
  Rest paint = separator-line when tone is omitted.
  Semantic paint = bound tone. Authors omit tone for rest (never tone="default").
  Host subset: muted | primary | destructive (product never used full Tone).
  Orientation × dashed compounds remain (not pure group-member paint).
-->

<scope>
  props: {
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    tone: 'muted' | 'primary' | 'destructive'
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
  // Solid brand via bound tone (omit tone → separator-line rest).
  match(variant, {
    solid: style({ paint: bg(tone) }),
  }),
  // Dashed rest geometry + separator-line color (orientation specials).
  compound({ variant: dashed, orientation: horizontal }, style({ box: h(0), paint: borderTWidth(1).border(dashed).borderT(separator-line) })),
  compound({ variant: dashed, orientation: vertical }, style({ box: w(0), paint: borderLWidth(1).border(dashed).borderL(separator-line) })),
  // Dashed brand via bound tone.
  compound({ variant: dashed, orientation: horizontal }, style({ paint: borderT(tone) })),
  compound({ variant: dashed, orientation: vertical }, style({ paint: borderL(tone) })),
  // Muted specials use border-subtle (not pure muted group fill).
  compound({ variant: solid, tone: muted }, style({ paint: bg(border-subtle) })),
  compound({ variant: dashed, orientation: horizontal, tone: muted }, style({ paint: borderT(border-subtle) })),
  compound({ variant: dashed, orientation: vertical, tone: muted }, style({ paint: borderL(border-subtle) })),
])>
