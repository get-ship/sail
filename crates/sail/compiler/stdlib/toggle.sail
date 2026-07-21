<!--
  <toggle> — first:/last: join + variant/orientation CVA (stamped by group).
-->
<scope>
  props: {
    value = ''
    disabled = false
    variant: {
      outline: 'rounded-none -ms-px first:ms-0 first:rounded-l-md last:rounded-r-md',
      ghost: 'border-transparent rounded-md ms-0 me-1 last:me-0',
      segmented: 'border-transparent rounded-sm ms-0 me-1 last:me-0',
    }
    orientation: {
      horizontal: '',
      vertical: '',
    }
  }
  compound: [
    { when: { orientation: 'vertical', variant: 'outline' }, sx: 'rounded-none first:rounded-t-md last:rounded-b-md -mt-px first:mt-0 ms-0 me-0' },
    { when: { variant: 'ghost' }, sx: 'border-transparent rounded-md ms-0 mt-0 me-1 last:me-0' },
    { when: { variant: 'segmented' }, sx: 'border-transparent rounded-sm ms-0 mt-0 me-1 last:me-0' },
    { when: { orientation: 'vertical', variant: 'ghost' }, sx: 'border-transparent rounded-md ms-0 mt-0 me-0 mb-1 last:mb-0' },
    { when: { orientation: 'vertical', variant: 'segmented' }, sx: 'border-transparent rounded-sm ms-0 mt-0 me-0 mb-1 last:mb-0' },
  ]
</scope>
<button type="button" sx="relative font-body font-medium leading-none text-foreground bg-background cursor-pointer whitespace-nowrap select-none h-(--tg-h) px-(--tg-px) border border-border rounded-none -ms-px first:ms-0 first:rounded-l-md last:rounded-r-md tap-highlight-none transition-colors duration-fast ease-out hover:bg-muted pressed:bg-foreground pressed:text-background pressed:border-foreground pressed:z-1 disabled:opacity-50 disabled:cursor-not-allowed focus:outline-solid focus:outline-2 focus:outline-ring focus:z-2">
  <slot></slot>
</button>
