<scope>
  props: {
    kind: 'body' | 'body-sm' | 'heading' | 'label' | 'mono' | 'mono-sm' | 'display' | 'display-sm' = 'body'
  }
  style body = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type(body) }
  style bodySm = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type(body-sm) }
  style heading = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type(heading) }
  style label = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type(label) }
  style mono = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type(mono) }
  style monoSm = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type(mono-sm) }
  style display = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type(display) }
  style displaySm = { size({ w: 16 }), pad(2), bg(surface), radius(sm), fg(primary), type(display-sm) }
</scope>
<column style({ match: { kind: { body: body, 'body-sm': bodySm, heading: heading, label: label, mono: mono, 'mono-sm': monoSm, display: display, 'display-sm': displaySm } } })>
  <slot></slot>
</column>
