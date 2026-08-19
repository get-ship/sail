<!--
  Look: an 8-step painted box that clips a taller child
  Spelling: overflow(hidden)
  Owner: language (overflow)
  Trap: fixtures/overflow-unknown.bad.sail
-->
<column gap="6">
  <l0-case look="64px well clips a 96px-tall child" spell="overflow(hidden) · size w:16 h:16 · child h:24">
    <l0-overflow-face kind="hidden">
      <l0-overflow-face kind="tall"></l0-overflow-face>
    </l0-overflow-face>
  </l0-case>
  <l0-case look="overflow(auto) — scroll if needed" spell="overflow(auto) · child h:24">
    <l0-overflow-face kind="auto">
      <l0-overflow-face kind="tall"></l0-overflow-face>
    </l0-overflow-face>
  </l0-case>
  <l0-case look="overflow(clip) — clip without a scrollport" spell="overflow(clip) · child h:24">
    <l0-overflow-face kind="clip">
      <l0-overflow-face kind="tall"></l0-overflow-face>
    </l0-overflow-face>
  </l0-case>
  <l0-case look="overflow(scroll) — always a scrollport" spell="overflow(scroll) · child h:24">
    <l0-overflow-face kind="scroll">
      <l0-overflow-face kind="tall"></l0-overflow-face>
    </l0-overflow-face>
  </l0-case>
  <l0-case look="overflow(visible) — do not clip" spell="overflow(visible) · child h:24">
    <l0-overflow-face kind="visible">
      <l0-overflow-face kind="tall"></l0-overflow-face>
    </l0-overflow-face>
  </l0-case>
  <l0-case look="16-step clamp, 48-step child" spell="max-w:16 overflow(hidden) · child w:48">
    <l0-overflow-face kind="clamp">
      <l0-overflow-face kind="huge"></l0-overflow-face>
    </l0-overflow-face>
  </l0-case>
</column>
