<!--
  Look: a painted box that stays primary. tablet is not on this theme.
  Spelling: style box = { bg(primary), when: { tablet: bg(accent) } }
            1-bp theme (mobile only). tablet arm must be authored.
            Compiles away — no tablet media in the sheet.
  Owner: theme breakpoints (1-bp, no tablet)
  Default: primary
  Theme: mobile 390
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style box = { size({ w: 16, h: 16 }), bg(primary), when: { tablet: bg(accent) } }
</scope>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>64px box: always primary — tablet compiled away</p>
    <p slot="spell" style(spell)>when tablet · bg(accent) · default bg(primary)</p>
    <p slot="note" style(spell)>theme 1-bp: mobile 390 · no tablet</p>
    <column style(box)></column>
  </l0-case>
</column>
