<!--
  <ticker> — infinite linear marquee. Host overflow Style; track is a nowrap
  row. Duration/reverse drive use:marquee (motion loop) — not contracts PE.

  Track is two identical copies side-by-side (second is decorative/hidden). Marquee
  translates the track by -50% (or +50% reverse) so the seam is invisible.
  w(max) on the track is required: without it the row collapses to host width
  and percent translate moves nothing useful.
-->

<scope>
  props: {
    duration: 'fast' | 'normal' | 'slow' = 'normal'
    reverse = false
  }

  style host = { box: overflow(hidden).w(full).minW(0) }
  // Track must size to dual content (max-content). Row default min-w:0 on
  // children would collapse both copies into the host width and kill the loop.
  style track = { box: w(max) }
  style cell = { box: minW(max), text: whitespace(nowrap) }
</scope>

<div style(host)>
  <row nowrap use:marquee style(track)>
    <span style(cell, { box: shrink(0) })><slot></slot></span>
    <span hidden="true" style(cell, { box: shrink(0) })><slot></slot></span>
  </row>
</div>
