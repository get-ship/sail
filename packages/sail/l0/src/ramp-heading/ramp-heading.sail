<!--
  Look: six distinct sizes, theme ramp 4xl · 2xl · xl · lg · md · sm
  Spelling: <h1> … <h6> — the tags, not a type() call
  Owner: theme (tagDefaults)
  Default: UA heading sizes (not the product)
  Trap: fixtures/application-default.bad.sail — `type(heading)` is the default
        for `<h1>` — omit it, or is this not an h1?
        fixtures/const-eq-name.bad.sail — `heading` already names `type(heading)`
        — spend the name, don't redeclare it.
        fixtures/dead-const.bad.sail — `unused` is never spent — delete it, or
        apply `style(unused)`.
        Quoted below. Not compiled on this page (CI: pages compile;
        trap fixtures assert the error).

        <h1 style({ type(heading) })>x</h1>
        style heading = { type(heading) }
        style unused = { bg(muted) }
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
</scope>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>six sizes from the theme ramp — 4xl · 2xl · xl · lg · md · sm</p>
    <p slot="spell" style(spell)>h1 · h2 · h3 · h4 · h5 · h6 — no type() on the tags</p>
    <p slot="note" style(spell)>trap: type heading on h1 · heading alias · unused never spent</p>
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
