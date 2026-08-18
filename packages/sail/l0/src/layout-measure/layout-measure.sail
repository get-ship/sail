<!--
  Look: a painted reading column (theme measure width)
  Spelling: style col = { size(measure), bg(muted) }
  Owner: theme
  Default: hug content
  Trap: none as a compile error. size(measure) on a sidebar is mapping
        crime (the look is not a reading column) — forbidden on these
        pages, not a new diagnostic this slice.
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style col = { size(measure), pad(4), bg(muted) }
  style sm = { size(measure-sm), pad(4), bg(primary) }
  style md = { size(measure-md), pad(4), bg(accent) }
  style lg = { size(measure-lg), pad(4), bg(muted) }
</scope>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>a reading column — theme measure width, 4-step pad</p>
    <p slot="spell" style(spell)>size(measure) · pad(4) · bg(muted)</p>
    <column style(col)>measure is a reading column, not a width stamp</column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>measure-sm — 20rem reading column</p>
    <p slot="spell" style(spell)>size(measure-sm) · pad(4)</p>
    <column style(sm)>measure-sm</column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>measure-md — 38ch, same as measure</p>
    <p slot="spell" style(spell)>size(measure-md) · pad(4)</p>
    <column style(md)>measure-md</column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>measure-lg — 60ch reading column</p>
    <p slot="spell" style(spell)>size(measure-lg) · pad(4)</p>
    <column style(lg)>measure-lg</column>
  </l0-case>
</column>
