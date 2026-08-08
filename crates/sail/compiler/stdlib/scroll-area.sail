<!--
  <scroll-area> — overflow axes + standard scrollbar pack on host Style.
  Paint: thumb from component.scroll-area tokens
         (cssNames scroll-area-thumb, scroll-area-thumb-hover).

  Blocker: webkit scrollbar pseudo pack (scrollbar:w/h, thumb rounded/border/
  bg-clip) has no Style setters yet — standard scrollbar-width/color only.
-->
<scope>
  props: {
    orientation: 'vertical' | 'horizontal' | 'both' = 'vertical'
  }

  const base = style({ paint: overscroll(contain).scrollbarThin().scrollbarThumb(scroll-area-thumb).scrollbarTrack(transparent) })
  const vertical = style({ box: overflowX(hidden).overflowY(auto) })
  const horizontal = style({ box: overflowX(auto).overflowY(hidden) })
  const both = style({ box: overflow(auto) })
</scope>
<div style(base, { when: { hover: { paint: scrollbarThumb(scroll-area-thumb-hover) } }, match: { orientation: { vertical: vertical, horizontal: horizontal, both: both } } })><slot></slot></div>
