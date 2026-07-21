<!--
  <switch> — native checkbox restyled as sliding toggle. No controller.

  Host paint: cursor/align/m-0 on sx. Residue: appearance:none track/thumb
  geometry, size/tone channels, checked/hover/disabled/focus, forced-colors,
  reduced-motion.
-->

<scope>
  props: {
    checked = false
    disabled = false
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    size: 'sm' | 'md' | 'lg' = 'md'
  }
</scope>

<style>
  .sail-switch {
    appearance: none;
    -webkit-appearance: none;
    position: relative;
    flex-shrink: 0;
    box-sizing: border-box;
    background: var(--muted);
    border: 1px solid var(--border-strong);
    border-radius: var(--shape-pill);
    -webkit-tap-highlight-color: transparent;
    --tone: var(--primary);
    --tone-on-fg: var(--primary-foreground);
    --sw-w: var(--control-h-sm);
    --sw-h: var(--icon-size-md);
    --sw-thumb: var(--space-3);
    width: var(--sw-w);
    height: var(--sw-h);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
  }
  .sail-switch[size='sm'] { --sw-w: var(--control-h-xs); --sw-h: var(--icon-size-sm); --sw-thumb: var(--space-2_5); }
  .sail-switch[size='lg'] { --sw-w: var(--space-7);      --sw-h: var(--icon-size-lg); --sw-thumb: var(--icon-size-sm); }

  .sail-switch[tone='primary']     { --tone: var(--primary);          --tone-on-fg: var(--primary-foreground); }
  .sail-switch[tone='info']        { --tone: var(--info);             --tone-on-fg: var(--info-foreground); }
  .sail-switch[tone='success']     { --tone: var(--success);          --tone-on-fg: var(--success-foreground); }
  .sail-switch[tone='warning']     { --tone: var(--warning);          --tone-on-fg: var(--warning-foreground); }
  .sail-switch[tone='destructive'] { --tone: var(--destructive);      --tone-on-fg: var(--destructive-foreground); }
  .sail-switch[tone='accent']      { --tone: var(--accent);           --tone-on-fg: var(--accent-foreground); }
  .sail-switch[tone='muted']       { --tone: var(--muted-foreground); --tone-on-fg: var(--background); }

  .sail-switch::before {
    content: '';
    position: absolute;
    top: 50%;
    left: 1px;
    width: var(--sw-thumb);
    height: var(--sw-thumb);
    background: var(--surface-1);
    border-radius: 50%;
    box-shadow: var(--shadow-sm);
    transform: translateY(-50%);
    transition:
      left var(--duration-fast) var(--ease-out),
      background-color var(--duration-fast) var(--ease-out);
  }

  .sail-switch:not(:disabled):not(:checked):hover { background: var(--hover-bg); }

  .sail-switch:checked {
    background: var(--tone);
    border-color: var(--selected-border);
    box-shadow: var(--shadow-xs);
  }
  .sail-switch:checked::before {
    left: calc(100% - var(--sw-thumb) - 1px);
    background: var(--tone-on-fg);
  }

  .sail-switch:focus-visible {
    --ring-color: var(--tone);
    outline: var(--focus-ring);
    outline-offset: var(--focus-ring-offset);
  }

  .sail-switch:disabled {
    cursor: not-allowed;
    background: var(--muted);
    border-color: var(--border);
    filter: saturate(0.6);
    box-shadow: none;
  }
  .sail-switch:disabled::before { box-shadow: var(--shadow-xs); }

  @media (forced-colors: active) {
    .sail-switch { border-color: CanvasText; }
    .sail-switch:checked { background: Highlight; }
    .sail-switch:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
  @media (prefers-reduced-motion: reduce) {
    .sail-switch, .sail-switch::before { transition: none; }
  }
</style>

<input type="checkbox" role="switch" sx="inline-block align-middle m-0 cursor-pointer" />
