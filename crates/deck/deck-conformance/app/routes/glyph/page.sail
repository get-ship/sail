<!-- Inline SVG paint via currentColor + Style fg. -->

<scope>
  style page = { box: pad(6).maxW(breakpoint-tablet).mx(auto) }
  style title = { paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) }
  style icon = { paint: fg(foreground) }
</scope>

<column gap="2" style(page)>
  <h1 style(title)>Glyph</h1>
  <svg viewBox="0 0 24 24" width="24" height="24" style(icon)>
    <circle cx="12" cy="12" r="10" fill="none" stroke="currentColor" stroke-width="2"></circle>
    <path d="M8 12l3 3 5-6" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"></path>
  </svg>
</column>
