<!--
  <slider> — native range. Single thumb by default.
  range=true: two thumbs (value + value-end). use:rangePair owns fill + order.
  Track/thumb vendor pseudos: platform/slider.css.
-->

<scope>
  props: {
    value = 0
    'value-end' = 100
    min = 0
    max = 100
    step = 1
    disabled = false
    range = false
    size: 'sm' | 'md' | 'lg' = 'md'
    name = ''
    ticks = ''
  }

  style host = {position(relative).display(block).size({ w: full })}
  style base = {size({ w: full, h: 5 }).appearance(none).bg(transparent).cursor(pointer), when: {disabled: cursor(not-allowed).filter({ opacity: 50 })}}
  style sizeSm = {size({ h: 4 })}
  style sizeMd = {size({ h: 5 })}
  style sizeLg = {size({ h: 6 })}
  style thumbOn = {display(block)}
  style thumbOff = {display(none)}
</scope>

<div use:rangePair style(host)>
  <input
    type="range"
    use:rangeFill
    style(base, { match: { size: { sm: sizeSm, md: sizeMd, lg: sizeLg } } })
  >
  <input
    type="range"
    use:rangeFill
    style(base, { match: { range: { true: thumbOn, false: thumbOff }, size: { sm: sizeSm, md: sizeMd, lg: sizeLg } } })
  >
</div>
