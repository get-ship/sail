<!--
  <tab-list> — the tab strip inside <tabs>. Rows the <tab>s (columns when the
  parent is [orientation='vertical']) with a hairline rail. role=tablist. Bare
  tag lowers to a layout row (`.sail-tab-list`).

  Host paint: layout owns row+gap+align; sx owns bottom rail. Vertical
  orientation (column + side rail) stays parent-keyed residue.
-->

<scope>
</scope>

<style>
  .sail-tabs[orientation='vertical'] .sail-tab-list {
    flex-direction: column;
    align-items: stretch;
    border-bottom: 0;
    border-right: 1px solid var(--border);
  }
</style>

<row align="stretch" gap="1" role="tablist" sx="border-b border-border"><slot></slot></row>
