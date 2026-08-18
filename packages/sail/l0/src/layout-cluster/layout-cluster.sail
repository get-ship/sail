<!--
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
  style a = { size({ w: 4, h: 4 }), bg(primary) }
  style b = { size({ w: 4, h: 4 }), bg(accent) }
  style c = { size({ w: 4, h: 4 }), bg(muted) }
</scope>
<row cluster gap="2">
  <column style(a)></column>
  <column style(b)></column>
  <column style(c)></column>
</row>
