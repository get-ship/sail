<!--
  Registered CE is src/layout-absolute/.
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
  style host = { position(relative), size({ w: 16, h: 16 }), bg(muted) }
  style pin = { position({ scheme: absolute, t: 4, l: 4 }), size({ w: 4, h: 4 }), bg(primary) }
</scope>
<column style(host)>
  <column style(pin)></column>
</column>
