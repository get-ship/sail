<!--
  <demo-chip> — trivial custom element for the deck-conformance shared-chunk dedup
  showcase (GAP 4). Both routes/widgets and routes/gallery use this tag; the
  multi-entry build must hoist it into ONE shared chunk.

  `data-marker="demo-chip-marker"` is the stable literal the deck-conformance
  `shared_chunk` test searches `.deck/` JS files for to prove the dedup
  invariant. `data-*` is a recognized HTML pass-through (see `layout::is_passthrough_attr`)
  so the framework keeps it on the element; the marker survives lowering
  into the emitted bundle as `... data-marker="demo-chip-marker" ...`.
-->

<span data-marker="demo-chip-marker" sx={style.radius(md).bg(surface-2).fg(foreground).fontSize(xs).fontFamily(mono).pad(1)}><slot></slot></span>
