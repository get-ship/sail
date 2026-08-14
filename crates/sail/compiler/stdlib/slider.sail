<!--
  <slider> — native range alias. use:rangeFill → --rng-pct (WebKit fill).
  Track/thumb vendor pseudos: RANGE_MAP.
-->

<scope>
  props: {
    value = 0
    min = 0
    max = 100
    step = 1
    disabled = false
    size: 'sm' | 'md' | 'lg' = 'md'
    name = ''
    /** Comma-separated labels under thumb-centers. Owned by the slider — not a docs pad hack. */
    ticks = ''
  }

  style base = {size({ w: full, h: 5 }).appearance(none).bg(transparent).cursor(pointer), when: {disabled: cursor(not-allowed).filter({ opacity: 50 })}}
  style sizeSm = {size({ h: 4 })}
  style sizeMd = {size({ h: 5 })}
  style sizeLg = {size({ h: 6 })}
</scope>

<input
  type="range"
  use:rangeFill
  style(base, { match: { size: { sm: sizeSm, md: sizeMd, lg: sizeLg } } })
>
