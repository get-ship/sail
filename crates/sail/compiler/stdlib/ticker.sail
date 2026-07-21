<!--
  <ticker> — infinite linear marquee. Host overflow is sx; track is a nowrap
  row. Animation duration/reverse/reduced-motion pause are contracts (fx on
  child anatomy; no animation sx primitive).
-->

<scope>
  props: {
    duration: 'fast' | 'normal' | 'slow' = 'normal'
    reverse = false
  }
</scope>

<div sx="overflow-hidden">
  <row nowrap class="track" sx="w-max">
    <span class="segment"><slot></slot></span>
    <span class="segment" aria-hidden="true"><slot></slot></span>
  </row>
</div>
