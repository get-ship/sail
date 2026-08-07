<column gap="8" style({ box: pad(6).maxW(breakpoint-tablet).mx(auto) })>
  <header style({ box: padB(4), paint: borderBWidth(1).borderB(border) })>
    <h1 style({ box: mb(2), paint: fg(foreground), text: fontSize(2xl).fontWeight(bold) })>ship/viewer — Domain Stubs</h1>
    <p style({ paint: fg(muted-foreground), text: fontSize(sm) })>
      Pack <strong>ship/viewer</strong>. The card below renders the real custom
      element tag with static mock attribute values so the shell registers and
      paints. A visible badge marks it as a domain stub — no
      <code>ship.*</code> calls, no live data. The <code>src</code> identity
      binding is intentionally absent; the component falls back to the
      <code>content-type</code>, <code>filename</code>, and <code>text</code>
      scalar props for detection and dispatch.
    </p>
  </header>

  <!-- ── ship-viewer ──────────────────────────────────────────────────────── -->
  <column gap="4" style({ box: pad(5), paint: borderWidth(1).border(border).radius(lg).bg(surface-1) })>
    <h2 style({ paint: fg(foreground), text: fontSize(lg).fontWeight(semibold).fontFamily(mono) })>
      <code>ship-viewer</code>
      <span style({ box: padX(2).padY(1).ms(3), paint: fg(warning-foreground).bg(warning).radius(sm), text: fontSize(xs).fontWeight(medium).fontFamily(body).tracking(wide) })>domain-stub — awaits backend spec</span>
    </h2>
    <p style({ paint: fg(muted-foreground), text: fontSize(sm).leading(relaxed) })>
      Props: <code>content-type: string</code> (MIME hint, e.g.
      <code>text/markdown</code>), <code>filename: string</code> (extension
      fallback for detection), <code>text: string</code> (text-content fallback
      when no <code>src</code> identity is wired). The dispatcher resolves
      the renderer through the pure <code>createMimeRegistry()</code> engine and
      branches via <code>&#123;#match kind()&#125;</code> — one of:
      <code>markdown</code>, <code>code</code>, <code>json</code>,
      <code>diff</code>, <code>image</code>, <code>media</code>,
      <code>tabular</code>, <code>pdf</code>, or the default unsupported
      empty-state. Slots: <code>toolbar</code> (above renderer),
      <code>download</code> (forwarded into the empty-state action slot).
      Read-only — no <code>fetch</code> or <code>ship.*</code> calls; the host
      resolves bytes/text and passes via <code>src</code>.
    </p>

    <column gap="4">

      <!-- markdown arm -->
      <column gap="2" style({ box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) })>
        <span style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) })>content-type=text/markdown · toolbar slot</span>
        <ship-viewer content-type="text/markdown" text="# Hello from ship-viewer&#10;&#10;This is a **markdown** document rendered by the dispatcher.&#10;&#10;- item one&#10;- item two&#10;- item three">
          <span slot="toolbar" style({ box: padX(2).padY(1), paint: fg(muted-foreground).borderWidth(1).border(border).radius(sm).bg(surface-1), text: fontSize(xs).fontFamily(mono) })>Toolbar slot</span>
        </ship-viewer>
      </column>

      <!-- code arm — TypeScript via filename extension -->
      <column gap="2" style({ box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) })>
        <span style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) })>filename=controller.ts · code arm (extension detection)</span>
        <ship-viewer filename="controller.ts" text="import { controller, signal } from '@ship/sail-runtime'&#10;&#10;export default controller(() => ({ count: signal(0) }))">
        </ship-viewer>
      </column>

      <!-- json arm -->
      <column gap="2" style({ box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) })>
        <span style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) })>content-type=application/json · json arm</span>
        <ship-viewer content-type="application/json" text={'{"id":"artifact-001","kind":"file","name":"report.json","size":1024,"created_at":"2026-06-05T00:00:00Z"}'}>
        </ship-viewer>
      </column>

      <!-- unsupported / empty-state arm + download slot -->
      <column gap="2" style({ box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) })>
        <span style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) })>content-type=application/octet-stream · unsupported empty-state · download slot</span>
        <ship-viewer content-type="application/octet-stream" filename="binary.bin">
          <a slot="download" href="#" style({ paint: fg(accent), text: fontSize(sm).underline(true) })>Download binary.bin</a>
        </ship-viewer>
      </column>

      <!-- empty (no props) — default empty-state -->
      <column gap="2" style({ box: pad(3), paint: borderWidth(1).border(divider).radius(md).bg(surface-2) })>
        <span style({ paint: fg(muted-foreground), text: fontSize(xs).fontWeight(medium).textCase(lowercase).tracking(wide).fontFamily(mono) })>no props — default empty-state (no source)</span>
        <ship-viewer></ship-viewer>
      </column>

    </column>
  </column>
</column>
