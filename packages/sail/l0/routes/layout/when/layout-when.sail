<!--
  Registered CE is src/layout-when/.
  Look: a painted box; primary by default, accent at tablet, muted at desktop
  Spelling: style box = { bg(primary), when: { tablet: bg(accent), desktop: bg(muted) } }
            Language keys + theme breakpoints. Colored-box demo, not an app remap.
  Owner: language keys, theme breakpoints
  Default: no breakpoint arms
  Trap: fixtures/when-unknown-key.bad.sail — Style error: style when key `phone` is not in the closed browser-state table
        Quoted below. Not compiled on this page.

        <column style({ bg(primary), when: { phone: bg(accent) } })>x</column>
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style box = { size({ w: 16, h: 16 }), bg(primary), when: { tablet: bg(accent), desktop: bg(muted) } }
  style mobile = { size({ w: 16, h: 16 }), bg(muted), when: { mobile: bg(primary) } }
  style tablet = { size({ w: 16, h: 16 }), bg(muted), when: { tablet: bg(accent) } }
  style desktop = { size({ w: 16, h: 16 }), bg(muted), when: { desktop: bg(primary) } }
</scope>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>64px box: primary, accent at tablet, muted at desktop</p>
    <p slot="spell" style(spell)>when tablet / desktop · size w:16 h:16</p>
    <column style(box)></column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>mobile — primary at mobile, muted otherwise</p>
    <p slot="spell" style(spell)>when mobile · bg(primary)</p>
    <column style(mobile)></column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>tablet — accent at tablet, muted otherwise</p>
    <p slot="spell" style(spell)>when tablet · bg(accent)</p>
    <column style(tablet)></column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>desktop — primary at desktop, muted otherwise</p>
    <p slot="spell" style(spell)>when desktop · bg(primary)</p>
    <column style(desktop)></column>
  </l0-case>
</column>
