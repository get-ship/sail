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

  style base = {
    box: position(relative).display(inline-block).m(0).w(control-h-sm).h(icon-size-md),
    paint: appearance(none).boxSizing(border).align(middle).cursor(pointer).bg(none).bg(switch).borderWidth(1).border(switch.border).radius(full).tapHighlight(none).transition(colors).duration(fast).ease(out),
  }
  style sizeMd = { box: w(control-h-sm).h(icon-size-md) }
  style sizeSm = { box: w(control-h-xs).h(icon-size-sm) }
  style sizeLg = { box: w(7).h(icon-size-lg) }
  style thumbMd = { box: size(3) }
  style thumbSm = { box: size(2_5) }
  style thumbLg = { box: size(icon-size-sm) }
</scope>

<input
  type="checkbox"
  style(base, {
    box: shrink(0),
    when: {
      hover: { paint: border(muted-foreground) },
      checked: {
        paint: bg(primary).border(selected-border).shadow(xs),
        when: {
          // Track only — never host opacity wash (would fade switch.thumb).
          hover: { paint: bg(primary).border(selected-border) },
          // Slide the knob; paint stays switch.thumb from rest before.
          before: { box: left(auto).right(px) },
        },
      },
      focus: { paint: outline(solid).outline(2).outline(primary).outlineOffset(ring-offset) },
      disabled: { paint: cursor(not-allowed).bg(switch).border(border).saturate(60).shadow(none) },
      before: {
        box: position(absolute).top(0).bottom(0).my(auto).left(px).size(3),
        paint: content(empty).bg(switch.thumb).radius(full).shadow(sm).transition(all).duration(fast).ease(out),
      },
      disabled: { when: { before: { paint: shadow(xs) } } },
      motion-reduce: { when: { before: { paint: transition(none) } } },
      forced-colors: { paint: border(CanvasText) },
      forced-colors: { when: { checked: { paint: bg(Highlight) }, focus: { paint: outline(CanvasText) } } },
    },
    match: {
      size: {
        md: style(sizeMd, { when: { before: thumbMd } }),
        sm: style(sizeSm, { when: { before: thumbSm } }),
        lg: style(sizeLg, { when: { before: thumbLg } }),
      },
    },
    compound: [
      // Tone present → track (and focus) only. Thumb never leaves switch.thumb.
      { tone: *, when: { checked: { paint: bg(tone).border(selected-border).shadow(xs) } } },
      { tone: *, when: { checked: { when: { hover: { paint: bg(tone).border(selected-border) } } } } },
      { tone: *, when: { focus: { paint: outline(tone) } } },
      { tone: muted, when: { checked: { paint: bg(muted-foreground) }, focus: { paint: outline(muted-foreground) } } },
      { tone: muted, when: { checked: { when: { hover: { paint: bg(muted-foreground) } } } } },
    ],
  })
>
