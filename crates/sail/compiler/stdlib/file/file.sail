<!--
  <file> — system file picker. Native Browse chrome is hidden.
  variant=drop (default): dashed drop surface — drag or click.
  variant=picker: same skin, compact well.
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
    variant: 'picker' | 'drop' = 'drop'
  }

  // Compact browse well — same dropzone skin, shorter.
  style pickerHost = {display(flex).size({ w: 100%, min-h: control-h-lg }).pad({ x: 4, y: 3 }).group().fg(muted-foreground).bg(muted).border({ color: border, width: 1, style: dashed }).radius(md).cursor(pointer).type({ align: center, family: body, size: sm, weight: medium }), when: {hover: border({ color: primary }).bg({ color: primary, mix: 10 }).fg(foreground), stateOver: border({ color: primary }).bg({ color: primary, mix: 10 }).fg(foreground), disabled: cursor(not-allowed).filter({ opacity: 50 })}}
  // Dashed drop surface for variant=drop.
  style dropHost = {display(flex).size({ w: 100%, min-h: 32 }).pad(6).group().fg(muted-foreground).bg(muted).border({ color: border, width: 1, style: dashed }).radius(lg).cursor(pointer).type({ align: center, family: body, size: sm, weight: medium }), when: {hover: border({ color: primary }).bg({ color: primary, mix: 10 }).fg(foreground), stateOver: border({ color: primary }).bg({ color: primary, mix: 10 }).fg(foreground), disabled: cursor(not-allowed).filter({ opacity: 50 })}}
  style file = {display(none)}
  style sizeXs = {size({ min-h: control-h-2xs }).pad({ x: 2 }).type({ size: xs })}
  style sizeSm = {size({ min-h: control-h-sm }).pad({ x: 2_5 }).type({ size: sm })}
  style sizeMd = {size({ min-h: control-h-md }).pad({ x: 3 }).type({ size: base })}
  style sizeLg = {size({ min-h: control-h-lg }).pad({ x: 4 }).type({ size: md })}
  style hint = {type({ size: sm })}
</scope>

<div
  use:dropzone
  style({group(), match: { variant: { picker: pickerHost, drop: dropHost } }, compound: [{invalid: true, variant: drop, border({ color: destructive })}]})
>
  <input
    type="file"
    style(file, {match: { size: { xs: sizeXs, sm: sizeSm, md: sizeMd, lg: sizeLg } }, compound: [{invalid: true, border({ color: destructive })}]})
  >
  <span data-dz-files style(hint)>Drop a file or click to browse</span>
</div>
