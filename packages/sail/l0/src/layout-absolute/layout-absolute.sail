<!--
  Look: a painted box inset 4 steps from the top-left of a relative host
  Spelling: style host = { position(relative), size({ w: 16, h: 16 }), bg(muted) }
            style pin = { position({ scheme: absolute, t: 4, l: 4 }), size({ w: 4, h: 4 }), bg(primary) }
            4/16 are space steps.
  Owner: language (position)
  Default: static
  Trap: fixtures/position-unknown-scheme.bad.sail — Style error: style `position:scheme(middle)` — value is not a token on this setter's scale
        Quoted below. Not compiled on this page.

        <column style({ position({ scheme: middle }) })>x</column>
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style host = { position(relative), size({ w: 24, h: 16 }), bg(muted) }
  style pin = { position({ scheme: absolute, t: 4, l: 4 }), size({ w: 8, h: 8 }), bg(primary) }
  style stat = { position(static), size({ w: 16, h: 8 }), bg(accent) }
  style rel = { position(relative), size({ w: 16, h: 8 }), bg(primary) }
  style stickWell = { size({ w: 24, h: 16 }), overflow(auto), bg(muted) }
  style stick = { position({ scheme: sticky, t: 0 }), size({ w: 16, h: 8 }), bg(accent) }
  style stickPad = { size({ h: 24 }), bg(muted) }
</scope>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>pin 4 steps from the top-left of a 24×16 host</p>
    <p slot="spell" style(spell)>position relative + absolute t:4 l:4 · pin 8×8</p>
    <column style(host)>
      <column style(pin)></column>
    </column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>static — normal flow</p>
    <p slot="spell" style(spell)>position(static)</p>
    <column style(stat)></column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>relative — containing block</p>
    <p slot="spell" style(spell)>position(relative)</p>
    <column style(rel)></column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>sticky — sticks inside the scroller</p>
    <p slot="spell" style(spell)>position sticky t:0 in overflow(auto)</p>
    <column style(stickWell)>
      <column style(stick)>sticky</column>
      <column style(stickPad)></column>
    </column>
  </l0-case>
</column>
