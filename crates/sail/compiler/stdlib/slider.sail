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

  style base = {box: size({ w: full, h: 5 }), paint: appearance(none).fill({ color: transparent }).cursor(pointer), when: {disabled: {paint: cursor(not-allowed).filter({ opacity: 50 })}}}
  style sizeSm = {box: size({ h: 4 })}
  style sizeMd = {box: size({ h: 5 })}
  style sizeLg = {box: size({ h: 6 })}
</scope>

<input
  type="range"
  use:rangeFill
  style(base, {
    match: { size: { sm: sizeSm, md: sizeMd, lg: sizeLg } },
  })
>
