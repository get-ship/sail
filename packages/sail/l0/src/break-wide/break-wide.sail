<!--
  Look: a painted box; primary by default, accent at wide
  Spelling: style box = { bg(primary), when: { wide: bg(accent) } }
            Theme offers wide. 4-bp table: mobile / tablet / desktop / wide.
            wide is not in today's closed table — that red is the cell.
  Owner: theme breakpoints (4-bp including wide)
  Default: primary
  Theme: mobile 390 · tablet 768 · desktop 1280 · wide 1600
  Trap: fixtures/when-undeclared.bad.sail — Style error: style when key `kiosk`
        is not in the closed browser-state table
        (kiosk is neither a language state nor a theme breakpoint).
        Quoted below. Not compiled on this page.

        <column style({ bg(primary), when: { kiosk: bg(accent) } })>x</column>
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style box = { size({ w: 16, h: 16 }), bg(primary), when: { wide: bg(accent) } }
</scope>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>64px box: primary, accent at wide</p>
    <p slot="spell" style(spell)>when wide · bg(accent) · default bg(primary)</p>
    <p slot="note" style(spell)>theme 4-bp: mobile / tablet / desktop / wide 1600</p>
    <column style(box)></column>
  </l0-case>
</column>
