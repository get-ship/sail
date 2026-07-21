<!--
  <tab> — paint + stamped variant CVA for pill/enclosed. Pad via --tab-* channels.
-->
<scope>
  props: {
    name = ''
    disabled = false
    variant: {
      line: '',
      pill: 'border-b-0 rounded-sm selected:text-foreground selected:bg-background selected:shadow-xs selected:border-b-transparent',
      enclosed: 'border border-transparent border-b-0 rounded-t-md selected:text-foreground selected:border-border selected:bg-background selected:-mb-px',
    }
  }
</scope>
<button type="button" sx="font-body font-medium leading-none text-muted-foreground bg-transparent cursor-pointer whitespace-nowrap select-none text-(--tab-fs) py-(--tab-py) px-(--tab-px) border-0 border-b-2 border-b-transparent rounded-t-sm tap-highlight-none transition-colors duration-fast ease-out hover:text-foreground selected:text-foreground selected:font-semibold selected:border-b-tone focus:outline-solid focus:outline-ring-width focus:outline-tone-ring focus:outline-offset-ring-inset disabled:opacity-50 disabled:cursor-not-allowed">
  <slot></slot>
</button>
