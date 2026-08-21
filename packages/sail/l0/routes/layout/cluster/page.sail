<!--
  Look: three 4-step boxes packed in a hugging nowrap row, 8px between
  Spelling: <row cluster gap=2>
  Owner: language (cluster attr)
  Default: row wraps and fills
  Trap: fixtures/cluster-on-column.bad.sail
-->
<column gap="6">
  <l0-case look="three 16×8 bars packed, hug + nowrap, 8px between" spell="&lt;row cluster gap=2&gt;">
    <row cluster gap="2">
      <l0-swatch kind="bar"></l0-swatch>
      <l0-swatch kind="bar" face="accent"></l0-swatch>
      <l0-swatch kind="bar" face="muted"></l0-swatch>
    </row>
  </l0-case>
  <l0-case look="nowrap — single line, fills the host" spell="&lt;row nowrap gap=2&gt;">
    <row nowrap gap="2">
      <l0-swatch kind="bar"></l0-swatch>
      <l0-swatch kind="bar" face="accent"></l0-swatch>
      <l0-swatch kind="bar" face="muted"></l0-swatch>
    </row>
  </l0-case>
  <l0-case look="inline — shrink-to-fit next to text" spell="&lt;row inline gap=2&gt;">
    <row align="center">
      <p>sits inline with this sentence —</p>
      <row inline gap="2">
        <l0-swatch kind="bar"></l0-swatch>
        <l0-swatch kind="bar" face="accent"></l0-swatch>
        <l0-swatch kind="bar" face="muted"></l0-swatch>
      </row>
    </row>
  </l0-case>
</column>
