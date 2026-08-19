<!--
  Look: a painted box whose face spends every bg slot.
  Spelling: bg({ color, mix, image, clip, glow, gradient })
  Owner: language (bg slot table)
  Default: color (bare bg(primary) fills the color slot)
  Trap: fixtures/gradient-unknown.bad.sail
-->
<column gap="6">
  <l0-case look="presets — the product names" spell="bg(primary) · bg(accent) · bg(muted) · bg(surface)">
    <row gap="3">
      <l0-bg-preset kind="primary"></l0-bg-preset>
      <l0-bg-preset kind="accent"></l0-bg-preset>
      <l0-bg-preset kind="muted"></l0-bg-preset>
      <l0-bg-preset kind="surface"></l0-bg-preset>
    </row>
  </l0-case>
  <l0-case look="color — primary · accent · muted · surface" spell="bg color: primary | accent | muted | surface">
    <row gap="3">
      <l0-bg-face kind="color-primary"></l0-bg-face>
      <l0-bg-face kind="color-accent"></l0-bg-face>
      <l0-bg-face kind="color-muted"></l0-bg-face>
      <l0-bg-face kind="color-surface"></l0-bg-face>
    </row>
  </l0-case>
  <l0-case look="mix — primary 15 · 50 · 85 onto transparent" spell="bg color: primary mix: 15 | 50 | 85">
    <row gap="3">
      <l0-bg-face kind="mix-15"></l0-bg-face>
      <l0-bg-face kind="mix-50"></l0-bg-face>
      <l0-bg-face kind="mix-85"></l0-bg-face>
    </row>
  </l0-case>
  <l0-case look="image — primary-accent · none" spell="bg image: primary-accent | none">
    <row gap="3">
      <l0-bg-face kind="image-primary-accent"></l0-bg-face>
      <l0-bg-face kind="image-none"></l0-bg-face>
    </row>
  </l0-case>
  <l0-case look="clip — border-box · padding-box · content-box · text" spell="bg clip: border-box | padding-box | content-box | text">
    <row gap="3" align="center">
      <l0-bg-face kind="clip-border"></l0-bg-face>
      <l0-bg-face kind="clip-padding"></l0-bg-face>
      <l0-bg-face kind="clip-content"></l0-bg-face>
      <l0-bg-face kind="clip-text">Aa</l0-bg-face>
    </row>
  </l0-case>
  <l0-case look="glow — primary · accent · none" spell="bg glow: primary | accent | none">
    <row gap="3">
      <l0-bg-face kind="glow-primary"></l0-bg-face>
      <l0-bg-face kind="glow-accent"></l0-bg-face>
      <l0-bg-face kind="glow-none"></l0-bg-face>
    </row>
  </l0-case>
  <l0-case look="gradient — primary · accent · surface · none" spell="bg gradient: primary | accent | surface | none">
    <row gap="3">
      <l0-bg-face kind="grad-primary"></l0-bg-face>
      <l0-bg-face kind="grad-accent"></l0-bg-face>
      <l0-bg-face kind="grad-surface"></l0-bg-face>
      <l0-bg-face kind="grad-none"></l0-bg-face>
    </row>
  </l0-case>
</column>
