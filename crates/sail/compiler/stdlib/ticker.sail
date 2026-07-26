<!--
  <ticker> — infinite linear marquee. Host overflow Style; track is a nowrap
  row. Duration/reverse drive use:marquee (fx:loop) — not contracts PE.
-->

<scope>
  props: {
    duration: 'fast' | 'normal' | 'slow' = 'normal'
    reverse = false
  }

  const host = style.overflow(hidden)
  const track = style.w(max)
</scope>

<div sx={host}>
  <row nowrap class="track" use:marquee sx={track}>
    <span class="segment"><slot></slot></span>
    <span class="segment" aria-hidden="true"><slot></slot></span>
  </row>
</div>
