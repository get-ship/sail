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

  const host = style.display(block).position(relative).fontFamily(body)
  const input = style.w(full)
  const listbox = style.position(fixed).inset(auto).m(0).left(bind(anchorLeft)).top(bind(anchorTop)).minW(bind(anchorWidth)).maxH(panel-max-h-sm).overflowY(auto).overscroll(contain).pad(1).fg(combobox-foreground).bg(combobox).borderWidth(1).border(border).radius(md).shadow(lg).z(popover)
</scope>

<div use:combobox sx={host}>
  <input class="input" type="text" sx={input}>
  <div
    class="listbox"
    popover
    use:anchor
    placement="bottom-start"
    sx={listbox}
  >
    <slot></slot>
  </div>
</div>
