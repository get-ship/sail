<scope>
  props: {
    kind: 'ref-bg' | 'ref-fg' | 'mix' | 'chain' = 'ref-bg'
  }
  style refBg = { size({ w: 100%, h: 24 }), radius(sm), bg({ ref: primary }) }
  style refFg = { size({ w: 100%, h: 24 }), radius(sm), type(heading), fg({ color: { ref: accent } }) }
  style mix = { size({ w: 100%, h: 24 }), radius(sm), bg({ op: mix, from: primary, onto: accent, amount: 50 }) }
  style chain = { size({ w: 100%, h: 24 }), radius(sm), pad(4), bg({ ref: muted }), fg({ ref: primary }), type(heading) }
</scope>
<column style({ match: { kind: { 'ref-bg': refBg, 'ref-fg': refFg, mix: mix, chain: chain } } })>
  <slot></slot>
</column>
