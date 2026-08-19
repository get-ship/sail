<!--
  <combobox placeholder="…"> — L1 filtering picker. Text input filters a listbox
  and commits one option. Listbox is a native [popover] via CSS anchor; the
  host controller owns ARIA + keyboard. Commit fires onSelect { value, label } and reflects value.
  Element-mode structure is static: placeholder rides the host attr into the controller.
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
  style listbox = {position({ scheme: absolute, inset: auto, anchor: auto, area: bottom, z: popover }).pad(1).margin(0).size({ w: 100%, min-w: 100%, max-h: panel-max-h-sm }).overflow({ y: auto, overscroll: contain }).fg(foreground).bg(overlay).border({ color: border, width: 1 }).radius(md).elevation(lg)}
</scope>

<controller>
import { combobox } from '@ship/sail-runtime/actions'
export default controller((_ctx, host) => {
  mount((el) => {
    const result = combobox((el ?? host) as HTMLElement)
    return () => {
      result?.destroy?.()
    }
  })
  return {}
})
</controller>
<div style(host)>
  <input type="text" style(input)>
  <!-- manual: auto light-dismiss races pointerdown-open (flash). The controller dismisses. -->
  <div popover="manual" placement="bottom" style(listbox)>
    <slot></slot>
  </div>
</div>
