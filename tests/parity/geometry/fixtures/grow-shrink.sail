<!-- Flex grow/shrink participation (grow is 0|1 only). -->
<row nowrap gap="2" style({ box: w(full).pad(2) })>
  <column style({ box: grow(1).pad(2) })>
    <span>Grow A</span>
  </column>
  <column style({ box: grow(1).pad(2) })>
    <span>Grow B</span>
  </column>
  <column style({ box: shrink(0).w(32).pad(2) })>
    <span>Fixed</span>
  </column>
</row>
