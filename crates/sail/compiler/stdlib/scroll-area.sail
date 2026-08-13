<!--
  <scroll-area> — overflow axes + standard scrollbar pack on host Style.
  Thumb from component.scroll-area tokens. WebKit scrollbar pseudos
  (width/height, thumb radius/border/bg-clip) have no Style setters yet —
  standard scrollbar-width/color only.
-->

<scope>
  props: {
    orientation: 'vertical' | 'horizontal' | 'both' = 'vertical'
  }

  style base = {paint: scrollbarThin().scrollbarThumb(scroll-area.thumb).scrollbarTrack(transparent), box: overflow({ overscroll: contain })}
  style vertical = {box: overflow({ x: hidden, y: auto })}
  style horizontal = {box: overflow({ x: auto, y: hidden })}
  style both = {box: overflow({ all: auto })}
</scope>

<div
  style(base, {when: {hover: {paint: scrollbarThumb(border-strong)}}, match: {
      orientation: {
        vertical: vertical,
        horizontal: horizontal,
        both: both,
      },
    }})
>
  <slot></slot>
</div>
