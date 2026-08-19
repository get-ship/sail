<!--
  <switch> — native checkbox restyled as sliding toggle. No controller.

  Design tokens (muted / border-strong / inverse):
    track rest  → muted
    border rest → border-strong
    thumb ALWAYS → inverse

  Checked track = primary (default tone). Tone paints the checked track
  via compound { tone: * } only (same presence gate as separator/avatar —
  Style product contract, not a CSS escape hatch). Thumb stays inverse.
-->

<scope>
  props: {
    checked = false
    disabled = false
    tone: 'primary' | 'info' | 'success' | 'warning' | 'danger' | 'accent' | 'muted' = 'primary'
    size: 'md' | 'sm' | 'lg' = 'md'
  }

  // Size ladder is a visible 8px width step. md/lg used to share --space-7
  // width (28) so the size attr painted as a no-op.
  style base = {position(relative).display(inline-block).margin(0).size({ w: 9, h: 5 }).appearance(none).boxSizing(border).align(middle).cursor(pointer).bg(muted).border({ color: border-strong, width: 1 }).radius(full).tapHighlight(none), animate: { transition: play(colors, fast) }}
  style sizeMd = {size({ w: 9, h: 5 })}
  style sizeSm = {size({ w: 7, h: icon-size-md })}
  style sizeLg = {size({ w: 12, h: 6 })}
  style thumbMd = {size(4)}
  style thumbSm = {size(3)}
  style thumbLg = {size(5)}
</scope>

<input
  type="checkbox"
  style(base, {shrink(0), when: {hover: border({ color: muted-foreground }), checked: {bg(primary).border({ color: selected-border }).elevation(xs), when: {hover: bg(primary).border({ color: selected-border }), before: position({ l: auto, r: px })}}, focus: border({ style: solid, kind: outline, width: 2, color: primary, offset: ring-offset }), disabled: cursor(not-allowed).bg(muted).border({ color: border }).filter({ saturate: 60 }).elevation(none), before: position({ scheme: absolute, t: 0, b: 0, l: px }).margin({ y: auto }).size(4).content(empty).bg(inverse).radius(full).elevation(sm).transition(all).duration(fast).ease(out), disabled: {when: {before: elevation(xs)}}, motion-reduce: {when: {before: transition(none)}}, forced-colors: border({ color: CanvasText }), forced-colors: {when: {checked: bg(Highlight), focus: border({ color: CanvasText, kind: outline })}}}, match: { size: { md: style(sizeMd, { when: { before: thumbMd } }), sm: style(sizeSm, { when: { before: thumbSm } }), lg: style(sizeLg, { when: { before: thumbLg } }) } }, compound: [
      // Tone present → track (and focus) only. Thumb never leaves inverse.
      {tone: *, when: {checked: bg(tone).border({ color: selected-border }).elevation(xs)}},
      {tone: *, when: {checked: {when: {hover: bg(tone).border({ color: selected-border })}}}},
      {tone: *, when: {focus: border({ color: tone, kind: outline })}},
      {tone: muted, when: {checked: bg(muted-foreground), focus: border({ color: muted-foreground, kind: outline })}},
      {tone: muted, when: {checked: {when: {hover: bg(muted-foreground)}}}},
    ]})
>
