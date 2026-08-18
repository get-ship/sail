<!--
  Look: a painted box whose face is a token via {ref}, and a mix of two tokens via {op: mix}.
  Spelling: bg({ ref: primary }) · bg({ op: mix, from: primary, onto: accent, amount: 50 })
            Theme form, opened in Style. Not a second spelling.
  Owner: language (structured {ref} / {op})
  Default: no paint
  Trap: fixtures/structured-unknown.bad.sail — Style error: structured value: unknown op `blur` — expected {op: mix}
        fixtures/structured-dangling.bad.sail — Style error: structured value: dangling ref `{no-such}` — target identity does not exist
        Quoted below. Not compiled on this page.

        <column style({ bg({ op: blur, from: primary, onto: accent }) })>x</column>
        <column style({ bg({ ref: no-such }) })>x</column>
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style refBg = { size({ w: 16, h: 8 }), radius(sm), bg({ ref: primary }) }
  style refFg = { size({ w: 16, h: 8 }), radius(sm), type(label), fg({ color: { ref: accent } }) }
  style mix = { size({ w: 16, h: 8 }), radius(sm), bg({ op: mix, from: primary, onto: accent, amount: 50 }) }
  style chain = { size({ w: 16, h: 8 }), radius(sm), pad(3), bg({ ref: muted }), fg({ ref: primary }) }
</scope>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>ref — primary face from a token ref</p>
    <p slot="spell" style(spell)>bg ref: primary</p>
    <column style(refBg)></column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>ref in a color slot — accent ink</p>
    <p slot="spell" style(spell)>fg color: ref accent</p>
    <column style(refFg)>Aa</column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>op mix — primary 50% onto accent</p>
    <p slot="spell" style(spell)>bg op: mix from primary onto accent amount 50</p>
    <column style(mix)></column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>chain — pad + muted face + primary ink, both refs</p>
    <p slot="spell" style(spell)>pad 3 · bg ref muted · fg ref primary</p>
    <column style(chain)>Aa</column>
  </l0-case>
</column>
