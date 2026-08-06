<!-- PARITY-1: flex grow/shrink participation (grow is 0|1 only). -->
<row nowrap w="full" gap="2">
  style({ box: pad(2) })

  <column grow>
    style({ box: pad(2) })

    <span>Grow A</span>
  </column>
  <column grow>
    style({ box: pad(2) })

    <span>Grow B</span>
  </column>
  <column shrink="0" w="32">
    style({ box: pad(2) })

    <span>Fixed</span>
  </column>
</row>
