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

    <combobox placeholder="Pick a fruit">
      <option value="apple">Apple</option>
      <option value="banana">Banana</option>
    </combobox>

  Paint (one path):
    host sx=         — block relative + typeface
    child sx=        — input full width; listbox bg/fg from component.combobox
                       tokens (cssNames combobox, combobox-foreground) + anchor
    POPOVER_SKIN     — [role=listbox] option anatomy (author children; same
                       shape as [role=menu] menuitem chrome)
  No residual combobox matrix.
-->

<scope>
  props: {
    placeholder = ''
    value = ''
  }
</scope>

<div use:combobox sx="block relative font-family: body">
  <input class="input" type="text" sx="w: full">
  <div
    class="listbox"
    popover
    use:anchor
    placement="bottom-start"
    sx="fixed inset: auto m: 0 left: (--anchor-left) top: (--anchor-top) min-w: (--anchor-width) max-h: panel-max-h-sm overflow-y: auto overscroll: contain p: 1 color: combobox-foreground bg: combobox border border: border rounded: md shadow: lg z: (--z-popover)"
  >
    <slot></slot>
  </div>
</div>
