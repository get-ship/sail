<!-- Default wrap row + clipped overflow host. -->
<column gap="4" style({ size({ w: 100% }).pad(4) })>
  <row gap="2" style({ size({ w: 100% }) })>
    <span>One</span>
    <span>Two</span>
    <span>Three</span>
    <span>Four</span>
    <span>Five</span>
    <span>Six</span>
    <span>Seven</span>
    <span>Eight</span>
  </row>
  <column style({ size({ w: 100%, h: 24 }).pad(2).overflow(clip) })>
    <span>Clipped body that may overflow the host height when text wraps longer than expected.</span>
  </column>
</column>
