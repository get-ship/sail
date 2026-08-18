<scope>
  props: {}
  style band = { size({ w: 100% }), pad(5), bg(muted), radius(md) }
  style pair = { size({ w: 100%, min-h: 32 }) }
  style well = { size(grow), pad(6), bg(surface), radius(sm) }
  style nums = { size({ w: 48 }), pad({ l: 4 }) }
</scope>

<column gap="4" style(band)>
  <column gap="1">
    <slot name="look"></slot>
    <slot name="spell"></slot>
    <slot name="note"></slot>
  </column>
  <row gap="4" align="stretch" style(pair)>
    <column style(well)>
      <center>
        <slot></slot>
      </center>
    </column>
    <column gap="1" style(nums)>
    </column>
  </row>
</column>
