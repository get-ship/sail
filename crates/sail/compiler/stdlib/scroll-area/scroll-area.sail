<!--
  <scroll-area> — overflow axes + standard scrollbar pack on host Style.
  Thumb from border. WebKit scrollbar pseudos
  (width/height, thumb radius/border/bg-clip) have no Style setters yet —
  standard scrollbar-width/color only.
-->

<scope>
  props: {
    orientation: 'vertical' | 'horizontal' | 'both' = 'vertical'
  }

  style base = {scrollbarThin().scrollbarThumb(border).scrollbarTrack(transparent).overflow({ overscroll: contain })}
  style vertical = {overflow({ x: hidden, y: auto })}
  style horizontal = {overflow({ x: auto, y: hidden })}
  style both = {overflow({ all: auto })}
</scope>

<div
  label="Scroll"
  style(base, {when: {hover: scrollbarThumb(border-strong)}, match: { orientation: { vertical: vertical, horizontal: horizontal, both: both } }})
>
  <slot></slot>
</div>
