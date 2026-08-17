<!--
  <separator> — native <hr>. Style value language only.
  tone defaults to muted — a bare <separator> is a finished divider.
  Semantic paint = bound tone via presence compound only (never unconditional
  bg(tone) — empty --tone would stomp the rest line).
  Host subset: muted | primary | danger (not full Tone).
  Orientation × dashed compounds remain (not pure group-member paint).
  label = accessible name (aria-label). Visible labeled chrome (two lines +
  text) is a language gap: native hr is void, a slot would drop author-void.
-->

<scope>
  props: {
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    tone: 'muted' | 'primary' | 'danger' = 'muted'
    variant: 'solid' | 'dashed' = 'solid'
    label = ''
  }

  style base = {display(block).size({ w: 100%, h: border-width }).pad(0).margin({ y: 2 }).border({ width: 0 }).bg(border)}
  style orientH = {display(block).size({ w: 100%, h: border-width }).margin({ y: 2, x: 0 })}
  style orientV = {display(inline-block).self(stretch).size({ w: border-width, h: 100% }).margin({ x: 2, y: 0 })}
  style solid = {pad(0).bg(border).border({ width: 0 })}
  style dashed = {pad(0).bg(transparent).border({ width: 0 })}
</scope>

<hr
  style(base, {match: { orientation: { horizontal: orientH, vertical: orientV }, variant: { solid: solid, dashed: dashed } }, compound: [
      {orientation: horizontal, variant: dashed, size({ h: 0 }).border({ style: dashed, edge: b }).border({ style: dashed, edge: l }).border({ style: dashed, edge: r }).border({ style: dashed, width: 1, edge: t, color: border })},
      {orientation: vertical, variant: dashed, size({ w: 0 }).border({ style: dashed, edge: b }).border({ style: dashed, edge: r }).border({ style: dashed, edge: t }).border({ style: dashed, width: 1, edge: l, color: border })},
      // Tone only when present — never empty bg(tone)/border*(tone) on rest.
      {tone: *, variant: solid, bg(tone)},
      {orientation: horizontal, tone: *, variant: dashed, border({ color: tone, edge: t })},
      {orientation: vertical, tone: *, variant: dashed, border({ color: tone, edge: l })},
      // Muted spends --border (the line), not --border-subtle. Themes that
      // zero subtle (halo) would make a divider disappear.
      {tone: muted, variant: solid, bg(border)},
      {orientation: horizontal, tone: muted, variant: dashed, border({ color: border, edge: t })},
      {orientation: vertical, tone: muted, variant: dashed, border({ color: border, edge: l })},
    ]})
>
