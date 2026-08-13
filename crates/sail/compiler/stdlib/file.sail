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

  style pickerHost = {box: flow({ display: contents }), paint: group()}
  // Keep in lockstep with dropzone.sail host — same surface.
  style dropHost = {box: flow({ display: flex }).size({ w: full, min-h: 32 }).space({ pad: 6 }), paint: group().ink({ color: muted-foreground }).fill(muted).border({ color: border, width: 1, style: dashed }).radius(lg).cursor(pointer), text: type({ align: center, family: body, size: sm, weight: medium }), when: {hover: {paint: border({ color: primary }).fill({ color: primary, mix: 10 }).ink(foreground)}, stateOver: {paint: border({ color: primary }).fill({ color: primary, mix: 10 }).ink(foreground)}, disabled: {paint: cursor(not-allowed).filter({ opacity: 50 })}}}
  style file = {box: flow({ display: flex }).size({ min-h: control-h-md, w: full }).space({ pad-y: 1, pad-x: 3 }), paint: ink({ color: foreground }).fill({ color: input }).border({ color: border, width: 1 }).radius(md).elevation(xs).cursor(pointer), text: type({ size: base }), when: {hover: {paint: border({ color: border-strong })}, disabled: {paint: cursor(not-allowed).filter({ opacity: 50 })}}}
  style sizeXs = {box: size({ min-h: control-h-2xs }).space({ pad-x: 2 }), text: type({ size: xs })}
  style sizeSm = {box: size({ min-h: control-h-sm }).space({ pad-x: 2_5 }), text: type({ size: sm })}
  style sizeMd = {box: size({ min-h: control-h-md }).space({ pad-x: 3 }), text: type({ size: base })}
  style sizeLg = {box: size({ min-h: control-h-lg }).space({ pad-x: 4 }), text: type({ size: md })}
  style hint = {text: type({ size: sm })}
</scope>

<div
  use:dropzone
  style({paint: group(), match: {
      variant: {
        picker: pickerHost,
        drop: dropHost,
      },
    }, compound: [{invalid: true, variant: drop, paint: border({ color: destructive })}]})
>
  <input
    type="file"
    style(file, {match: { size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg } }, compound: [{invalid: true, paint: border({ color: destructive })}]})
  >
  <span data-dz-files style(hint)>Drop a file, or click to browse</span>
</div>
