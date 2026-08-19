<!--
  Look: six distinct sizes, theme ramp 4xl · 2xl · xl · lg · md · sm
  Spelling: <h1> … <h6> — the tags, not a type() call
  Owner: theme (tagDefaults)
  Trap: fixtures/application-default.bad.sail · const-eq-name · dead-const
-->
<column gap="6">
  <l0-case
    look="six sizes from the theme ramp — 4xl · 2xl · xl · lg · md · sm"
    spell="h1 · h2 · h3 · h4 · h5 · h6 — no type() on the tags"
    note="trap: type heading on h1 · heading alias · unused never spent"
  >
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
