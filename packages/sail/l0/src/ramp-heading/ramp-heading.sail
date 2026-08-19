<!--
  Look: six distinct sizes, theme ramp 4xl · 2xl · xl · lg · md · sm
  Spelling: <h1> … <h6> — the tags, not a type() call
  Owner: theme (tagDefaults)
  Default: UA heading sizes (not the product)
  Trap: none this slice
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
</scope>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>six sizes from the theme ramp — 4xl · 2xl · xl · lg · md · sm</p>
    <p slot="spell" style(spell)>h1 · h2 · h3 · h4 · h5 · h6 — no type() on the tags</p>
    <column>
      <h1>h1</h1>
      <h2>h2</h2>
      <h3>h3</h3>
      <h4>h4</h4>
      <h5>h5</h5>
      <h6>h6</h6>
    </column>
  </l0-case>
</column>
