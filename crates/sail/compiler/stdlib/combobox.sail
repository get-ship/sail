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

  style host = {box: flow({ display: block }).place({ position: relative }).size({ w: full }), text: type({ family: body })}
  // Field chrome matches bare text input so combobox survives theme swaps.
  style input = {box: flow({ display: block }).size({ w: full, min-h: control-h-md }).space({ pad-y: 1_5, pad-x: 3 }), paint: ink(foreground).fill({ color: background }).border({ color: border, width: 1 }).radius(md).elevation(xs), text: type({ size: base }), when: {hover: {paint: border({ color: border-strong })}, focus: {paint: border({ style: solid, kind: outline, width: 2, color: ring, offset: 2 })}, disabled: {paint: cursor(not-allowed).filter({ opacity: 50 })}}}
  // w + minW both from --anchor-width so the list matches the field (min alone hugs content when options are short).
  style listbox = {box: place({ position: fixed, inset: auto, left: bind(anchorLeft), top: bind(anchorTop), z: popover }).space({ m: 0, pad: 1 }).size({ w: bind(anchorWidth), min-w: bind(anchorWidth), max-h: panel-max-h-sm }).overflow({ y: auto, overscroll: contain }), paint: ink({ color: combobox.foreground }).fill({ color: combobox }).border({ color: border, width: 1 }).radius(md).elevation(lg)}
</scope>

<div use:combobox style(host)>
  <input type="text" style(input)>
  <!-- manual: auto light-dismiss races pointerdown-open (flash). use:combobox dismisses. -->
  <div popover="manual" use:anchor placement="bottom-start" style(listbox)>
    <slot></slot>
  </div>
</div>
