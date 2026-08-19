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
  style auto = { size({ w: 16, h: 16 }), overflow(auto), bg(muted) }
  style clipWell = { size({ w: 16, h: 16 }), overflow(clip), bg(muted) }
  style scroll = { size({ w: 16, h: 16 }), overflow(scroll), bg(muted) }
  style vis = { size({ w: 16, h: 16 }), overflow(visible), bg(muted) }
  style tall = { size({ w: 8, h: 24 }), bg(accent) }
  style clamp = { size({ max-w: 16, h: 8 }), overflow(hidden), bg(muted) }
  style huge = { size({ w: 48, h: 8 }), bg(accent) }
</scope>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>64px well clips a 96px-tall child</p>
    <p slot="spell" style(spell)>overflow(hidden) · size w:16 h:16 · child h:24</p>
    <column style(clip)>
      <column style(tall)></column>
    </column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>overflow(auto) — scroll if needed</p>
    <p slot="spell" style(spell)>overflow(auto) · child h:24</p>
    <column style(auto)>
      <column style(tall)></column>
    </column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>overflow(clip) — clip without a scrollport</p>
    <p slot="spell" style(spell)>overflow(clip) · child h:24</p>
    <column style(clipWell)>
      <column style(tall)></column>
    </column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>overflow(scroll) — always a scrollport</p>
    <p slot="spell" style(spell)>overflow(scroll) · child h:24</p>
    <column style(scroll)>
      <column style(tall)></column>
    </column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>overflow(visible) — do not clip</p>
    <p slot="spell" style(spell)>overflow(visible) · child h:24</p>
    <column style(vis)>
      <column style(tall)></column>
    </column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>16-step clamp, 48-step child</p>
    <p slot="spell" style(spell)>max-w:16 overflow(hidden) · child w:48</p>
    <column style(clamp)>
      <column style(huge)></column>
    </column>
  </l0-case>
</column>
