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

  style host = {display(block).position(relative).size({ w: 100% }).type({ family: body })}
  // Same `--input` surface as input/select/textarea — never bg(background).
  style input = {display(block).size({ min-h: control-h-md }).pad({ y: 1_5, x: 3 }).fg(foreground).bg(inset).border({ color: border, width: 1 }).radius(md).elevation(xs).type({ size: base }), when: {hover: border({ color: border-strong }), focus: border({ style: solid, kind: outline, width: 2, color: ring, offset: 2 }), disabled: cursor(not-allowed).filter({ opacity: 50 })}}
  // w + minW both from --anchor-width so the list matches the field (min alone hugs content when options are short).
  style listbox = {position({ scheme: fixed, inset: auto, l: bind(anchorLeft), t: bind(anchorTop), z: popover }).pad(1).margin(0).size({ w: bind(anchorWidth), min-w: bind(anchorWidth), max-h: overlay-max-h-sm }).overflow({ y: auto, overscroll: contain }).fg(foreground).bg(overlay).border({ color: border, width: 1 }).radius(md).elevation(lg)}
</scope>

<div use:combobox style(host)>
  <input type="text" style(input)>
  <!-- manual: auto light-dismiss races pointerdown-open (flash). use:combobox dismisses. -->
  <div popover="manual" use:anchor placement="bottom-start" style(listbox)>
    <slot></slot>
  </div>
</div>
