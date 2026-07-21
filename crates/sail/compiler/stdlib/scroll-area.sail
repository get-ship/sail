<!--
  <scroll-area> — standard + webkit scrollbar packs on host sx.
-->
<scope>
  props: {
    orientation: {
      vertical: 'overflow-x-hidden overflow-y-auto',
      horizontal: 'overflow-x-auto overflow-y-hidden',
      both: 'overflow-auto',
    }
  }
</scope>
<div sx="overflow-x-hidden overflow-y-auto overscroll-contain scrollbar-thin scrollbar-thumb-border scrollbar:w-2_5 scrollbar:h-2_5 scrollbar-track:bg-transparent scrollbar-thumb:bg-border scrollbar-thumb:rounded-full scrollbar-thumb:border-3 scrollbar-thumb:border-transparent scrollbar-thumb:bg-clip-padding hover:scrollbar-thumb:bg-muted-foreground"><slot></slot></div>
