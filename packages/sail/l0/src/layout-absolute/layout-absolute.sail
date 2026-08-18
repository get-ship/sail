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
  style stick = { position(sticky), size({ w: 16, h: 8 }), bg(accent) }
  style stickPad = { size({ h: 24 }), bg(muted) }
</scope>
<column gap="6">
  <column gap="2">
    <p style(look)>pin 4 steps from the top-left of a 24×16 host</p>
    <p style(spell)>position relative + absolute t:4 l:4 · pin 8×8</p>
    <column style(host)>
      <column style(pin)></column>
    </column>
  </column>
  <column gap="2">
    <p style(look)>static — normal flow</p>
    <p style(spell)>position(static)</p>
    <column style(stat)></column>
  </column>
  <column gap="2">
    <p style(look)>relative — containing block</p>
    <p style(spell)>position(relative)</p>
    <column style(rel)></column>
  </column>
  <column gap="2">
    <p style(look)>sticky — sticks inside the scroller</p>
    <p style(spell)>position(sticky) in overflow(auto)</p>
    <column style(stickWell)>
      <column style(stick)>sticky</column>
      <column style(stickPad)></column>
    </column>
  </column>
</column>
