<!-- PARITY-1: wrap row + clipped overflow host. -->
<column gap="4" style({ box: w(full).pad(4) })>
  <row gap="2" style({ box: w(full) })>
    <span>One</span>
    <span>Two</span>
    <span>Three</span>
    <span>Four</span>
    <span>Five</span>
    <span>Six</span>
    <span>Seven</span>
    <span>Eight</span>
  </row>
  <column style({ box: w(full).pad(2).h(24).overflow(clip) })>
    <span>Clipped body that may overflow the host height when text wraps longer than expected.</span>
  </column>
</column>
