<!--
  Look: the five hosts as painted bars.
  Spelling: <row> <column> <grid cols=2> <center> <stack> + size w:16 h:8.
  Owner: language (hosts)
-->
<column gap="6">
  <l0-case look="row — three 64×32 bars, 8px between" spell="&lt;row gap=2&gt; size w:16 h:8">
    <row gap="2">
      <l0-swatch kind="bar"></l0-swatch>
      <l0-swatch kind="bar" face="accent"></l0-swatch>
      <l0-swatch kind="bar" face="muted"></l0-swatch>
    </row>
  </l0-case>
  <l0-case look="column — same bars stacked" spell="&lt;column gap=2&gt;">
    <column gap="2">
      <l0-swatch kind="bar"></l0-swatch>
      <l0-swatch kind="bar" face="accent"></l0-swatch>
      <l0-swatch kind="bar" face="muted"></l0-swatch>
    </column>
  </l0-case>
  <l0-case look="grid — 2 columns, hug the tracks" spell="&lt;grid cols=2 gap=2&gt; size(hug)">
    <l0-hug>
      <grid cols="2" gap="2">
        <l0-swatch kind="bar"></l0-swatch>
        <l0-swatch kind="bar" face="accent"></l0-swatch>
        <l0-swatch kind="bar" face="muted"></l0-swatch>
        <l0-swatch kind="bar"></l0-swatch>
      </grid>
    </l0-hug>
  </l0-case>
  <l0-case look="center — one bar in the host" spell="&lt;center&gt;">
    <center>
      <l0-swatch kind="bar" face="accent"></l0-swatch>
    </center>
  </l0-case>
  <l0-case look="stack — overlay, muted under primary" spell="&lt;stack&gt;">
    <stack>
      <l0-swatch kind="bar" face="muted"></l0-swatch>
      <l0-swatch kind="bar"></l0-swatch>
    </stack>
  </l0-case>
</column>
