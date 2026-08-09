<!--
  <combobox placeholder="…"> — L1 filtering picker. Text input filters a listbox
  and commits one option. Listbox is a native [popover] via use:anchor; use:combobox
  owns ARIA + keyboard. Commit fires onSelect { value, label } and reflects value.
  Element-mode structure is static: placeholder rides the host attr into use:combobox.
-->

<scope>
  props: {
    placeholder = ''
    value = ''
    onSelect: (detail: { value: string; label: string }) => void
  }

  style host = { box: display(block).position(relative).w(full), text: fontFamily(body) }
  // Field chrome matches bare text input so combobox survives theme swaps.
  style input = { box: display(block).w(full).minH(control-h-md).padY(1_5).padX(3), paint: fg(foreground).bg(background).borderWidth(1).border(border).radius(md).shadow(xs), text: fontSize(base), when: { hover: { paint: border(border-strong) }, focus: { paint: outline(solid).outline(2).outline(ring).outlineOffset(2) }, disabled: { paint: cursor(not-allowed).opacity(50) } } }
  // w + minW both from --anchor-width so the list matches the field (min alone hugs content when options are short).
  style listbox = { box: position(fixed).inset(auto).m(0).left(bind(anchorLeft)).top(bind(anchorTop)).w(bind(anchorWidth)).minW(bind(anchorWidth)).maxH(panel-max-h-sm).overflowY(auto).pad(1), paint: overscroll(contain).fg(combobox.foreground).bg(combobox).borderWidth(1).border(border).radius(md).shadow(lg).z(popover) }
</scope>

<div use:combobox style(host)>
  <input type="text" style(input)>
  <div popover use:anchor placement="bottom-start" style(listbox)>
    <slot></slot>
  </div>
</div>
