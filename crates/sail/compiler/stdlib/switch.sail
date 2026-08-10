<!--
  <switch> — native checkbox restyled as sliding toggle. No controller.
  Paint (Style product): track + ::before thumb; size via match; tone via bound paths.
  Rest track/border/thumb from component.switch tokens
  (cssNames switch, switch.border, switch.thumb). Checked rest = primary.
  Semantic checked/focus = bound tone ONLY when tone is present
  (never unconditional bg(tone)/bg(tone.foreground) — empty --tone invalidates
  at computed-value time to transparent and the thumb disappears; same trap as
  separator/avatar). Hover paints track color only — never host opacity wash
  (that equalizes track+thumb and makes checked look “all dark”).
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
      // Track hover only — never global opacity wash (kills thumb contrast).
      hover: { paint: border(muted-foreground) },
      checked: {
        paint: bg(primary).border(selected-border).shadow(xs),
        when: {
          // Keep selected identity under pointer — track color, not opacity.
          hover: { paint: bg(primary).border(selected-border) },
          before: { box: left(auto).right(px), paint: bg(primary-foreground) },
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
      // Tone present → bridge vars are set. Never emit empty var(--tone) on rest.
      { tone: *, when: { checked: { paint: bg(tone).border(selected-border).shadow(xs) } } },
      { tone: *, when: { checked: { when: { before: { paint: bg(tone.foreground) }, hover: { paint: bg(tone).border(selected-border) } } } } },
      { tone: *, when: { focus: { paint: outline(tone) } } },
      { tone: muted, when: { checked: { paint: bg(muted-foreground) }, focus: { paint: outline(muted-foreground) } } },
      { tone: muted, when: { checked: { when: { before: { paint: bg(background) }, hover: { paint: bg(muted-foreground) } } } } },
    ],
  })
>
