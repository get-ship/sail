<!-- Sidebar-left row (grow/shrink/nowrap). -->
<column style({ size({ w: 100%, min-h: screen }) })>
  <row nowrap align="stretch" style({ grow(1).size({ w: 100% }) })>
    <column style({ shrink(0).size({ w: 48 }).pad(3) })>
      <span>Side</span>
    </column>
    <column style({ grow(1).size({ min-w: 0 }).pad(3) })>
      <span>Main content</span>
    </column>
  </row>
</column>
