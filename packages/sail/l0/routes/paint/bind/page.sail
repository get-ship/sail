<!--
  Look: a painted box whose width ticks 240px → 480px (free px, not space steps).
  Owner: language (bind is the geometry channel)
-->
<column gap="6">
  <l0-case
    look="width ticks 240px → 480px from the boxW signal. h is space-24"
    spell="size w: bind boxW (240|480 px) · h: 24"
    note="ledger: bind of a signal still requires a CSS-var mailbox — width: var(--box-w)"
  >
    <l0-bind-face kind="w"></l0-bind-face>
  </l0-case>
  <l0-case
    look="pad x ticks 16px → 48px from the boxPad signal"
    spell="pad x: bind boxPad (16|48 px)"
    note="ledger: bind of a signal still requires a CSS-var mailbox — padding-inline: var(--box-pad)"
  >
    <l0-bind-face kind="pad"></l0-bind-face>
  </l0-case>
</column>
