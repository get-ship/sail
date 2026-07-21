<!--
  <ticker> — L1 stdlib element (element mode). An infinite, linear marquee:
  overflow hidden on the root, a flex track carrying TWO copies of the
  projected content (the second `aria-hidden`), animated left by exactly
  half its own width so the loop seams invisibly.

  duration: 'fast' (16s) | 'normal' (24s, default) | 'slow' (32s).
  reverse: runs the track backwards.

  Host paint: overflow is sx. Track flex + --animate-ticker + reduced-motion
  stay mechanics residue (child anatomy / fx duration axes).
-->

<scope>
  props: {
    duration: 'fast' | 'normal' | 'slow' = 'normal'
    reverse = false
  }
</scope>

<div sx="overflow-hidden">
  <div class="track">
    <span class="segment"><slot></slot></span>
    <span class="segment" aria-hidden="true"><slot></slot></span>
  </div>
</div>
