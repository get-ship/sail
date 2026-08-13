<!--
  <separator> — native <hr>. Style value language only.
  Rest paint = border when tone is omitted.
  Semantic paint = bound tone via presence compound only (never unconditional
  bg(tone) — empty --tone would stomp the rest line).
  Host subset: muted | primary | destructive (not full Tone).
  Orientation × dashed compounds remain (not pure group-member paint).
-->

<scope>
  props: {
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    tone: 'muted' | 'primary' | 'destructive'
    variant: 'solid' | 'dashed' = 'solid'
  }

  style base = {box: flow({ display: block }).size({ w: full, h: border-width }).space({ my: 2, pad: 0 }), paint: border({ width: 0 }).fill({ color: border })}
  style orientH = {box: flow({ display: block }).size({ w: full, h: border-width }).space({ my: 2, mx: 0 })}
  style orientV = {box: flow({ display: inline-block, self: stretch }).size({ w: border-width, h: full }).space({ mx: 2, my: 0 })}
  style solid = {box: space({ pad: 0 }), paint: fill({ color: border }).border({ width: 0 })}
  style dashed = {box: space({ pad: 0 }), paint: fill({ color: transparent }).border({ width: 0 })}
</scope>

<hr
  style(base, {match: {
      orientation: { horizontal: orientH, vertical: orientV },
      variant: {
        solid: solid,
        dashed: dashed,
      },
    }, compound: [
      {orientation: horizontal, variant: dashed, box: size({ h: 0 }), paint: border({ style: dashed, edge: b }).border({ style: dashed, edge: l }).border({ style: dashed, edge: r }).border({ style: dashed, width: 1, edge: t, color: border })},
      {orientation: vertical, variant: dashed, box: size({ w: 0 }), paint: border({ style: dashed, edge: b }).border({ style: dashed, edge: r }).border({ style: dashed, edge: t }).border({ style: dashed, width: 1, edge: l, color: border })},
      // Tone only when present — never empty bg(tone)/border*(tone) on rest.
      {tone: *, variant: solid, paint: fill({ color: tone })},
      {orientation: horizontal, tone: *, variant: dashed, paint: border({ color: tone, edge: t })},
      {orientation: vertical, tone: *, variant: dashed, paint: border({ color: tone, edge: l })},
      {tone: muted, variant: solid, paint: fill({ color: border-subtle })},
      {orientation: horizontal, tone: muted, variant: dashed, paint: border({ color: border-subtle, edge: t })},
      {orientation: vertical, tone: muted, variant: dashed, paint: border({ color: border-subtle, edge: l })},
    ]})
>
