<!--
  <tab-list> — the tab strip inside <tabs>. Rows the <tab>s (columns when the
  parent is [orientation='vertical']) with a hairline rail. role=tablist. Bare
  tag lowers to a layout row (`.sail-tab-list`).

  Host paint: layout owns row+gap+align; sx owns bottom rail. Vertical
  orientation (column + side rail) stays parent-keyed residue.
-->

<scope>
</scope>

<row align="stretch" gap="1" role="tablist" sx="border-b border-border"><slot></slot></row>
