<!--
  <chip> — filter/selection pill.
  tone defaults to primary — a bare <chip> is a finished control.
  Explicit tone always paints.

  Host is a text-bearing <span> (text channel + Style match; a host
  <button> cannot wrap the inner label/remove buttons). Size/variant/
  selected attrs and paint classes must share one node or match CSS never
  fires. Leading + label + remove pack is the inner <row cluster> — not
  Style gap(), not host CSS. Do not wrap the host in a layout tag.

  State: this file's <controller> (selected / removable / disabled as
  signals, onRemove, keys). Listen: use:hotkeys={keys} on the host.
  Click is the platform <button>. Selected look is Style on this host.
-->

<scope>
  props: {
    tone: 'primary' | 'info' | 'success' | 'warning' | 'danger' | 'accent' | 'muted' = 'primary'
    variant: 'solid' | 'soft' | 'outline' | 'ghost' = 'solid'
    size: 'md' | 'sm' | 'lg' = 'md'
    selected = false
    removable = false
    disabled = false
    onRemove: () => void
  }

  // Rest fill is muted; tone×variant compounds paint the bound member.
  style base = {display(inline-flex).size({ min-h: control-h-2xs }).pad({ x: 2, y: px }).group().appearance(none).cursor(pointer).select(none).tapHighlight(none).radius(full).border({ color: border, width: 1, style: solid }).bg(muted).fg(muted-foreground).type({ family: body, weight: medium, leading: none, wrap: nowrap, size: xs }), animate: { transition: play(colors, fast) }}
  style sizeMd = {size({ min-h: control-h-2xs }).pad({ x: 2_5 }).type({ size: sm })}
  style sizeSm = {size({ min-h: control-h-2xs }).pad({ x: 2 }).type({ size: xs })}
  style sizeLg = {size({ min-h: control-h-xs }).pad({ x: 3 }).type({ size: sm })}
  // Glyph, not a button: no fill invert, no control chrome. Opacity-only hover.
  style remove = {display(none).size({ w: icon-size-3xs, h: icon-size-3xs, min-w: 0, min-h: 0 }).pad(0).border({ width: 0 }).bg(transparent).fg(currentColor).radius(none).cursor(pointer).filter({ opacity: 65 }), animate: { transition: play(colors, fast) }}
  style leading = {display(contents)}
  style xmark = {display(block).size({ w: 100%, h: 100% })}
  style labelBtn = {appearance(none).bg(transparent).border({ width: 0 }).pad(0).fg(currentColor).cursor(pointer)}
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
  function onRemove() {
    if (isOn(init.disabled)) return
    const cb = init.onRemove
    if (typeof cb === 'function') cb()
  }
  const selected = derived(() => isOn(init.selected))
  const removable = derived(() => isOn(init.removable))
  const disabled = derived(() => isOn(init.disabled))
  const keys = isOn(init.removable)
    ? { Backspace: () => onRemove(), Delete: () => onRemove() }
    : {}
  return { onRemove, keys, selected, removable, disabled }
})
</controller>

<span
  use:hotkeys={keys}
  selected={selected()}
  removable={removable()}
  disabled={disabled()}
  style(base, {when: {pressed: bg(primary).fg(primary-foreground).border({ color: primary }).elevation(xs).type({ weight: semibold }), disabled: cursor(not-allowed).bg(muted).border({ color: border-subtle }).fg(muted-foreground).filter({ saturate: 65 }).elevation(none), focusWithin: border({ style: solid, kind: outline, width: 2, color: ring, offset: 2 })}, match: {size: { md: sizeMd, sm: sizeSm, lg: sizeLg }, variant: {solid: {bg(muted).fg(muted-foreground).border({ color: border }), when: {hover: filter({ opacity: 90 })}}, soft: {bg(muted).fg(muted-foreground).border({ color: border }), when: {hover: filter({ opacity: 90 })}}, outline: {bg(transparent).fg(muted-foreground).border({ color: border }), when: {hover: bg(hover-bg)}}, ghost: {bg(transparent).border({ color: transparent }).fg(muted-foreground), when: {hover: bg(hover-bg)}}}}, compound: [
      // Tone + solid
      {tone: primary, variant: solid, bg(primary).fg(primary-foreground).border({ color: primary })},
      {tone: info, variant: solid, bg(info).fg(info-foreground).border({ color: info })},
      {tone: success, variant: solid, bg(success).fg(success-foreground).border({ color: success })},
      {tone: warning, variant: solid, bg(warning).fg(warning-foreground).border({ color: warning })},
      {tone: danger, variant: solid, bg(danger).fg(danger-foreground).border({ color: danger })},
      {tone: accent, variant: solid, bg(accent).fg(accent-foreground).border({ color: accent })},
      {tone: muted, variant: solid, bg(muted).fg(muted-foreground).border({ color: border })},
      // Tone + soft (wash)
      {tone: primary, variant: soft, bg({ color: primary, mix: 15 }).border({ color: primary }).fg(primary), when: {hover: bg({ color: primary, mix: 25 })}},
      {tone: info, variant: soft, bg({ color: info, mix: 15 }).border({ color: info }).fg(info), when: {hover: bg({ color: info, mix: 25 })}},
      {tone: success, variant: soft, bg({ color: success, mix: 15 }).border({ color: success }).fg(success), when: {hover: bg({ color: success, mix: 25 })}},
      {tone: warning, variant: soft, bg({ color: warning, mix: 15 }).border({ color: warning }).fg(warning), when: {hover: bg({ color: warning, mix: 25 })}},
      {tone: danger, variant: soft, bg({ color: danger, mix: 15 }).border({ color: danger }).fg(danger), when: {hover: bg({ color: danger, mix: 25 })}},
      {tone: accent, variant: soft, bg({ color: accent, mix: 15 }).border({ color: accent }).fg(accent), when: {hover: bg({ color: accent, mix: 25 })}},
      {tone: muted, variant: soft, bg({ color: muted, mix: 15 }).border({ color: border }).fg(muted-foreground), when: {hover: bg({ color: muted, mix: 25 })}},
      // Tone + outline / ghost
      {tone: primary, variant: outline, bg(transparent).fg(primary).border({ color: primary })},
      {tone: info, variant: outline, bg(transparent).fg(info).border({ color: info })},
      {tone: success, variant: outline, bg(transparent).fg(success).border({ color: success })},
      {tone: warning, variant: outline, bg(transparent).fg(warning).border({ color: warning })},
      {tone: danger, variant: outline, bg(transparent).fg(danger).border({ color: danger })},
      {tone: accent, variant: outline, bg(transparent).fg(accent).border({ color: accent })},
      {tone: muted, variant: outline, fg(muted-foreground).border({ color: border })},
      {tone: primary, variant: ghost, bg(transparent).border({ color: transparent }).fg(primary)},
      {tone: info, variant: ghost, bg(transparent).border({ color: transparent }).fg(info)},
      {tone: success, variant: ghost, bg(transparent).border({ color: transparent }).fg(success)},
      {tone: warning, variant: ghost, bg(transparent).border({ color: transparent }).fg(warning)},
      {tone: danger, variant: ghost, bg(transparent).border({ color: transparent }).fg(danger)},
      {tone: accent, variant: ghost, bg(transparent).border({ color: transparent }).fg(accent)},
      {tone: muted, variant: ghost, fg(muted-foreground)},
      // Selected beats tone×variant (same/higher attr count so it can win cascade).
      {selected: true, bg(primary).fg(primary-foreground).border({ color: primary }).elevation(xs)},
      {selected: true, variant: solid, bg(primary).fg(primary-foreground).border({ color: primary }).elevation(xs)},
      {selected: true, variant: soft, bg(primary).fg(primary-foreground).border({ color: primary }).elevation(xs)},
      {selected: true, variant: outline, bg(primary).fg(primary-foreground).border({ color: primary }).elevation(xs)},
      {selected: true, variant: ghost, bg(primary).fg(primary-foreground).border({ color: primary }).elevation(xs)},
      {selected: true, when: {hover: filter({ opacity: 90 })}},
    ]})
>
  <row cluster gap="1">
    <span style(leading)>
      <slot name="leading"></slot>
    </span>
    <button type="button" anatomy pressed={selected()} disabled={disabled()} style(labelBtn)>
      <slot></slot>
    </button>
    <button
      type="button"
      anatomy
      label="Remove"
      tabindex="-1"
      disabled={disabled()}
      on:click|stopPropagation={onRemove}
      style(remove, {shrink(0).self(center), match: {removable: {true: display(inline-flex), false: display(none)}}, when: {hover: filter({ opacity: 100 }), focus: border({ style: solid, kind: outline, width: 2, color: ring, offset: 2 })}})
    >
      <svg viewBox="0 0 10 10" stroke-width="1.25" stroke-linecap="round" stroke="currentColor" fill="none" style(xmark)>
        <path d="M2 2 L8 8 M8 2 L2 8"></path>
      </svg>
    </button>
  </row>
</span>
