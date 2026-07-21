<!--
  <spinner> — loading indicator. Host align + PE size on sx; centered via
  compound. Size/tone channels for the ring + .ring geometry + spin fx are
  contracts (two-tone border anatomy). PE --tone-fg muted default in contracts.
-->

<scope>
  props: {
    // first key = default = PE md (box only; stroke width stays contracts)
    size: {
      md: 'size-icon-size-md',
      xs: 'size-icon-size-2xs',
      sm: 'size-icon-size-sm',
      lg: 'size-icon-size-xl',
    }
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' = 'default'
    centered = false
  }
  compound: [
    { when: { centered: true }, sx: 'block my-8 mx-auto' },
  ]
</scope>

<span class="spin" role="status" aria-label="Loading" sx="inline-block align-middle size-icon-size-md">
  <span class="ring" aria-hidden="true" sx="block size-full"></span>
</span>
