<!--
  <switch> — native checkbox restyled as sliding toggle. No controller.

  Design tokens (component.switch.* → cssNames switch / switch.border / switch.thumb):
    track rest  → switch
    border rest → switch.border
    thumb ALWAYS → switch.thumb  (light contrast knob; themes override this token —
                                  never primary-foreground / tone.foreground, which
                                  collapse into the track when those resolve dark
                                  or when --tone is unset)

  Checked track = primary (rest brand). Tone present → track uses bound tone
  via compound { tone: * } only (same presence gate as separator/avatar —
  Style product contract, not a CSS escape hatch). Thumb stays switch.thumb
  in every state so track vs knob contrast is owned by the token model.
-->

<scope>
  props: {
    checked = false
    disabled = false
    tone: 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted'
    size: 'md' | 'sm' | 'lg' = 'md'
  }

  style base = {box: place({ position: relative }).flow({ display: inline-block }).space({ m: 0 }).size({ w: control-h-sm, h: icon-size-md }), paint: appearance(none).boxSizing(border).align(middle).cursor(pointer).fill({ color: switch }).border({ color: switch.border, width: 1 }).radius(full).motion({ property: colors, duration: fast, ease: out }).tapHighlight(none)}
  style sizeMd = {box: size({ w: control-h-sm, h: icon-size-md })}
  style sizeSm = {box: size({ w: control-h-xs, h: icon-size-sm })}
  style sizeLg = {box: size({ w: 7, h: icon-size-lg })}
  style thumbMd = {box: size(3)}
  style thumbSm = {box: size(2_5)}
  style thumbLg = {box: size(icon-size-sm)}
</scope>

<input
  type="checkbox"
  style(base, {box: flow({ shrink: 0 }), when: {hover: {paint: border({ color: muted-foreground })}, checked: {paint: fill({ color: primary }).border({ color: selected-border }).elevation(xs), when: {// Track only — never host opacity wash (would fade switch.thumb).
          hover: {paint: fill({ color: primary }).border({ color: selected-border })}, // Slide the knob; paint stays switch.thumb from rest before.
          before: {box: place({ left: auto, right: px })}}}, focus: {paint: border({ style: solid, kind: outline, width: 2, color: primary, offset: ring-offset })}, disabled: {paint: cursor(not-allowed).fill({ color: switch }).border({ color: border }).filter({ saturate: 60 }).elevation(none)}, before: {box: place({ position: absolute, top: 0, bottom: 0, left: px }).space({ my: auto }).size(3), paint: content(empty).fill({ color: switch.thumb }).radius(full).elevation(sm).motion({ property: all, duration: fast, ease: out })}, disabled: {when: {before: {paint: elevation(xs)}}}, motion-reduce: {when: {before: {paint: motion({ property: none })}}}, forced-colors: {paint: border({ color: CanvasText })}, forced-colors: {when: {checked: {paint: fill({ color: Highlight })}, focus: {paint: border({ color: CanvasText, kind: outline })}}}}, match: {
      size: {
        md: style(sizeMd, { when: { before: thumbMd } }),
        sm: style(sizeSm, { when: { before: thumbSm } }),
        lg: style(sizeLg, { when: { before: thumbLg } }),
      },
    }, compound: [
      // Tone present → track (and focus) only. Thumb never leaves switch.thumb.
      {tone: *, when: {checked: {paint: fill({ color: tone }).border({ color: selected-border }).elevation(xs)}}},
      {tone: *, when: {checked: {when: {hover: {paint: fill({ color: tone }).border({ color: selected-border })}}}}},
      {tone: *, when: {focus: {paint: border({ color: tone, kind: outline })}}},
      {tone: muted, when: {checked: {paint: fill({ color: muted-foreground })}, focus: {paint: border({ color: muted-foreground, kind: outline })}}},
      {tone: muted, when: {checked: {when: {hover: {paint: fill({ color: muted-foreground })}}}}},
    ]})
>
