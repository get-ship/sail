<!--
  Look: one painted box of free width 7
  Spelling: size({ w: 7 })
  Owner: free
  Trap: fixtures/size-wide.bad.sail
-->
<column gap="6">
  <l0-case look="free width 7 (28px), height 8 (32px)" spell="size w:7 h:8 · bg(accent)">
    <l0-box-face kind="pin"></l0-box-face>
  </l0-case>
  <l0-case look="w — 16-step bar" spell="size w:16 h:8">
    <l0-box-face kind="w"></l0-box-face>
  </l0-case>
  <l0-case look="h — 16-step tall" spell="size w:8 h:16">
    <l0-box-face kind="h"></l0-box-face>
  </l0-case>
  <l0-case look="min-w — at least 16 steps" spell="size min-w:16 h:8">
    <l0-box-face kind="min-w"></l0-box-face>
  </l0-case>
  <l0-case look="min-h — at least 16 steps" spell="size w:8 min-h:16">
    <l0-box-face kind="min-h"></l0-box-face>
  </l0-case>
  <l0-case look="max-w — at most 16 steps" spell="size max-w:16 h:8">
    <l0-box-face kind="max-w">max-w 16</l0-box-face>
  </l0-case>
  <l0-case look="max-h — at most 8 steps" spell="size w:16 max-h:8">
    <l0-box-face kind="max-h">max-h 8</l0-box-face>
  </l0-case>
  <l0-case look="aspect square" spell="size w:16 aspect:square">
    <l0-box-face kind="square"></l0-box-face>
  </l0-case>
  <l0-case look="aspect video" spell="size w:16 aspect:video">
    <l0-box-face kind="video"></l0-box-face>
  </l0-case>
  <l0-case look="aspect auto" spell="size w:16 h:8 aspect:auto">
    <l0-box-face kind="auto"></l0-box-face>
  </l0-case>
  <l0-case look="one 4-step child in a 16-step bar" spell="size w:16 h:8 · one child">
    <l0-box-face kind="w">
      <l0-swatch face="accent"></l0-swatch>
    </l0-box-face>
  </l0-case>
  <l0-case look="zero box" spell="size w:0 h:0">
    <l0-box-face kind="zero"></l0-box-face>
  </l0-case>
  <l0-case look="16-step clamp, 48-step child" spell="size max-w:16 overflow(hidden) · child w:48">
    <l0-box-face kind="clamp">
      <l0-box-face kind="huge"></l0-box-face>
    </l0-box-face>
  </l0-case>
</column>
