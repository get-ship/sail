<!--
  Look: a painted box inset 4 steps from the top-left of a relative host
  Spelling: position relative + absolute t:4 l:4
  Owner: language (position)
  Trap: fixtures/position-unknown-scheme.bad.sail
-->
<column gap="6">
  <l0-case look="pin 4 steps from the top-left of a 24×16 host" spell="position relative + absolute t:4 l:4 · pin 8×8">
    <l0-abs-face kind="host">
      <l0-abs-face kind="pin"></l0-abs-face>
    </l0-abs-face>
  </l0-case>
  <l0-case look="static — normal flow" spell="position(static)">
    <l0-abs-face kind="static"></l0-abs-face>
  </l0-case>
  <l0-case look="relative — containing block" spell="position(relative)">
    <l0-abs-face kind="rel"></l0-abs-face>
  </l0-case>
  <l0-case look="fixed — viewport-fixed" spell="position(fixed)">
    <l0-abs-face kind="fixed"></l0-abs-face>
  </l0-case>
  <l0-case look="sticky — sticks inside the scroller" spell="position sticky t:0 in overflow(auto)">
    <l0-abs-face kind="stick-well">
      <l0-abs-face kind="stick">sticky</l0-abs-face>
      <l0-abs-face kind="stick-pad"></l0-abs-face>
    </l0-abs-face>
  </l0-case>
  <l0-case look="pin 4 steps from the bottom-right of a 24×16 host" spell="position absolute b:4 r:4 · pin 8×8">
    <l0-abs-face kind="host">
      <l0-abs-face kind="pin-br"></l0-abs-face>
    </l0-abs-face>
  </l0-case>
  <l0-case look="fills the 24×16 host" spell="position absolute inset:0">
    <l0-abs-face kind="host">
      <l0-abs-face kind="inset-0"></l0-abs-face>
    </l0-abs-face>
  </l0-case>
  <l0-case look="inset auto — 8×8 at static position in the host" spell="position absolute inset:auto">
    <l0-abs-face kind="host">
      <l0-abs-face kind="inset-auto"></l0-abs-face>
    </l0-abs-face>
  </l0-case>
  <l0-case look="auto on t b l r — 8×8 at static position" spell="position absolute t:auto b:auto l:auto r:auto">
    <l0-abs-face kind="host">
      <l0-abs-face kind="edges-auto"></l0-abs-face>
    </l0-abs-face>
  </l0-case>
</column>
