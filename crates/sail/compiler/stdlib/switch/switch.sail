<!--
  <switch> — native checkbox restyled as sliding toggle.

  Design tokens (muted / border-strong / knob):
    track rest  → switch
    border rest → border-strong
    thumb ALWAYS → knob  (light contrast knob; themes override this token —
                                  never primary-foreground / tone.foreground, which
                                  collapse into the track when those resolve dark
                                  or when --tone is unset)

  Checked track = primary (default tone). Tone paints the checked track
  via compound { tone: * } only (same presence gate as separator/avatar —
  Style product contract, not a CSS escape hatch). Thumb stays knob
  in every state so track vs knob contrast is owned by the token model.

  Look: this tag owns its Style. Slots, not :part.
  State: this file's <controller> (checked / disabled as signals). No DOM.
  Listen: native <input type=checkbox> (click/Space). No use:hotkeys. No use:switch.
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

<controller>
export default controller((init) => {
  function read(v) {
    return typeof v === 'function' ? v() : v
  }
  function isOn(v) {
    const x = read(v)
    return x === true || x === '' || x === 'true'
  }
  const checked = derived(() => isOn(init.checked))
  const disabled = derived(() => isOn(init.disabled))
  return { checked, disabled }
})
</controller>

<input
  type="checkbox"
  checked={checked()}
  disabled={disabled()}
  size={size()}
  tone={tone()}
  style(base, {shrink(0), when: {hover: border({ color: muted-foreground }), checked: {bg(primary).border({ color: selected-border }).elevation(xs), when: {hover: bg(primary).border({ color: selected-border }), before: position({ l: auto, r: px })}}, focus: border({ style: solid, kind: outline, width: 2, color: primary, offset: ring-offset }), disabled: cursor(not-allowed).bg(muted).border({ color: border }).filter({ saturate: 60 }).elevation(none), before: position({ scheme: absolute, t: 0, b: 0, l: px }).margin({ y: auto }).size(4).content(empty).bg(knob).radius(full).elevation(sm).transition(all).duration(fast).ease(out), disabled: {when: {before: elevation(xs)}}, motion-reduce: {when: {before: transition(none)}}, forced-colors: border({ color: CanvasText }), forced-colors: {when: {checked: bg(Highlight), focus: border({ color: CanvasText, kind: outline })}}}, match: { size: { md: style(sizeMd, { when: { before: thumbMd } }), sm: style(sizeSm, { when: { before: thumbSm } }), lg: style(sizeLg, { when: { before: thumbLg } }) } }, compound: [
      // Tone present → track (and focus) only. Thumb never leaves knob.
      {tone: *, when: {checked: bg(tone).border({ color: selected-border }).elevation(xs)}},
      {tone: *, when: {checked: {when: {hover: bg(tone).border({ color: selected-border })}}}},
      {tone: *, when: {focus: border({ color: tone, kind: outline })}},
      {tone: muted, when: {checked: bg(muted-foreground), focus: border({ color: muted-foreground, kind: outline })}},
      {tone: muted, when: {checked: {when: {hover: bg(muted-foreground)}}}},
    ]})
>
