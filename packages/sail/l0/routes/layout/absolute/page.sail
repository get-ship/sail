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
  <l0-case look="sticky — sticks inside the scroller" spell="position sticky t:0 in overflow(auto)">
    <l0-abs-face kind="stick-well">
      <l0-abs-face kind="stick">sticky</l0-abs-face>
      <l0-abs-face kind="stick-pad"></l0-abs-face>
    </l0-abs-face>
  </l0-case>
</column>
