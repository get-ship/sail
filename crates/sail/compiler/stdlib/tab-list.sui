<!--
  <tab-list> — the tab strip inside <tabs>. Rows the <tab>s (columns when the
  parent is [orientation='vertical']) with a hairline rail. role=tablist. Bare
  tag lowers to <div class="sail-tab-list" role="tablist">.
-->

<scope>
</scope>

<style>
  .sail-tab-list {
    display: flex;
    align-items: stretch;
    gap: var(--space-1);
    border-bottom: 1px solid var(--border);
  }
  .sail-tabs[orientation='vertical'] .sail-tab-list {
    flex-direction: column;
    align-items: stretch;
    border-bottom: 0;
    border-right: 1px solid var(--border);
  }
</style>

<div role="tablist"><slot></slot></div>
