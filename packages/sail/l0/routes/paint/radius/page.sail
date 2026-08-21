<!--
  Look: painted boxes, one per radius name — none sm md lg xl full · all t b l r
  Spelling: radius(none) · radius(sm) · radius(md) · radius(lg) · radius(xl) · radius(full) · radius({ all: md }) · radius({ t: lg }) · radius({ b: lg }) · radius({ l: lg }) · radius({ r: lg })
  Owner: language (radius) + theme scale
  Default: none
  Trap: fixtures/radius-unknown.bad.sail
-->
<column gap="6">
  <l0-case
    look="presets — the product names"
    spell="radius(none) · radius(sm) · radius(md) · radius(lg) · radius(xl) · radius(full)"
    note="trap: radius(xs) — xs is not a token on the radius scale"
  >
    <row gap="2">
      <l0-radius-face kind="none"></l0-radius-face>
      <l0-radius-face kind="sm"></l0-radius-face>
      <l0-radius-face kind="md"></l0-radius-face>
      <l0-radius-face kind="lg"></l0-radius-face>
      <l0-radius-face kind="xl"></l0-radius-face>
      <l0-radius-face kind="full"></l0-radius-face>
    </row>
  </l0-case>
  <l0-case look="slots — all t b l r at md / lg" spell="radius all:md · t:lg · b:lg · l:lg · r:lg">
    <row gap="2">
      <l0-radius-face kind="all"></l0-radius-face>
      <l0-radius-face kind="t"></l0-radius-face>
      <l0-radius-face kind="b"></l0-radius-face>
      <l0-radius-face kind="l"></l0-radius-face>
      <l0-radius-face kind="r"></l0-radius-face>
    </row>
  </l0-case>
</column>
