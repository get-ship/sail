<!--
  Look: three 4-step boxes, 8px between, none stretch
  Spelling: <row gap=2> + three painted children. 8px is space-2.
  Owner: language (gap) + theme scale for the step
  Default: no gap
  Trap: fixtures/gap-off-enum.bad.sail — <row gap=3.5>
-->
<column gap="6">
  <l0-case look="three 4-step boxes (16px), 8px between, none stretch" spell="&lt;row gap=2&gt; size w:4 h:4 · 8px is space-2">
    <row gap="2">
      <l0-swatch></l0-swatch>
      <l0-swatch face="accent"></l0-swatch>
      <l0-swatch face="muted"></l0-swatch>
    </row>
  </l0-case>
  <l0-case look="gap=0 — three faces, no space" spell="&lt;row gap=0&gt;">
    <row gap="0">
      <l0-swatch></l0-swatch>
      <l0-swatch face="accent"></l0-swatch>
      <l0-swatch face="muted"></l0-swatch>
    </row>
  </l0-case>
  <l0-case look="gap=1 — 4px" spell="&lt;row gap=1&gt;">
    <row gap="1">
      <l0-swatch></l0-swatch>
      <l0-swatch face="accent"></l0-swatch>
      <l0-swatch face="muted"></l0-swatch>
    </row>
  </l0-case>
  <l0-case look="gap=2_5 — 10px" spell="&lt;row gap=2_5&gt;">
    <row gap="2_5">
      <l0-swatch></l0-swatch>
      <l0-swatch face="accent"></l0-swatch>
      <l0-swatch face="muted"></l0-swatch>
    </row>
  </l0-case>
  <l0-case look="gap=3 — 12px" spell="&lt;row gap=3&gt;">
    <row gap="3">
      <l0-swatch></l0-swatch>
      <l0-swatch face="accent"></l0-swatch>
      <l0-swatch face="muted"></l0-swatch>
    </row>
  </l0-case>
  <l0-case look="gap=4 — 16px" spell="&lt;row gap=4&gt;">
    <row gap="4">
      <l0-swatch></l0-swatch>
      <l0-swatch face="accent"></l0-swatch>
      <l0-swatch face="muted"></l0-swatch>
    </row>
  </l0-case>
  <l0-case look="gap=5 — 20px" spell="&lt;row gap=5&gt;">
    <row gap="5">
      <l0-swatch></l0-swatch>
      <l0-swatch face="accent"></l0-swatch>
      <l0-swatch face="muted"></l0-swatch>
    </row>
  </l0-case>
  <l0-case look="gap=6 — 24px" spell="&lt;row gap=6&gt;">
    <row gap="6">
      <l0-swatch></l0-swatch>
      <l0-swatch face="accent"></l0-swatch>
      <l0-swatch face="muted"></l0-swatch>
    </row>
  </l0-case>
  <l0-case look="gap=7 — 28px" spell="&lt;row gap=7&gt;">
    <row gap="7">
      <l0-swatch></l0-swatch>
      <l0-swatch face="accent"></l0-swatch>
      <l0-swatch face="muted"></l0-swatch>
    </row>
  </l0-case>
  <l0-case look="gap=8 — 32px" spell="&lt;row gap=8&gt;">
    <row gap="8">
      <l0-swatch></l0-swatch>
      <l0-swatch face="accent"></l0-swatch>
      <l0-swatch face="muted"></l0-swatch>
    </row>
  </l0-case>
  <l0-case look="one 4-step box in a gapped row" spell="&lt;row gap=2&gt; one child">
    <row gap="2">
      <l0-swatch></l0-swatch>
    </row>
  </l0-case>
</column>
