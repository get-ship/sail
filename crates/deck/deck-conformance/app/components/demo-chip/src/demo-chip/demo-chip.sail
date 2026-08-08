<!-- Shared CE for multi-entry chunk dedup; data-marker is the shared_chunk test literal. -->

<scope>
  style chip = {
    box: pad(1),
    paint: radius(md).bg(surface-2).fg(foreground),
    text: fontSize(xs).fontFamily(mono),
  }
</scope>

<span data-marker="demo-chip-marker" style(chip)><slot></slot></span>
