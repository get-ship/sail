<!--
  <ticker> — infinite linear marquee band. Host clips; track is a cluster
  row with no gap so the two copies read as one continuous strip.
  Duration/reverse drive use:marquee (motion loop).

  Track is two identical copies side-by-side. Compile-time slot fill
  projects author children into BOTH default slots (string duplication —
  not an empty second <slot>). The second span is hidden="true"
  (aria-hidden, still laid out) so the seam copy is decorative. Marquee
  translates the track by -50% (or +50% reverse). w(max) on the track is
  required: without it the row is host-width and -50% slides a half-host
  empty gap, then the loop jumps all-at-once.

  Cluster (not a plain nowrap row): `.sail-row > * { min-width: 0 }` would
  crush both copies into the host so percent translate never moves past
  one viewport. Cluster kids stay intrinsic (`min-width: auto; flex: 0 0
  auto`). gap=0: the default row gap would flash at the seam.

  Defaults paint a quiet band and nowrap the slot so a bare <p> is enough.
-->

<scope>
  props: {
    duration: 'fast' | 'normal' | 'slow' = 'normal'
    reverse = false
  }

  style host = {overflow({ all: hidden }).size({ w: 100%, min-w: 0 }).pad({ y: 2 }).bg(muted).fg(foreground).type({ size: sm, wrap: nowrap })}
  // Track must size to dual content (max-content). Cluster on the row
  // defeats the child min-w:0 crush; this width hugs the two copies.
  style track = {size({ w: max })}
  style cell = {size({ min-w: max }).pad({ x: 4 }).type({ wrap: nowrap })}
</scope>

<div style(host)>
  <row cluster gap="0" use:marquee style(track)>
    <span style(cell, {shrink(0)})><slot></slot></span>
    <span hidden="true" style(cell, {shrink(0)})><slot></slot></span>
  </row>
</div>
