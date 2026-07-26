<!--
  <ticker> — infinite linear marquee. Host overflow Style; track is a nowrap
  row. Duration/reverse drive use:marquee (motion loop) — not contracts PE.
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
  <row nowrap use:marquee sx={track}>
    <span><slot></slot></span>
    <span aria-hidden="true"><slot></slot></span>
  </row>
</div>
