<!--
  <tooltip> — hover/focus text bubble. use:tipDescribe supplies per-instance
  anchor geometry vars (placement offsets + arrow insets), content text, and Escape.

  Paint (one path):
    host sx=         — group (hover/focus-within → bubble) + relative positioning context
    child sx=        — bubble chrome + absolute geometry via (--tt-*) vars + arrow ::after
  No contracts PE matrix.
-->

<scope>
  props: {
    placement: 'top' | 'bottom' | 'left' | 'right' = 'top'
    content = ''
  }
</scope>

<span use:tipDescribe sx="group relative">
  <slot></slot>
  <span
    class="bubble"
    role="tooltip"
    sx="absolute top: (--tt-top) bottom: (--tt-bottom) left: (--tt-left) right: (--tt-right) translate-x: (--tt-tx) z: (--z-popover) opacity: 0 transition: opacity duration: fast ease: out group-hover:opacity: 100 group-focus-within:opacity: 100 py: 1 px: 2 bg: foreground color: background font-family: body font-size: xs font-weight: medium rounded: md shadow: md whitespace: nowrap pointer-events: none after:content: empty after:absolute after:w: 2 after:h: 2 after:bg: foreground after:rotate: 45 after:top: (--tt-arrow-top) after:bottom: (--tt-arrow-bottom) after:left: (--tt-arrow-left) after:right: (--tt-arrow-right) after:ml: (--tt-arrow-ml) after:mt: (--tt-arrow-mt)"
  ></span>
</span>
