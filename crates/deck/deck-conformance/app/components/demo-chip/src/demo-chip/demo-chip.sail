<!-- Shared CE for multi-entry chunk dedup; data-marker is the shared_chunk test literal. -->

<scope>
  style chip = { pad(1).radius(md).bg(surface-2).fg(foreground).type({ size: xs, family: mono }) }
</scope>

<span data-marker="demo-chip-marker" style(chip)><slot></slot></span>
