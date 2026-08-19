<!--
  Registered CE is src/layout-measure/.
  Look: a painted reading column (author-spelled ch)
  Spelling: style col = { size({ w: 38ch, max-w: 100% }), margin({ x: auto }), bg(muted) }
  Owner: author
  Default: hug content
  Trap: none. Reading column is not a theme token — spell the ch.
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style col = { size({ w: 38ch, max-w: 100% }), margin({ x: auto }), pad(4), bg(muted) }
  style sm = { size({ w: 20ch, max-w: 100% }), margin({ x: auto }), pad(4), bg(primary) }
  style lg = { size({ w: 60ch, max-w: 100% }), margin({ x: auto }), pad(4), bg(muted) }
</scope>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>a reading column — 38ch wide, 4-step pad</p>
    <p slot="spell" style(spell)>size({ w: 38ch, max-w: 100% }) · margin({ x: auto }) · pad(4)</p>
    <column style(col)>reading column is author-spelled ch, not a theme token</column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>narrow — 20ch</p>
    <p slot="spell" style(spell)>size({ w: 20ch, max-w: 100% })</p>
    <column style(sm)>20ch</column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>wide — 60ch</p>
    <p slot="spell" style(spell)>size({ w: 60ch, max-w: 100% })</p>
    <column style(lg)>60ch</column>
  </l0-case>
</column>
