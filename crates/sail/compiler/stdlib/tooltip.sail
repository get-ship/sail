<!--
  <tooltip> — hover/focus text bubble via CSS Anchor Positioning. use:tooltip
  supplies anchor name + content + Escape.

  Host paint: bubble chrome on sx. Residue: anchor-name/positioning matrix,
  arrow pseudo, opacity show/hide, placement @supports branches.
-->

<scope>
  props: {
    placement: 'top' | 'bottom' | 'left' | 'right' = 'top'
    content = ''
  }
</scope>

<span use:tooltip><slot></slot><span class="bubble" role="tooltip" sx="py-1 px-2 bg-foreground text-background font-body text-xs font-medium rounded-md shadow-md whitespace-nowrap pointer-events-none"></span></span>
