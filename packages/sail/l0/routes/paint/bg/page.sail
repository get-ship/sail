<!--
  Look: a painted box whose face spends every bg slot.
  Spelling: bg({ color, mix, image, clip, glow, gradient })
  Owner: language (bg slot table)
  Default: color (bare bg(primary) fills the color slot)
  Trap: fixtures/gradient-unknown.bad.sail
-->
<column gap="6">
  <l0-case look="presets — app · surface · subtle · inset · overlay · primary · accent · muted" spell="bg(app) · bg(surface) · bg(subtle) · bg(inset) · bg(overlay) · bg(primary) · bg(accent) · bg(muted)">
    <row gap="3">
      <l0-bg-preset kind="app"></l0-bg-preset>
      <l0-bg-preset kind="surface"></l0-bg-preset>
      <l0-bg-preset kind="subtle"></l0-bg-preset>
      <l0-bg-preset kind="inset"></l0-bg-preset>
      <l0-bg-preset kind="overlay"></l0-bg-preset>
      <l0-bg-preset kind="primary"></l0-bg-preset>
      <l0-bg-preset kind="accent"></l0-bg-preset>
      <l0-bg-preset kind="muted"></l0-bg-preset>
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
  <l0-case look="color keywords — transparent · current · currentColor · inherit · none · CanvasText · Highlight" spell="bg color: transparent | current | currentColor | inherit | none | CanvasText | Highlight">
    <row gap="3">
      <l0-bg-face kind="color-transparent"></l0-bg-face>
      <l0-bg-face kind="color-current"></l0-bg-face>
      <l0-bg-face kind="color-currentColor"></l0-bg-face>
      <l0-bg-face kind="color-inherit"></l0-bg-face>
      <l0-bg-face kind="color-none"></l0-bg-face>
      <l0-bg-face kind="color-CanvasText"></l0-bg-face>
      <l0-bg-face kind="color-Highlight"></l0-bg-face>
    </row>
  </l0-case>
  <l0-case look="mix — primary 5 · 10 · 15 · 20 · 25 · 30 · 35 · 40 · 45 · 50 · 55 · 60 · 65 · 70 · 75 · 80 · 85 · 90 · 95 onto transparent" spell="bg color: primary mix: 5 | 10 | 15 | 20 | 25 | 30 | 35 | 40 | 45 | 50 | 55 | 60 | 65 | 70 | 75 | 80 | 85 | 90 | 95">
    <row gap="3">
      <l0-bg-face kind="mix-5"></l0-bg-face>
      <l0-bg-face kind="mix-10"></l0-bg-face>
      <l0-bg-face kind="mix-15"></l0-bg-face>
      <l0-bg-face kind="mix-20"></l0-bg-face>
      <l0-bg-face kind="mix-25"></l0-bg-face>
      <l0-bg-face kind="mix-30"></l0-bg-face>
      <l0-bg-face kind="mix-35"></l0-bg-face>
      <l0-bg-face kind="mix-40"></l0-bg-face>
      <l0-bg-face kind="mix-45"></l0-bg-face>
      <l0-bg-face kind="mix-50"></l0-bg-face>
      <l0-bg-face kind="mix-55"></l0-bg-face>
      <l0-bg-face kind="mix-60"></l0-bg-face>
      <l0-bg-face kind="mix-65"></l0-bg-face>
      <l0-bg-face kind="mix-70"></l0-bg-face>
      <l0-bg-face kind="mix-75"></l0-bg-face>
      <l0-bg-face kind="mix-80"></l0-bg-face>
      <l0-bg-face kind="mix-85"></l0-bg-face>
      <l0-bg-face kind="mix-90"></l0-bg-face>
      <l0-bg-face kind="mix-95"></l0-bg-face>
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
  <l0-case
    look="trap — rainbow is not a gradient molecule"
    spell="bg({ gradient: rainbow }) — fixtures/gradient-unknown.bad.sail"
    note="trap: unknown gradient molecule rainbow"
  ></l0-case>
</column>
