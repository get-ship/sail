<!--
  <checkbox> — boolean check product. Impl: native input type=checkbox.
  Zero JS for toggle; checkmark via CHECKBOX_MAP. Face tokens align with <radio>.
  size · invalid · disabled · name · required · value. bind:checked for state.

  Face chrome aims at native weight: 1px soft border, no drop shadow, border-box.
  (Earlier: 2px + foreground ink + shadow-xs read as a heavy “scream” square.)
-->

<scope>
  props: {
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    invalid = false
    disabled = false
    name = ''
    required = false
    value = ''
  }

  // Explicit border(solid): appearance:none leaves border-style:none in some UAs.
  // Fill is CHECKBOX_MAP (host-specific) — not Style bg() atomics (unlayered
  // component scopes re-emit .s-bgbackground and clobber checked primary).
  // Border ink is --border (field hairline), not --foreground — FG at 1–2px
  // looks twice as thick as a real checkbox. No shadow(xs): the soft outer
  // blur double-paints the edge. boxSizing(border) so size tokens are outer.
  style check = {
    box: display(inline-block).m(0).pad(0).minH(0).w(icon-size-md).h(icon-size-md).shrink(0),
    paint: appearance(none).align(middle).boxSizing(border).borderWidth(1).border(solid).border(border).radius(sm).cursor(pointer),
    text: fontSize(sm),
    when: {
      disabled: { paint: cursor(not-allowed).opacity(50) },
    },
  }
  style checkSm = { box: w(icon-size-sm).h(icon-size-sm) }
  style checkMd = { box: w(icon-size-md).h(icon-size-md) }
  style checkLg = { box: w(icon-size-lg).h(icon-size-lg) }
</scope>

<input
  type="checkbox"
  style(check, {
    box: shrink(0),
    match: { size: { xs: checkSm, sm: checkSm, md: checkMd, lg: checkLg } },
    compound: [{ invalid: true, paint: border(destructive) }],
  })
>
