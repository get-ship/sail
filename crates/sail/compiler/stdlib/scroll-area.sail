<!--
  <scroll-area> — standard + webkit scrollbar packs on host sx.
  Paint: thumb from component.scroll-area tokens
         (cssNames scroll-area-thumb, scroll-area-thumb-hover).
-->
<scope>
  props: {
    orientation: {
      vertical: 'overflow-x: hidden overflow-y: auto',
      horizontal: 'overflow-x: auto overflow-y: hidden',
      both: 'overflow: auto',
    }
  }
</scope>
<div sx="overflow-x: hidden overflow-y: auto overscroll: contain scrollbar-thin scrollbar-thumb: scroll-area-thumb scrollbar: w-2_5 scrollbar: h-2_5 scrollbar-track: bg-transparent scrollbar-thumb: bg-scroll-area-thumb scrollbar-thumb: rounded-full scrollbar-thumb: border-3 scrollbar-thumb: border-transparent scrollbar-thumb: bg-clip-padding hover:scrollbar-thumb: bg-scroll-area-thumb-hover"><slot></slot></div>
