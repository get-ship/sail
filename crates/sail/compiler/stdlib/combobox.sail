<!--
  <combobox placeholder="…"> — L1 filtering picker (element mode). A text input
  that filters a listbox and commits one option.

  The platform ALMOST has this: `<input list>` + `<datalist>` is native typeahead
  with zero JS — but the UA draws that popup and it cannot be styled or themed.
  That gap, and only that gap, is why this is a primitive rather than a native+skin.

  The listbox is a native [popover] anchored by use:anchor — the same two
  mechanisms <popover>/<dropdown>/<tooltip> already run on, not a third. use:combobox
  supplies everything the ARIA pattern demands (ids, popovertarget, aria-expanded /
  -controls / -activedescendant, filter-as-you-type, Up/Down/Home/End, Enter to
  commit, Escape to close) so the author writes the tag and the options and nothing
  else. Commit emits `sail-combobox-select` with { value, label } and reflects to
  the element's `value`.

  Element mode is STATIC structure, so `placeholder` is not bound in the template:
  it rides through to the lowered root as an attribute and use:combobox forwards
  it to the inner input — the same way use:anchor reads `placement`.

  Paint (Style product):
    host    — block relative + typeface
    input   — full width
    listbox — component.combobox tokens + anchor geometry + scroll chrome
  POPOVER_SKIN [role=listbox] option anatomy (author children).
-->

<scope>
  props: {
    placeholder = ''
    value = ''
  }

  const host = style.display(block).position(relative).fontFamily(body).w(full)
  // Same field chrome as bare <input type=text> so combobox survives theme swaps.
  const input = style.display(block).w(full).minH(control-h-md).padY(1_5).padX(3).fontSize(base).fg(foreground).bg(background).borderWidth(1).border(border).radius(md).shadow(xs).when({
    hover: style.border(border-strong),
    focus: style.outline(solid).outline(2).outline(ring).outlineOffset(2),
    disabled: style.cursor(not-allowed).opacity(50),
  })
  const listbox = style.position(fixed).inset(auto).m(0).left(bind(anchorLeft)).top(bind(anchorTop)).minW(bind(anchorWidth)).maxH(panel-max-h-sm).overflowY(auto).overscroll(contain).pad(1).fg(combobox-foreground).bg(combobox).borderWidth(1).border(border).radius(md).shadow(lg).z(popover)
</scope>

<div use:combobox>
  style(host)

  <input type="text">
    style(input)

  <div
    popover
    use:anchor
    placement="bottom-start"

  >
    style(listbox)

    <slot></slot>
  </div>
</div>
