<!--
  <label> — size CVA; weight channel; .req gate in element-skins anatomy.
-->
<scope>
  props: {
    size: {
      md: 'text-base',
      sm: 'text-sm',
      lg: 'text-sm',
    }
    required = false
  }
</scope>
<label sx="font-body font-(--label-weight) tracking-normal text-foreground cursor-pointer text-base group">
  <slot></slot>
  <span class="req" aria-hidden="true" sx="hidden group-required:inline text-destructive font-bold ms-1">*</span>
</label>
