<!--
  <checkbox> — boolean check product. Impl: native input type=checkbox.
  Zero JS for toggle; check mark is platform/checkbox.css. Face tokens align with <radio>.
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
  // Fill is platform/checkbox.css — not Style bg() atomics (unlayered
  // component scopes re-emit .s-bgbackground and clobber checked primary).
  // Border ink is --border (field hairline), not --foreground — FG at 1–2px
  // looks twice as thick as a real checkbox. No shadow(xs): the soft outer
  // blur double-paints the edge. boxSizing(border) so size tokens are outer.
  style check = {display(inline-block).shrink(0).pad(0).margin(0).size({ min-h: 0, w: icon-size-md, h: icon-size-md }).appearance(none).align(middle).boxSizing(border).border({ color: border, width: 1, style: solid }).radius(sm).cursor(pointer).type({ size: sm }), when: {disabled: cursor(not-allowed).filter({ opacity: 50 })}}
  style checkSm = {size({ w: icon-size-sm, h: icon-size-sm })}
  style checkMd = {size({ w: icon-size-md, h: icon-size-md })}
  style checkLg = {size({ w: icon-size-lg, h: icon-size-lg })}
</scope>

<input
  type="checkbox"
  style(check, {shrink(0), match: { size: { xs: checkSm, sm: checkSm, md: checkMd, lg: checkLg } }, compound: [{invalid: true, border({ color: destructive })}]})
>
