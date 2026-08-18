<!--
  Registered CE is src/layout-cluster/.
  Look: three 4-step boxes packed in a hugging nowrap row, 8px between
  Spelling: <row cluster gap="2"> + three painted children
            (bg(primary) / bg(accent) / bg(muted)). 8px is space-2.
            cluster is hug + nowrap + intrinsic kids.
  Owner: language (cluster attr)
  Default: row wraps and fills
  Trap: fixtures/cluster-on-column.bad.sail — <column> has no `cluster` — it takes: gap, align, justify, inline
        Quoted below. Not compiled on this page.

        <column cluster>x</column>
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style a = { size({ w: 16, h: 8 }), bg(primary) }
  style b = { size({ w: 16, h: 8 }), bg(accent) }
  style c = { size({ w: 16, h: 8 }), bg(muted) }
</scope>
<column gap="6">
  <column gap="2">
    <p style(look)>three 16×8 bars packed, hug + nowrap, 8px between</p>
    <p style(spell)>&lt;row cluster gap="2"&gt;</p>
    <row cluster gap="2">
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </row>
  </column>
  <column gap="2">
    <p style(look)>nowrap — single line, fills the host</p>
    <p style(spell)>&lt;row nowrap gap="2"&gt;</p>
    <row nowrap gap="2">
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </row>
  </column>
  <column gap="2">
    <p style(look)>inline — shrink-to-fit next to text</p>
    <p style(spell)>&lt;row inline gap="2"&gt;</p>
    <row inline gap="2">
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </row>
  </column>
</column>
