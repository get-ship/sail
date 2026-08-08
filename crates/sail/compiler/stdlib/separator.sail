<!--
  <separator> — native <hr>. Style value language only.
  Rest paint = separator.line when tone is omitted.
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

  const base = style({ box: display(block).w(full).h(border-width).my(2).pad(0), paint: borderWidth(0).bg(separator.line) })
  const orientH = style({ box: display(block).w(full).h(border-width).my(2).mx(0) })
  const orientV = style({ box: display(inline-block).self(stretch).w(border-width).h(full).mx(2).my(0) })
  const solid = style({ box: pad(0), paint: bg(separator.line).borderWidth(0) })
  const dashed = style({ box: pad(0), paint: bg(transparent).borderWidth(0) })
</scope>

<hr style(base, { match: { orientation: { horizontal: orientH, vertical: orientV }, variant: { solid: style(solid, { paint: bg(tone) }), dashed: dashed } }, compound: [ { orientation: horizontal, variant: dashed, box: h(0), paint: borderTWidth(1).border(dashed).borderT(separator.line) }, { orientation: vertical, variant: dashed, box: w(0), paint: borderLWidth(1).border(dashed).borderL(separator.line) }, { orientation: horizontal, variant: dashed, paint: borderT(tone) }, { orientation: vertical, variant: dashed, paint: borderL(tone) }, { tone: muted, variant: solid, paint: bg(border-subtle) }, { orientation: horizontal, tone: muted, variant: dashed, paint: borderT(border-subtle) }, { orientation: vertical, tone: muted, variant: dashed, paint: borderL(border-subtle) } ] })>
