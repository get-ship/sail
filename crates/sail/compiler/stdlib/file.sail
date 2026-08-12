<!--
  <file> — system file picker.
  variant=picker: native control (default).
  variant=drop: same dashed surface as <dropzone> — drag or click.
-->

<scope>
  props: {
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    invalid = false
    disabled = false
    name = ''
    required = false
    accept = ''
    multiple = false
    variant: 'picker' | 'drop' = 'picker'
  }

  style pickerHost = {
    box: display(contents),
    paint: group(),
  }
  // Keep in lockstep with dropzone.sail host — same surface.
  style dropHost = {
    box: display(flex).w(full).minH(32).pad(6),
    paint: group().fg(muted-foreground).bg(muted).borderWidth(1).border(dashed).border(border).radius(lg).cursor(pointer),
    text: textAlign(center).fontFamily(body).fontSize(sm).fontWeight(medium),
    when: {
      hover: { paint: border(primary).colorMix(primary, 10).fg(foreground) },
      stateOver: { paint: border(primary).colorMix(primary, 10).fg(foreground) },
      disabled: { paint: cursor(not-allowed).opacity(50) },
    },
  }
  style file = {
    box: display(flex).minH(control-h-md).padY(1).padX(3).w(full),
    paint: fg(field.foreground).bg(field).borderWidth(1).border(field.border).radius(md).shadow(xs).cursor(pointer),
    text: fontSize(base),
    when: {
      hover: { paint: border(border-strong) },
      disabled: { paint: cursor(not-allowed).opacity(50) },
    },
  }
  style sizeXs = { box: minH(control-h-2xs).padX(2), text: fontSize(xs) }
  style sizeSm = { box: minH(control-h-sm).padX(2_5), text: fontSize(sm) }
  style sizeMd = { box: minH(control-h-md).padX(3), text: fontSize(base) }
  style sizeLg = { box: minH(control-h-lg).padX(4), text: fontSize(md) }
  style hint = { text: fontSize(sm) }
</scope>

<div
  use:dropzone
  style({
    paint: group(),
    match: {
      variant: {
        picker: pickerHost,
        drop: dropHost,
      },
    },
    compound: [{ invalid: true, variant: drop, paint: border(destructive) }],
  })
>
  <input
    type="file"
    style(file, {
      match: { size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg } },
      compound: [{ invalid: true, paint: border(destructive) }],
    })
  >
  <span data-dz-files style(hint)>Drop a file, or click to browse</span>
</div>
