<scope>
  props: {
    tone: 'default' | 'primary' | 'muted' = 'default'
    interactive = false
  }
</scope>

<div sx="bg: surface-1 rounded: lg border: border match(tone){primary: border: tone-border; muted: border: subtle} phase(hover){shadow: hover} compound(interactive:true, tone:primary){cursor: pointer} px: 5 py: 4">
  <div class="header" sx="phase(empty){hidden} font-weight: semibold">
    <slot name="header"></slot>
  </div>
  <slot></slot>
</div>
