<!-- PARITY-1: sidebar-left row (grow/shrink/nowrap). -->
<column style({ box: w(full).minH(screen) })>
  <row nowrap align="stretch" style({ box: grow(1).w(full) })>
    <column style({ box: shrink(0).w(48).pad(3) })>
      <span>Side</span>
    </column>
    <column style({ box: grow(1).minW(0).pad(3) })>
      <span>Main content</span>
    </column>
  </row>
</column>
