<!--
  Look: an 8-step painted box that clips a taller child
  Spelling: style clip = { size({ w: 8, h: 8 }), overflow(hidden), bg(muted) }
            Child is size({ h: 16 }) painted accent. 8/16 are space steps.
  Owner: language (overflow)
  Default: visible
  Trap: fixtures/overflow-unknown.bad.sail — Style error: style `overflow:all(foo)` — value is not a token on this setter's scale
        Quoted below. Not compiled on this page.

        <column style({ overflow(foo) })>x</column>
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style clip = { size({ w: 16, h: 16 }), overflow(hidden), bg(muted) }
  style tall = { size({ w: 8, h: 24 }), bg(accent) }
</scope>
<column gap="3">
  <p style(look)>64px well clips a 96px-tall child</p>
  <p style(spell)>overflow(hidden) · size w:16 h:16 · child h:24</p>
  <column style(clip)>
    <column style(tall)></column>
  </column>
</column>
