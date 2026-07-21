<!--
  <tab-panel name="…"> — content panel for the matching <tab>. Host sx owns
  default pad; vertical orientation parent-keyed pad lives in contracts.
-->

<scope>
  props: {
    name = ''
  }
</scope>

<div sx="pt-4 min-w-0"><slot></slot></div>
