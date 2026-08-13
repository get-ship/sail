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

  style host = {box: flow({ display: flex }).size({ w: full, min-h: 32 }).space({ pad: 6 }), paint: group().ink({ color: muted-foreground }).fill(muted).border({ color: border, width: 1, style: dashed }).radius(lg).cursor(pointer), text: type({ align: center, family: body, size: sm, weight: medium }), when: {hover: {paint: border({ color: primary }).fill({ color: primary, mix: 10 }).ink(foreground)}, stateOver: {paint: border({ color: primary }).fill({ color: primary, mix: 10 }).ink(foreground)}, disabled: {paint: cursor(not-allowed).filter({ opacity: 50 })}}}
  // Align + justify so the label sits in the middle of the zone.
  style input = {box: flow({ display: none })}
  style label = {box: flow({ display: block }), text: type({ size: sm })}
</scope>

<div
  use:dropzone
  style(host, {compound: [{invalid: true, paint: border({ color: destructive })}]})
>
  <input type="file" style(input)>
  <span data-dz-files style(label)>Drop a file, or click to browse</span>
</div>
