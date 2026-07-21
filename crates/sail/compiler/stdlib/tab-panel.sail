<!--
  <tab-panel name="…"> — the content panel for the matching <tab>. use:tabs on
  the parent <tabs> wires role=tabpanel, aria-labelledby, tabindex, and toggles
  `hidden` so only the active panel shows. Bare tag lowers to
  <div class="sail-tab-panel">.
-->

<scope>
  props: {
    name = ''
  }
</scope>

<style>
  .sail-tabs[orientation='vertical'] .sail-tab-panel { padding: 0 0 0 var(--space-4); }
</style>

<div sx="pt-4 min-w-0"><slot></slot></div>
