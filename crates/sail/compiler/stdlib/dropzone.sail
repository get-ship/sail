<!--
  <dropzone> — dashed drop surface that accepts files.
  Click to browse or drag onto the host. Native input is inside for form
  participation; chrome is the surface, not the picker.
-->

<scope>
  props: {
    accept = ''
    multiple = false
    disabled = false
    invalid = false
    name = ''
    required = false
  }

  style host = {display(flex).size({ w: full, min-h: 32 }).pad(6).group().fg(muted-foreground).bg(muted).border({ color: border, width: 1, style: dashed }).radius(lg).cursor(pointer).type({ align: center, family: body, size: sm, weight: medium }), when: {hover: border({ color: primary }).bg({ color: primary, mix: 10 }).fg(foreground), stateOver: border({ color: primary }).bg({ color: primary, mix: 10 }).fg(foreground), disabled: cursor(not-allowed).filter({ opacity: 50 })}}
  // Align + justify so the label sits in the middle of the zone.
  style input = {display(none)}
  style label = {display(block).type({ size: sm })}
</scope>

<div
  use:dropzone
  style(host, {compound: [{invalid: true, border({ color: destructive })}]})
>
  <input type="file" style(input)>
  <span data-dz-files style(label)>Drop a file, or click to browse</span>
</div>
