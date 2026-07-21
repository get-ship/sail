<!--
  <tabs> — tab strip + panels. use:tabs owns behavior.

  Host paint: column+gap+min-w layout. Residue: size channels, orientation row,
  sticky list, pill/enclosed variant matrices (child-keyed).
-->

<scope>
  props: {
    active = ''
    variant: 'line' | 'pill' | 'enclosed' = 'line'
    size: 'sm' | 'md' | 'lg' = 'md'
    tone = 'default'
    orientation: 'horizontal' | 'vertical' = 'horizontal'
    sticky = false
  }
</scope>

<style>
  .sail-tabs {
    /* size vocabulary (md base) — read by <tab> */
    --tab-fs: var(--text-sm);
    --tab-py: var(--space-2);
    --tab-px: var(--space-3);
  }
  .sail-tabs[size='sm'] { --tab-fs: var(--text-xs); --tab-py: var(--space-1_5); --tab-px: var(--space-2_5); }
  .sail-tabs[size='lg'] { --tab-fs: var(--text-base); --tab-py: var(--space-2_5); --tab-px: var(--space-4); }

  .sail-tabs[orientation='vertical'] { flex-direction: row; align-items: flex-start; }

  .sail-tabs[sticky] .sail-tab-list {
    position: sticky;
    top: 0;
    z-index: var(--z-sticky, 100);
    background: var(--background);
  }

  /* pill: the strip is a muted track, the active tab a raised chip */
  .sail-tabs[variant='pill'] .sail-tab-list {
    gap: var(--space-1);
    padding: var(--space-1);
    border: 0;
    border-radius: var(--radius-md);
    background: var(--muted);
  }
  .sail-tabs[variant='pill'] .sail-tab {
    border-bottom: 0;
    border-radius: var(--radius-sm);
  }
  .sail-tabs[variant='pill'] .sail-tab[aria-selected='true'] {
    color: var(--foreground);
    background: var(--background);
    box-shadow: var(--shadow-xs);
  }

  /* enclosed: bordered folder tabs joined to the panel */
  .sail-tabs[variant='enclosed'] .sail-tab-list { gap: 0; }
  .sail-tabs[variant='enclosed'] .sail-tab {
    border: 1px solid transparent;
    border-bottom: 0;
    border-radius: var(--radius-md) var(--radius-md) 0 0;
  }
  .sail-tabs[variant='enclosed'] .sail-tab[aria-selected='true'] {
    color: var(--foreground);
    border-color: var(--border);
    background: var(--background);
    margin-bottom: -1px;
  }
</style>

<column gap="3" sx="min-w-0" use:tabs>
  <slot></slot>
</column>
