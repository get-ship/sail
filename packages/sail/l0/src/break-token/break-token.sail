<!--
  Look: muted host, page inset around a primary face. Inset is one token,
        keyed — space-4 at base, space-6 at tablet. Not a hex dump.
  Spelling: style host = { pad(page), bg(muted) } · face bg(primary)
            Theme keys pad.page: { base: { x: space-4 }, tablet: { x: space-6 } }
  Owner: theme (breakpoint-keyed token spent in the skin)
  Default: no inset
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style host = { size({ w: 100% }), pad(page), bg(muted) }
  style face = { size({ h: 8 }), bg(primary) }
</scope>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>page inset keyed — space-4 at base, space-6 at tablet</p>
    <p slot="spell" style(spell)>pad(page) · theme keys page base:4 tablet:6</p>
    <column style(host)>
      <column style(face)></column>
    </column>
  </l0-case>
</column>
