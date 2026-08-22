<!-- ship/viewer domain stubs — static mock attrs; no src identity or live data. -->

<scope>
  style page = { pad(6).size({ max-w: breakpoint-tablet }).margin({ x: auto }) }
  style header = { pad({ b: 4 }).border({ width: 1, color: border, edge: b }) }
  style title = { margin({ b: 2 }).fg(foreground).type({ size: 2xl, weight: bold }) }
  style lead = { fg(muted-foreground).type({ size: sm }) }
  style card = { pad(5).border({ width: 1, color: border }).radius(lg).bg(surface) }
  style cardTitle = { fg(foreground).type({ size: lg, weight: semibold, family: mono }) }
  style body = { fg(muted-foreground).type({ size: sm, leading: relaxed }) }
  style arm = { pad(3).border({ width: 1, color: divider }).radius(md).bg(surface) }
  style caption = { fg(muted-foreground).type({ size: xs, weight: medium, case: lowercase, tracking: wide, family: mono }) }
  style toolbar = { pad({ x: 2, y: 1 }).fg(muted-foreground).border({ width: 1, color: border }).radius(sm).bg(surface).type({ size: xs, family: mono }) }
  style download = { fg(accent).type({ size: sm, decoration: underline }) }
</scope>

<column gap="8" style(page)>
  <header style(header)>
    <h1 style(title)>ship/viewer — Domain Stubs</h1>
    <p style(lead)>
      Pack <strong>ship/viewer</strong>. The card below renders the real custom
      element tag with static mock attribute values so the shell registers and
      paints. A visible badge marks it as a domain stub — no
      <code>ship.*</code> calls, no live data. The <code>src</code> identity
      binding is intentionally absent; the component falls back to the
      <code>content-type</code>, <code>filename</code>, and <code>text</code>
      scalar props for detection and dispatch.
    </p>
  </header>

  <column gap="4" style(card)>
    <h2 style(cardTitle)>
      <code>ship-viewer</code>
      <badge tone="warning" size="sm">domain-stub</badge>
    </h2>
    <p style(body)>
      Props: <code>content-type: string</code> (MIME hint, e.g.
      <code>text/markdown</code>), <code>filename: string</code> (extension
      fallback for detection), <code>text: string</code> (text-content fallback
      when no <code>src</code> identity is wired). The dispatcher resolves
      the renderer through the pure <code>createMimeRegistry()</code> engine and
      branches via match on kind — one of:
      <code>markdown</code>, <code>code</code>, <code>json</code>,
      <code>diff</code>, <code>image</code>, <code>media</code>,
      <code>tabular</code>, <code>pdf</code>, or the default unsupported
      empty-state. Slots: <code>toolbar</code> (above renderer),
      <code>download</code> (forwarded into the empty-state action slot).
      Read-only — no <code>fetch</code> or <code>ship.*</code> calls; the host
      resolves bytes/text and passes via <code>src</code>.
    </p>

    <column gap="4">
      <column gap="2" style(arm)>
        <span style(caption)>content-type=text/markdown · toolbar slot</span>
        <ship-viewer content-type="text/markdown" text="# Hello from ship-viewer&#10;&#10;This is a **markdown** document rendered by the dispatcher.&#10;&#10;- item one&#10;- item two&#10;- item three">
          <span slot="toolbar" style(toolbar)>Toolbar slot</span>
        </ship-viewer>
      </column>

      <column gap="2" style(arm)>
        <span style(caption)>filename=controller.ts · code arm (extension detection)</span>
        <ship-viewer filename="controller.ts" text="import { controller, signal } from '@ship/sail-runtime'&#10;&#10;export default controller(() => ({ count: signal(0) }))">
        </ship-viewer>
      </column>

      <column gap="2" style(arm)>
        <span style(caption)>content-type=application/json · json arm</span>
        <ship-viewer content-type="application/json" text={'{"id":"artifact-001","kind":"file","name":"report.json","size":1024,"created_at":"2026-06-05T00:00:00Z"}'}>
        </ship-viewer>
      </column>

      <column gap="2" style(arm)>
        <span style(caption)>content-type=application/octet-stream · unsupported empty-state · download slot</span>
        <ship-viewer content-type="application/octet-stream" filename="binary.bin">
          <a slot="download" href="#" style(download)>Download binary.bin</a>
        </ship-viewer>
      </column>

      <column gap="2" style(arm)>
        <span style(caption)>no props — default empty-state (no source)</span>
        <ship-viewer></ship-viewer>
      </column>
    </column>
  </column>
</column>
