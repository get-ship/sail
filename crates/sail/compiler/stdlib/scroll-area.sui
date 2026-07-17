<!--
  <scroll-area> — L1 scroll container (element mode) with thin, themed,
  overlay-style scrollbars. Constrain it (height/max-height via sx) and it
  scrolls its overflow. Bare tag lowers to <div class="sail-scroll-area">.

    <scroll-area sx="h-60 p-4"> … long content … </scroll-area>

  orientation vertical (default axis auto) / horizontal / both.
-->

<scope>
  props: {
    orientation: 'vertical' | 'horizontal' | 'both' = 'vertical'
  }
</scope>

<style>
  .sail-scroll-area {
    overflow: auto;
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
    border-radius: var(--radius-full);
    border: 3px solid transparent;
    background-clip: padding-box;
  }
  .sail-scroll-area:hover::-webkit-scrollbar-thumb { background: var(--muted-foreground); background-clip: padding-box; }
  .sail-scroll-area::-webkit-scrollbar-thumb:hover { background: var(--foreground); background-clip: padding-box; }
</style>

<div><slot></slot></div>
