<!--
  <separator> — native <hr>. Solid horizontal PE on host sx; orientation +
  dashed via CVA/compound. --separator-line channel writes live in contracts
  (PE default + branded tone overrides).
-->

<scope>
  props: {
    // first key = default = PE horizontal
    orientation: {
      // h/w 1px via --border-width (no h-px token on size scale)
      horizontal: 'block w: full h: (--border-width) my: 2',
      vertical: 'inline-block self: stretch w: (--border-width) h: full mx: 2 my: 0',
    }
    tone: 'default' | 'muted' | 'primary' | 'destructive' = 'default'
    // first key = default = PE solid
    variant: {
      solid: 'bg: (--separator-line) border: 0 p: 0',
      dashed: 'bg: transparent border: 0 p: 0',
    }
  }
  compound: [
    { when: { variant: 'dashed', orientation: 'horizontal' }, sx: 'h: 0 border-t border: dashed border-t: (--separator-line)' },
    { when: { variant: 'dashed', orientation: 'vertical' }, sx: 'w: 0 border-l border: dashed border-l: (--separator-line)' },
  ]
</scope>

<hr sx="block w: full h: (--border-width) my: 2 border: 0 p: 0 bg: (--separator-line)">
