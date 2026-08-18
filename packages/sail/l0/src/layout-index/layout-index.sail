<!--
  Look: the five hosts as painted bars, then verb links in a wrapping row.
        row: three 16-step × 8-step bars, 8px between.
        column: the same bars stacked, 8px between.
        grid: 2-col hug, four bars, 8px gap.
        center: one bar.
        stack: muted bar under a primary bar.
  Spelling: <row> <column> <grid cols="2"> <center> <stack> + size({ w: 16, h: 8 }).
            8px is space-2 (gap="2"). 16-step is 64px. Links: <row cluster gap="2">.
  Owner: language (hosts)
  Default: —
  Trap: none
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
  <column gap="2">
    <p style(look)>row — three 64×32 bars, 8px between</p>
    <p style(spell)>&lt;row gap="2"&gt; size w:16 h:8</p>
    <row gap="2">
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </row>
  </column>
  <column gap="2">
    <p style(look)>column — same bars stacked</p>
    <p style(spell)>&lt;column gap="2"&gt;</p>
    <column gap="2">
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </column>
  </column>
  <column gap="2">
    <p style(look)>grid — 2 columns, hug the tracks</p>
    <p style(spell)>&lt;grid cols="2" gap="2"&gt; size(hug)</p>
    <grid cols="2" gap="2" style(hug)>
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
      <column style(a)></column>
    </grid>
  </column>
  <column gap="2">
    <p style(look)>center — one bar in the host</p>
    <p style(spell)>&lt;center&gt;</p>
    <center>
      <column style(b)></column>
    </center>
  </column>
  <column gap="2">
    <p style(look)>stack — overlay, muted under primary</p>
    <p style(spell)>&lt;stack&gt;</p>
    <stack>
      <column style(c)></column>
      <column style(a)></column>
    </stack>
  </column>
  <row cluster gap="2">
    <a href="/layout/gap">gap</a>
    <a href="/layout/pad">pad</a>
    <a href="/layout/margin">margin</a>
    <a href="/layout/box">box</a>
    <a href="/layout/participate">participate</a>
    <a href="/layout/align">align</a>
    <a href="/layout/cluster">cluster</a>
    <a href="/layout/measure">measure</a>
    <a href="/layout/when">when</a>
    <a href="/layout/overflow">overflow</a>
    <a href="/layout/absolute">absolute</a>
    <a href="/layout/grid">grid</a>
    <a href="/layout/seeds">seeds</a>
  </row>
</column>
