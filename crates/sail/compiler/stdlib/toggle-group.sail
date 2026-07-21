<!--
  <toggle-group> — segmented toggle group. use:toggleGroup owns behavior.

  Host paint: inline row layout (gap 0 default). Residue: size channels,
  orientation column, outline join radii, ghost/segmented variant matrices.
-->

<scope>
  props: {
    type: 'single' | 'multiple' = 'single'
    value = ''
    variant: 'outline' | 'ghost' | 'segmented' = 'outline'
    size: 'sm' | 'md' | 'lg' = 'md'
    orientation: 'horizontal' | 'vertical' = 'horizontal'
  }
</scope>

<style>
  .sail-toggle-group {
    
    /* size vocabulary (md base) — read by <toggle> */
    --tg-fs: var(--text-sm);
    --tg-h: var(--control-h-md);
    --tg-px: var(--space-3);
  }
  .sail-toggle-group[size='sm'] { --tg-fs: var(--text-xs); --tg-h: var(--control-h-sm); --tg-px: var(--space-2_5); }
  .sail-toggle-group[size='lg'] { --tg-fs: var(--text-base); --tg-h: var(--control-h-lg); --tg-px: var(--space-4); }
  .sail-toggle-group[orientation='vertical'] { flex-direction: column; }

  /* outline (default): joined bordered cells, radius on the ends */
  .sail-toggle-group > .sail-toggle:not(:first-child) { margin-inline-start: -1px; }
  .sail-toggle-group > .sail-toggle:first-child { border-radius: var(--radius-md) 0 0 var(--radius-md); }
  .sail-toggle-group > .sail-toggle:last-child { border-radius: 0 var(--radius-md) var(--radius-md) 0; }
  .sail-toggle-group > .sail-toggle:not(:first-child):not(:last-child) { border-radius: 0; }
  .sail-toggle-group[orientation='vertical'] > .sail-toggle:not(:first-child) { margin-inline-start: 0; margin-top: -1px; }
  .sail-toggle-group[orientation='vertical'] > .sail-toggle:first-child { border-radius: var(--radius-md) var(--radius-md) 0 0; }
  .sail-toggle-group[orientation='vertical'] > .sail-toggle:last-child { border-radius: 0 0 var(--radius-md) var(--radius-md); }

  /* ghost: no frame, spaced pills */
  .sail-toggle-group[variant='ghost'] { gap: var(--space-1); }
  .sail-toggle-group[variant='ghost'] > .sail-toggle { margin: 0; border-color: transparent; border-radius: var(--radius-md); }

  /* segmented: muted track, radiused cells */
  .sail-toggle-group[variant='segmented'] {
    padding: var(--space-1);
    background: var(--muted);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    gap: var(--space-1);
  }
  .sail-toggle-group[variant='segmented'] > .sail-toggle { margin: 0; border-color: transparent; border-radius: var(--radius-sm); }
</style>

<row inline gap="0" use:toggleGroup>
  <slot></slot>
</row>
