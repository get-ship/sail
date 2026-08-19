<!--
  Look: the five hosts as painted bars.
        row: three 16-step × 8-step bars, 8px between.
        column: the same bars stacked, 8px between.
        grid: 2-col hug, four bars, 8px gap.
        center: one bar.
        stack: muted bar under a primary bar.
  Spelling: <row> <column> <grid cols="2"> <center> <stack> + size({ w: 16, h: 8 }).
            8px is space-2 (gap="2"). 16-step is 64px.
  Owner: language (hosts)
  Default: —
  Trap: none — hosts have no trap of their own. Axis traps live on
        align / cluster / grid / gap.
-->
<scope>
  style look = { type(label), fg(muted) }
  style spell = { type(mono), fg(muted) }
  style a = { size({ w: 16, h: 8 }), bg(primary) }
  style b = { size({ w: 16, h: 8 }), bg(accent) }
  style c = { size({ w: 16, h: 8 }), bg(muted) }
  style hug = { size(hug) }
</scope>
<column gap="6">
  <l0-case>
    <p slot="look" style(look)>row — three 64×32 bars, 8px between</p>
    <p slot="spell" style(spell)>&lt;row gap="2"&gt; size w:16 h:8</p>
    <row gap="2">
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </row>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>column — same bars stacked</p>
    <p slot="spell" style(spell)>&lt;column gap="2"&gt;</p>
    <column gap="2">
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </column>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>grid — 2 columns, hug the tracks</p>
    <p slot="spell" style(spell)>&lt;grid cols="2" gap="2"&gt; size(hug)</p>
    <grid cols="2" gap="2" style(hug)>
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
      <column style(a)></column>
    </grid>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>center — one bar in the host</p>
    <p slot="spell" style(spell)>&lt;center&gt;</p>
    <center>
      <column style(b)></column>
    </center>
  </l0-case>
  <l0-case>
    <p slot="look" style(look)>stack — overlay, muted under primary</p>
    <p slot="spell" style(spell)>&lt;stack&gt;</p>
    <stack>
      <column style(c)></column>
      <column style(a)></column>
    </stack>
  </l0-case>
</column>
