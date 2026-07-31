<!--
  <ticker> — infinite linear marquee. Host overflow Style; track is a nowrap
  row. Duration/reverse drive use:marquee (motion loop) — not contracts PE.

  Track is two identical copies side-by-side (second is aria-hidden). Marquee
  translates the track by -50% (or +50% reverse) so the seam is invisible.
  w(max) on the track is required: without it the row collapses to host width
  and percent translate moves nothing useful.
-->

<scope>
  props: {
    duration: 'fast' | 'normal' | 'slow' = 'normal'
    reverse = false
  }

  const host = style.overflow(hidden).w(full).minW(0)
  // Track must size to dual content (max-content). Row default min-w:0 on
  // children would collapse both copies into the host width and kill the loop.
  const track = style.w(max)
  const cell = style.whitespace(nowrap).minW(max)
</scope>

<div sx={host}>
  <row nowrap use:marquee sx={track}>
    <span shrink="0" sx={cell}><slot></slot></span>
    <span shrink="0" aria-hidden="true" sx={cell}><slot></slot></span>
  </row>
</div>
