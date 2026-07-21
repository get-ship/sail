<!--
  <scroll-area> — themed overlay scrollbars. Constrain via sx (h-*/max-h-*).

  Paint: CVA orientation overflow axes + overscroll on host sx.
  Contracts: scrollbar-width/color + webkit scrollbar pseudos.
-->

<scope>
  props: {
    // first key = default = PE vertical
    orientation: {
      vertical: 'overflow-x-hidden overflow-y-auto',
      horizontal: 'overflow-x-auto overflow-y-hidden',
      both: 'overflow-auto',
    }
  }
</scope>

<div sx="overflow-x-hidden overflow-y-auto overscroll-contain"><slot></slot></div>
