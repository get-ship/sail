<scope>
  props: {
    kind: 'stack' | 'mobile' | 'tablet' | 'desktop' = 'stack'
  }
  style box = { size({ w: 16, h: 16 }), bg(primary), when: { tablet: bg(accent), desktop: bg(muted) } }
  style mobile = { size({ w: 16, h: 16 }), bg(muted), when: { mobile: bg(primary) } }
  style tablet = { size({ w: 16, h: 16 }), bg(muted), when: { tablet: bg(accent) } }
  style desktop = { size({ w: 16, h: 16 }), bg(muted), when: { desktop: bg(primary) } }
</scope>
<column style({ match: { kind: { stack: box, mobile: mobile, tablet: tablet, desktop: desktop } } })></column>
