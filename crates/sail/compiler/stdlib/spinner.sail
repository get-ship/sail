<!--
  <spinner> — L1 loading indicator. Host paint: vertical-align on sx.
  Size/tone channels + .ring geometry + --animate-spin stay (child anatomy / fx).
  centered remains attr CSS (display/margin block).
-->

<scope>
  props: {
    size: 'xs' | 'sm' | 'md' | 'lg' = 'md'
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' = 'default'
    centered = false
  }
</scope>

<span class="spin" role="status" aria-label="Loading" sx="align-middle">
  <span class="ring" aria-hidden="true"></span>
</span>
