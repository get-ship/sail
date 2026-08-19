<!--
  Look: three boxes in a row, aligned to the end, none stretch
  Spelling: <row align=end gap=2>
  Owner: language (align attr)
  Default: row align is center
  Trap: fixtures/align-off-enum.bad.sail
-->
<column gap="6">
  <l0-case look="three bars on a 16-step rail, align=end, 8px between" spell="&lt;row align=end gap=2&gt;">
    <l0-cross align="end">
      <l0-swatch kind="short"></l0-swatch>
      <l0-swatch kind="mid" face="accent"></l0-swatch>
      <l0-swatch kind="short" face="muted"></l0-swatch>
    </l0-cross>
  </l0-case>
  <l0-case look="align=start" spell="&lt;row align=start gap=2&gt;">
    <l0-cross align="start">
      <l0-swatch kind="short"></l0-swatch>
      <l0-swatch kind="mid" face="accent"></l0-swatch>
      <l0-swatch kind="short" face="muted"></l0-swatch>
    </l0-cross>
  </l0-case>
  <l0-case look="align=center" spell="&lt;row align=center gap=2&gt;">
    <l0-cross align="center">
      <l0-swatch kind="short"></l0-swatch>
      <l0-swatch kind="mid" face="accent"></l0-swatch>
      <l0-swatch kind="short" face="muted"></l0-swatch>
    </l0-cross>
  </l0-case>
  <l0-case look="align=stretch" spell="&lt;row align=stretch gap=2&gt;">
    <l0-cross align="stretch">
      <l0-swatch kind="stretch"></l0-swatch>
      <l0-swatch kind="stretch" face="accent"></l0-swatch>
      <l0-swatch kind="stretch" face="muted"></l0-swatch>
    </l0-cross>
  </l0-case>
  <l0-case look="align=baseline" spell="&lt;row align=baseline gap=2&gt;">
    <l0-cross align="baseline">
      <l0-align-base kind="even">Ay</l0-align-base>
      <l0-align-base face="accent" kind="high">Hg</l0-align-base>
      <l0-align-base face="muted" kind="low">x</l0-align-base>
    </l0-cross>
  </l0-case>
</column>
