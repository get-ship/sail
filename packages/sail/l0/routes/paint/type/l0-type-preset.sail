<scope>
  props: {
    kind: 'body' | 'body-sm' | 'heading' | 'label' | 'mono' | 'mono-sm' | 'display' | 'display-sm' = 'body'
  }
  style body = { pad(2), fg(primary), type(body) }
  style bodySm = { pad(2), fg(primary), type(body-sm) }
  style heading = { pad(2), fg(primary), type(heading) }
  style label = { pad(2), fg(primary), type(label) }
  style mono = { pad(2), fg(primary), type(mono) }
  style monoSm = { pad(2), fg(primary), type(mono-sm) }
  style display = { pad(2), fg(primary), type(display) }
  style displaySm = { pad(2), fg(primary), type(display-sm) }
</scope>
<column style({ match: { kind: { body: body, 'body-sm': bodySm, heading: heading, label: label, mono: mono, 'mono-sm': monoSm, display: display, 'display-sm': displaySm } } })>
  <slot></slot>
</column>
