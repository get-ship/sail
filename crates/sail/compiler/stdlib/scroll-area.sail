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

  const base = style.overscroll(contain).scrollbarThin().scrollbarThumb(scroll-area-thumb).scrollbarTrack(transparent)
  const vertical = style.overflowX(hidden).overflowY(auto)
  const horizontal = style.overflowX(auto).overflowY(hidden)
  const both = style.overflow(auto)
</scope>
<div>
  style([
  base,
  match(orientation, {
    vertical: vertical,
    horizontal: horizontal,
    both: both,
  }),
  when({
    hover: style.scrollbarThumb(scroll-area-thumb-hover),
  }),
])
<slot></slot></div>
