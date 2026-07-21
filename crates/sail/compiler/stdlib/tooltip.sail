<!--
  <tooltip> — hover/focus text bubble via CSS Anchor Positioning. use:tooltip
  supplies anchor name + content + Escape.

  Host paint: bubble chrome on sx. Residue: anchor-name/positioning matrix,
  arrow pseudo, opacity show/hide, placement @supports branches.
-->

<scope>
  props: {
    placement: 'top' | 'bottom' | 'left' | 'right' = 'top'
    content = ''
  }
</scope>

<style>
  .sail-tooltip {
    anchor-name: var(--sail-tooltip-anchor);
  }

  .bubble {
    --tt-arrow-shift: -3.5px;
    position: fixed;
    position-anchor: var(--sail-tooltip-anchor);
    z-index: var(--z-popover);
    opacity: 0;
    transition: opacity var(--duration-fast) var(--ease-out);
    position-try-fallbacks: flip-block, flip-inline;
  }

  .sail-tooltip:hover .bubble,
  .sail-tooltip:focus-within .bubble { opacity: 1; }

  .bubble::after {
    content: '';
    position: absolute;
    width: 7px;
    height: 7px;
    background: var(--foreground);
    transform: rotate(45deg);
  }

  @supports (anchor-name: --x) {
    .bubble { margin: var(--space-2); }
    .sail-tooltip:not([placement]) .bubble,
    .sail-tooltip[placement='top'] .bubble { position-area: top; }
    .sail-tooltip[placement='bottom'] .bubble { position-area: bottom; }
    .sail-tooltip[placement='left'] .bubble { position-area: left; }
    .sail-tooltip[placement='right'] .bubble { position-area: right; }

    .sail-tooltip:not([placement]) .bubble::after,
    .sail-tooltip[placement='top'] .bubble::after { bottom: calc(-1 * var(--space-1)); left: 50%; margin-left: var(--tt-arrow-shift); }
    .sail-tooltip[placement='bottom'] .bubble::after { top: calc(-1 * var(--space-1)); left: 50%; margin-left: var(--tt-arrow-shift); }
    .sail-tooltip[placement='left'] .bubble::after { right: calc(-1 * var(--space-1)); top: 50%; margin-top: var(--tt-arrow-shift); }
    .sail-tooltip[placement='right'] .bubble::after { left: calc(-1 * var(--space-1)); top: 50%; margin-top: var(--tt-arrow-shift); }
  }

  @supports not (anchor-name: --x) {
    .sail-tooltip { position: relative; }
    .bubble { position: absolute; }
    .sail-tooltip:not([placement]) .bubble,
    .sail-tooltip[placement='top'] .bubble { bottom: calc(100% + var(--space-2)); left: 50%; transform: translateX(-50%); }
    .sail-tooltip:not([placement]) .bubble::after,
    .sail-tooltip[placement='top'] .bubble::after { bottom: calc(-1 * var(--space-1)); left: 50%; margin-left: var(--tt-arrow-shift); }
    .sail-tooltip[placement='bottom'] .bubble { top: calc(100% + var(--space-2)); left: 50%; transform: translateX(-50%); }
    .sail-tooltip[placement='bottom'] .bubble::after { top: calc(-1 * var(--space-1)); left: 50%; margin-left: var(--tt-arrow-shift); }
    .sail-tooltip[placement='left'] .bubble { right: calc(100% + var(--space-2)); top: 50%; transform: translateY(-50%); }
    .sail-tooltip[placement='left'] .bubble::after { right: calc(-1 * var(--space-1)); top: 50%; margin-top: var(--tt-arrow-shift); }
    .sail-tooltip[placement='right'] .bubble { left: calc(100% + var(--space-2)); top: 50%; transform: translateY(-50%); }
    .sail-tooltip[placement='right'] .bubble::after { left: calc(-1 * var(--space-1)); top: 50%; margin-top: var(--tt-arrow-shift); }
  }
</style>

<span use:tooltip><slot></slot><span class="bubble" role="tooltip" sx="py-1 px-2 bg-foreground text-background font-body text-xs font-medium rounded-md shadow-md whitespace-nowrap pointer-events-none"></span></span>
