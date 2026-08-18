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
</scope>
<column gap="3">
  <p style(look)>pin 4 steps from the top-left of a 24×16 host</p>
  <p style(spell)>position relative + absolute t:4 l:4 · pin 8×8</p>
  <column style(host)>
    <column style(pin)></column>
  </column>
</column>
