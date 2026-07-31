<!-- mini-app-routing overlay: root layout with nav chrome -->
<column sx={style.minH(screen)}>
  <row gap="4" align="center" sx={style.padX(4).padY(3).borderB(border)}>
    <span sx={style.fontWeight(semibold)}>Items</span>
    <a href="/">Home</a>
    <a href="/items">List</a>
  </row>
  <!-- route outlet: deck mounts the matched page here -->
  <column grow sx={style.pad(4)}>
    <slot name="route" />
  </column>
</column>
