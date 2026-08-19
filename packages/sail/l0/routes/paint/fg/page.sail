<!--
  Look: a painted box whose ink spends every fg slot.
  Spelling: fg({ color, caret, accent, gradient })
  Owner: language (fg slot table)
  Default: color (bare fg(primary) fills the color slot) · caret auto · accent auto
  Trap: fixtures/gradient-unknown.bad.sail
-->
<column gap="6">
  <l0-case look="color — primary · accent · muted · surface" spell="fg color: primary | accent | muted | surface">
    <row gap="3" align="center">
      <l0-fg-face kind="color-primary">Aa</l0-fg-face>
      <l0-fg-face kind="color-accent">Aa</l0-fg-face>
      <l0-fg-face kind="color-muted">Aa</l0-fg-face>
      <l0-fg-face kind="color-surface">Aa</l0-fg-face>
    </row>
  </l0-case>
  <l0-case look="caret — auto · primary · transparent · current · currentColor" spell="fg caret: auto | primary | transparent | current | currentColor">
    <row gap="3" align="center">
      <l0-fg-face kind="caret-auto">Aa</l0-fg-face>
      <l0-fg-face kind="caret-primary">Aa</l0-fg-face>
      <l0-fg-face kind="caret-transparent">Aa</l0-fg-face>
      <l0-fg-face kind="caret-current">Aa</l0-fg-face>
      <l0-fg-face kind="caret-currentColor">Aa</l0-fg-face>
    </row>
  </l0-case>
  <l0-case look="accent — auto · primary · transparent · current · currentColor" spell="fg accent: auto | primary | transparent | current | currentColor">
    <row gap="3" align="center">
      <l0-fg-face kind="accent-auto">Aa</l0-fg-face>
      <l0-fg-face kind="accent-primary">Aa</l0-fg-face>
      <l0-fg-face kind="accent-transparent">Aa</l0-fg-face>
      <l0-fg-face kind="accent-current">Aa</l0-fg-face>
      <l0-fg-face kind="accent-currentColor">Aa</l0-fg-face>
    </row>
  </l0-case>
  <l0-case look="gradient — primary · accent · surface · none" spell="fg gradient: primary | accent | surface | none">
    <row gap="3" align="center">
      <l0-fg-face kind="grad-primary">Aa</l0-fg-face>
      <l0-fg-face kind="grad-accent">Aa</l0-fg-face>
      <l0-fg-face kind="grad-surface">Aa</l0-fg-face>
      <l0-fg-face kind="grad-none">Aa</l0-fg-face>
    </row>
  </l0-case>
</column>
