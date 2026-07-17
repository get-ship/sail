<!--
  <tabs> — L1 tab strip (element mode). Compose a <tab-list> of <tab>s with
  <tab-panel> children; behavior rides use:tabs (roving tabindex, arrow-key nav
  per orientation, Home/End, Enter/Space, panel show/hide, `sail-tab-change`).
  Bare tag lowers to <div class="sail-tabs" use:tabs>.

    <tabs active="overview">
      <tab-list>
        <tab name="overview">Overview</tab>
        <tab name="activity">Activity</tab>
      </tab-list>
      <tab-panel name="overview">…</tab-panel>
      <tab-panel name="activity">…</tab-panel>
    </tabs>

  variant line (default, underline) / pill (filled active in a muted track) /
  enclosed (folder tabs). size sm/md/lg. orientation horizontal/vertical. tone
  drives the active accent (the global channel). sticky pins the strip.
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
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    min-width: 0;
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

<div use:tabs>
  <slot></slot>
</div>
