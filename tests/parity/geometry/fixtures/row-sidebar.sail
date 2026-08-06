<!-- PARITY-1: sidebar-left row (grow/shrink/nowrap). -->
<column w="full">
  style({ box: minH(screen) })

  <row grow nowrap align="stretch" w="full">
    <column shrink="0" w="48">
      style({ box: pad(3) })

      <span>Side</span>
    </column>
    <column grow min-w="0">
      style({ box: pad(3) })

      <span>Main content</span>
    </column>
  </row>
</column>
