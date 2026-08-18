<!--
  Look: the five hosts as painted boxes, then verb links in a wrapping row.
        row: three 4-step boxes, 8px between.
        column: three 4-step boxes, 8px between.
        grid: 2-col, four 4-step boxes, 8px gap.
        center: one 4-step box.
        stack: muted box under a primary box.
  Spelling: <row> <column> <grid cols="2"> <center> <stack> + size({ w: 4, h: 4 }).
            8px is space-2 (gap="2"). Links: <row cluster gap="2">.
  Owner: language (hosts)
  Default: —
  Trap: none
-->
<scope>
  style a = { size({ w: 4, h: 4 }), bg(primary) }
  style b = { size({ w: 4, h: 4 }), bg(accent) }
  style c = { size({ w: 4, h: 4 }), bg(muted) }
</scope>
<column gap="6">
  <column gap="2">
    <p>row</p>
    <row gap="2">
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </row>
  </column>
  <column gap="2">
    <p>column</p>
    <column gap="2">
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
    </column>
  </column>
  <column gap="2">
    <p>grid</p>
    <grid cols="2" gap="2">
      <column style(a)></column>
      <column style(b)></column>
      <column style(c)></column>
      <column style(a)></column>
    </grid>
  </column>
  <column gap="2">
    <p>center</p>
    <center>
      <column style(b)></column>
    </center>
  </column>
  <column gap="2">
    <p>stack</p>
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
