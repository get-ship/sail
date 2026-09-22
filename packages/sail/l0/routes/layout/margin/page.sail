<!--
  Look: two painted boxes; the second sits 4 steps below the first
  Spelling: margin({ t: 4 })
  Owner: theme scale + free edges
  Trap: fixtures/margin-off-scale.bad.sail
-->
<column gap="6">
  <l0-case look="two 16×8 bars; second is 4 steps (16px) below the first" spell="margin t:4 · size w:16 h:8">
    <column>
      <l0-margin-face kind="first"></l0-margin-face>
      <l0-margin-face kind="below"></l0-margin-face>
    </column>
  </l0-case>
  <l0-case look="step list — margin(N) after a face" spell="margin(1) … margin(8) · margin(2_5)">
    <row>
      <l0-margin-face kind="bar"></l0-margin-face>
      <l0-margin-face kind="1"></l0-margin-face>
      <l0-margin-face kind="bar"></l0-margin-face>
      <l0-margin-face kind="2"></l0-margin-face>
      <l0-margin-face kind="bar"></l0-margin-face>
      <l0-margin-face kind="2_5"></l0-margin-face>
      <l0-margin-face kind="bar"></l0-margin-face>
      <l0-margin-face kind="3"></l0-margin-face>
      <l0-margin-face kind="bar"></l0-margin-face>
      <l0-margin-face kind="4"></l0-margin-face>
      <l0-margin-face kind="bar"></l0-margin-face>
      <l0-margin-face kind="5"></l0-margin-face>
      <l0-margin-face kind="bar"></l0-margin-face>
      <l0-margin-face kind="6"></l0-margin-face>
      <l0-margin-face kind="bar"></l0-margin-face>
      <l0-margin-face kind="7"></l0-margin-face>
      <l0-margin-face kind="bar"></l0-margin-face>
      <l0-margin-face kind="8"></l0-margin-face>
    </row>
  </l0-case>
  <l0-case look="edges — x y t r b l at step 4" spell="margin x / y / t / r / b / l at step 4">
    <row>
      <l0-margin-face kind="bar"></l0-margin-face>
      <l0-margin-face kind="x"></l0-margin-face>
      <l0-margin-face kind="bar"></l0-margin-face>
      <l0-margin-face kind="y"></l0-margin-face>
      <l0-margin-face kind="bar"></l0-margin-face>
      <l0-margin-face kind="t"></l0-margin-face>
      <l0-margin-face kind="bar"></l0-margin-face>
      <l0-margin-face kind="r"></l0-margin-face>
      <l0-margin-face kind="bar"></l0-margin-face>
      <l0-margin-face kind="b"></l0-margin-face>
      <l0-margin-face kind="bar"></l0-margin-face>
      <l0-margin-face kind="l"></l0-margin-face>
    </row>
  </l0-case>
</column>
