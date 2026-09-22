<!--
  Look: one painted box with 4-step horizontal inset (pad x:4)
  Spelling: pad({ x: 4 })
  Owner: theme scale + free edges
  Trap: fixtures/pad-off-scale.bad.sail
-->
<column gap="6">
  <l0-case look="16-step bar, 4-step (16px) inset on x" spell="pad x:4 · size w:16 h:8 · bg(primary)">
    <l0-pad-face kind="pin">x</l0-pad-face>
  </l0-case>
  <l0-case look="step list — pad(N) on one face, same host" spell="pad(1) … pad(8) · pad(2_5)">
    <row gap="2">
      <l0-pad-face kind="1"></l0-pad-face>
      <l0-pad-face kind="2"></l0-pad-face>
      <l0-pad-face kind="2_5"></l0-pad-face>
      <l0-pad-face kind="3"></l0-pad-face>
      <l0-pad-face kind="4"></l0-pad-face>
      <l0-pad-face kind="5"></l0-pad-face>
      <l0-pad-face kind="6"></l0-pad-face>
      <l0-pad-face kind="7"></l0-pad-face>
      <l0-pad-face kind="8"></l0-pad-face>
    </row>
  </l0-case>
  <l0-case look="edges — x y t r b l at step 4" spell="pad x / y / t / r / b / l at step 4">
    <row gap="2">
      <l0-pad-face kind="x">x</l0-pad-face>
      <l0-pad-face kind="y">y</l0-pad-face>
      <l0-pad-face kind="t">t</l0-pad-face>
      <l0-pad-face kind="r">r</l0-pad-face>
      <l0-pad-face kind="b">b</l0-pad-face>
      <l0-pad-face kind="l">l</l0-pad-face>
    </row>
  </l0-case>
</column>
