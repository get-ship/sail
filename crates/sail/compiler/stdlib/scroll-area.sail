<!--
  <scroll-area> — themed overlay scrollbars. Constrain via sx (h-*/max-h-*).

  Host paint: overflow-auto on sx. Residue: overscroll/scrollbar-* + webkit
  pseudos + orientation overflow axes.
-->

<scope>
  props: {
    orientation: 'vertical' | 'horizontal' | 'both' = 'vertical'
  }
</scope>

<div sx="overflow-auto"><slot></slot></div>
