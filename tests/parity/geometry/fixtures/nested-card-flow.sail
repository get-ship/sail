<!-- PARITY-1: nested L1-ish flow (cards stack). Chrome paint is RENDER-1; geometry only here. -->
<column gap="4" w="full">
  style({ box: pad(4) })

  <card w="full">
    <span slot="header">Card A</span>
    <span>Body A</span>
  </card>
  <card w="full">
    <span slot="header">Card B</span>
    <span>Body B</span>
  </card>
</column>
