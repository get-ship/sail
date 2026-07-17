<!--
  <tooltip> — L1 language element (element mode). Wraps a trigger (default slot)
  and shows a text bubble on hover/focus, positioned with CSS Anchor Positioning.
  Show/hide + placement are PURE CSS (the skin); the behavior layer (`use:tooltip`,
  packages/sail/runtime/src/actions/tooltip.ts) supplies only a per-instance anchor
  name, the bubble text (from `content`) + aria-describedby wiring, and Escape.

  Faithful to the feat/next sail-tooltip baseline for placement/positioning/arrow.
  L1 divergence: the bubble body is a `content` STRING prop (not a named `content`
  slot — element mode projects only the default slot). Text tooltips are the
  common case; a rich-content tooltip stays an L2 component until named-slot
  projection lands.
-->

<scope>
  props: {
    placement: 'top' | 'bottom' | 'left' | 'right' = 'top'
    content = ''
  }
</scope>

<style>
  .sail-tooltip {
    display: inline-flex;
    anchor-name: var(--sail-tooltip-anchor);
  }

  .bubble {
    --tt-arrow-shift: -3.5px;
    position: fixed;
    position-anchor: var(--sail-tooltip-anchor);
    padding: var(--space-1) var(--space-2);
    background: var(--foreground);
    color: var(--background);
    font-family: var(--font-body);
    font-size: var(--text-xs);
    font-weight: var(--font-weight-medium);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-md);
    z-index: var(--z-popover);
    white-space: nowrap;
    pointer-events: none;
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
    /* Placement + centering via position-area: the box is laid into the region
       on the named side of the anchor and centered on the perpendicular axis;
       `margin` opens the gap toward the trigger. flip-* only kicks in on a real
       viewport clip (we do NOT reorder by most-room, which would ignore the
       author's chosen side). */
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

<span use:tooltip><slot></slot><span class="bubble" role="tooltip"></span></span>
