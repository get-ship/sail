<!--
  Look: two faces spend bg(surface). Shared concern, one .s-bg-surface.
  Spelling: two columns, style faceA / faceB — both bg(surface), pad 4 vs pad 8
  Owner: language (shared concern rules)
  Default: one rule per unique concern application
  Trap: none this slice — shared_concern_emits_one_rule owns the sheet.
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style faceA = { size({ w: 8, h: 8 }), pad(4), bg(surface), fg(muted), type(mono), radius(sm) }
  style faceB = { size({ w: 8, h: 8 }), pad(8), bg(surface), fg(muted), type(mono), radius(sm) }
</scope>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>two faces share bg(surface) — one .s-bg-surface</p>
    <p slot="spell" style(spell)>bg(surface) · pad(4) vs pad(8)</p>
    <row gap="2">
      <column data-inspect-a style(faceA)>4</column>
      <column data-inspect-b style(faceB)>8</column>
    </row>
  </l0-case>
</column>
