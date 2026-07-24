<!--
  <ticker> — infinite linear marquee. Host overflow is sx; track is a nowrap
  row. Duration/reverse drive use:marquee (motion:loop) — not contracts PE.
-->

<scope>
  props: {
    duration: 'fast' | 'normal' | 'slow' = 'normal'
    reverse = false
  }
</scope>

<div sx="overflow: hidden">
  <row nowrap class="track" use:marquee sx="w: max">
    <span class="segment"><slot></slot></span>
    <span class="segment" aria-hidden="true"><slot></slot></span>
  </row>
</div>
