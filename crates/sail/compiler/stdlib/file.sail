<!--
  <file> — system file picker product. Impl: native input type=file.
  accept · multiple · name · required · size · invalid · disabled.
  Drag-drop is use:dropzone on a surface — not a second file CE.
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
</scope>

<input
  type="file"
  style(file, {
    match: { size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg } },
    compound: [{ invalid: true, paint: border(destructive) }],
  })
>
