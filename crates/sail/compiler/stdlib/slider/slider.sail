<!--
  <slider> — native range. Single thumb by default.
  range=true: two thumbs on one track (`<stack>` — overlay is the product).
  ticks: named labels in a flow row under the track (not stamps on the thumb).
  Empty tick row uses HTML `hidden` so `.sail-row { display:flex }` cannot
  win the cascade. Track/thumb vendor pseudos: platform/slider.css.
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

  style host = {size({ w: 100%, min-w: 0 })}
  style track = {position(relative).size({ w: 100%, h: 5 })}
  style sizeSm = {size({ h: 4 })}
  style sizeMd = {size({ h: 5 })}
  style sizeLg = {size({ h: 6 })}
  style base = {size({ w: 100%, h: 100% }).appearance(none).bg(transparent).cursor(pointer), when: {disabled: cursor(not-allowed).filter({ opacity: 50 })}}
  style thumbOn = {display(block)}
  style thumbOff = {display(none)}
  style tickRow = {size({ w: 100%, min-w: 0 }).pad({ t: 2 }).fg(muted-foreground).type({ size: xs, family: mono, wrap: nowrap })}
</scope>

<column gap="0" use:rangePair style(host)>
  <stack style(track, { match: { size: { sm: sizeSm, md: sizeMd, lg: sizeLg } } })>
    <input type="range" use:rangeFill style(base)>
    <input type="range" use:rangeFill style(base, { match: { range: { true: thumbOn, false: thumbOff } } })>
  </stack>
  <row nowrap justify="between" gap="1" hidden="hidden" data-slider-ticks style(tickRow)></row>
</column>
