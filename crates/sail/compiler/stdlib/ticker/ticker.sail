<!--
  <ticker> — infinite linear marquee band. Host clips; track is a nowrap
  row with no gap so the two copies read as one continuous strip.
  Duration/reverse drive use:marquee (motion loop).

  Track is two identical copies side-by-side (second is decorative/hidden).
  Marquee translates the track by -50% (or +50% reverse) so the seam is
  invisible. w(max) on the track is required: without it the row collapses to
  host width and percent translate moves nothing useful.

  Defaults paint a quiet band and nowrap the slot so a bare <p> is enough.
-->

<scope>
  props: {
    duration: 'fast' | 'normal' | 'slow' = 'normal'
    reverse = false
  }

  style host = {overflow({ all: hidden }).size({ w: full, min-w: 0 }).pad({ y: 2 }).bg(muted).fg(foreground).type({ size: sm, wrap: nowrap })}
  // Track must size to dual content (max-content). Row default min-w:0 on
  // children would collapse both copies into the host width and kill the loop.
  // gap=0: the default row gap would flash at the seam.
  style track = {size({ w: max })}
  style cell = {size({ min-w: max }).pad({ x: 4 }).type({ wrap: nowrap })}
</scope>

<div style(host)>
  <row nowrap gap="0" use:marquee style(track)>
    <span style(cell, {shrink(0)})><slot></slot></span>
    <span hidden="true" style(cell, {shrink(0)})><slot></slot></span>
  </row>
</div>
