<!--
  <label> — native label root. Size via CVA (PE md on host sx); weight reads
  --label-weight channel; required star gated in contracts.
-->

<scope>
  props: {
    // first key = default = PE md (attr-absent)
    size: {
      md: 'text-base',
      sm: 'text-sm',
      lg: 'text-sm',
    }
    required = false
  }
</scope>

<label sx="font-body tracking-normal text-foreground cursor-pointer text-base">
  <slot></slot>
  <span class="req" aria-hidden="true" sx="text-destructive font-bold ms-1">*</span>
</label>
