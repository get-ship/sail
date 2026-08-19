<!--
  Look: a painted box per filter slot.
  Spelling: filter({ blur: 4 }) · filter({ brightness: 150 }) · filter({ contrast: 200 }) · filter({ grayscale: 100 }) · filter({ hue: 180 }) · filter({ invert: 100 }) · filter({ sepia: 100 }) · filter({ saturate: 200 }) · filter({ backdrop: filter-backdrop }) · filter({ blend: multiply }) · filter({ opacity: 50 })
  Owner: language (filter slot table)
-->
<column gap="6">
  <l0-case look="blur — 4px smear on a primary bar" spell="filter blur: 4">
    <l0-filter-face kind="blur"></l0-filter-face>
  </l0-case>
  <l0-case look="brightness — 150 on a primary bar" spell="filter brightness: 150">
    <l0-filter-face kind="brightness"></l0-filter-face>
  </l0-case>
  <l0-case look="contrast — 200 on a primary bar" spell="filter contrast: 200">
    <l0-filter-face kind="contrast"></l0-filter-face>
  </l0-case>
  <l0-case look="grayscale — 100 on an accent bar" spell="filter grayscale: 100">
    <l0-filter-face kind="grayscale"></l0-filter-face>
  </l0-case>
  <l0-case look="hue — 180deg on a primary bar" spell="filter hue: 180">
    <l0-filter-face kind="hue"></l0-filter-face>
  </l0-case>
  <l0-case look="invert — 100 on a primary bar" spell="filter invert: 100">
    <l0-filter-face kind="invert"></l0-filter-face>
  </l0-case>
  <l0-case look="sepia — 100 on a primary bar" spell="filter sepia: 100">
    <l0-filter-face kind="sepia"></l0-filter-face>
  </l0-case>
  <l0-case look="saturate — 200 on an accent bar" spell="filter saturate: 200">
    <l0-filter-face kind="saturate"></l0-filter-face>
  </l0-case>
  <l0-case look="backdrop — frosted overlay on a primary bar" spell="filter backdrop: filter-backdrop">
    <stack>
      <l0-filter-face kind="under"></l0-filter-face>
      <l0-filter-face kind="backdrop"></l0-filter-face>
    </stack>
  </l0-case>
  <l0-case look="blend — accent multiply over a primary bar" spell="filter blend: multiply">
    <stack>
      <l0-filter-face kind="under"></l0-filter-face>
      <l0-filter-face kind="blend"></l0-filter-face>
    </stack>
  </l0-case>
  <l0-case look="opacity — 50 on a primary bar" spell="filter opacity: 50">
    <l0-filter-face kind="opacity"></l0-filter-face>
  </l0-case>
</column>
