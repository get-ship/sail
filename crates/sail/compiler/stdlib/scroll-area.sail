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

<style>
  .sail-scroll-area {
    overscroll-behavior: contain;
    scrollbar-width: thin;
    scrollbar-color: var(--border) transparent;
  }
  .sail-scroll-area[orientation='vertical'] { overflow-x: hidden; overflow-y: auto; }
  .sail-scroll-area[orientation='horizontal'] { overflow-x: auto; overflow-y: hidden; }
  .sail-scroll-area[orientation='both'] { overflow: auto; }

  .sail-scroll-area::-webkit-scrollbar { width: var(--space-2_5); height: var(--space-2_5); }
  .sail-scroll-area::-webkit-scrollbar-track { background: transparent; }
  .sail-scroll-area::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: var(--shape-pill);
    border: 3px solid transparent; /* residue — 3px scrollbar thumb inset */
    background-clip: padding-box;
  }
  .sail-scroll-area:hover::-webkit-scrollbar-thumb { background: var(--muted-foreground); background-clip: padding-box; }
  .sail-scroll-area::-webkit-scrollbar-thumb:hover { background: var(--foreground); background-clip: padding-box; }
</style>

<div sx="overflow-auto"><slot></slot></div>
