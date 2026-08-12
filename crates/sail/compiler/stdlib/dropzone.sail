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

  style host = {
    box: display(flex).w(full).minH(32).pad(6),
    paint: group().fg(muted-foreground).bg(muted).borderWidth(1).border(dashed).border(border).radius(lg).cursor(pointer),
    text: textAlign(center).fontFamily(body).fontSize(sm).fontWeight(medium),
    when: {
      hover: { paint: border(primary).colorMix(primary, 10).fg(foreground) },
      stateOver: { paint: border(primary).colorMix(primary, 10).fg(foreground) },
      disabled: { paint: cursor(not-allowed).opacity(50) },
    },
  }
  // Align + justify so the label sits in the middle of the zone.
  style input = { box: display(none) }
  style label = { box: display(block), text: fontSize(sm) }
</scope>

<div
  use:dropzone
  style(host, {
    compound: [{ invalid: true, paint: border(destructive) }],
  })
>
  <input type="file" style(input)>
  <span data-dz-files style(label)>Drop a file, or click to browse</span>
</div>
