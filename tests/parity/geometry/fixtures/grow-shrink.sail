<!-- Flex grow/shrink participation (grow is 0|1 only). -->
<row nowrap gap="2" style({ size({ w: 100% }).pad(2) })>
  <column style({ grow(1).pad(2) })>
    <span>Grow A</span>
  </column>
  <column style({ grow(1).pad(2) })>
    <span>Grow B</span>
  </column>
  <column style({ shrink(0).size({ w: 32 }).pad(2) })>
    <span>Fixed</span>
  </column>
</row>
