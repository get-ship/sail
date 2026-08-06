<column gap="8">
  style.pad(6).maxW(breakpoint-tablet).mx(auto)

  <header>
    style.borderBWidth(1).borderB(border).padB(4)

    <h1>
      style.fontSize(2xl).fontWeight(bold).fg(foreground).mb(2)
ship/agents — Domain Stubs</h1>
    <p>
      style.fg(muted-foreground).fontSize(sm).leading(relaxed)

      Pack <strong>ship/agents</strong>. Each card below renders the real custom
      element tag with static mock attribute values so the shell registers and
      paints. A visible badge on each card marks it as a domain stub — no
      <code>ship.*</code> calls, no live data.
    </p>
  </header>

  <!-- ship-agent-card -->
  <column gap="4">
    style.borderWidth(1).border(border).radius(lg).bg(surface-1).pad(5)

    <h2>
      style.fontSize(lg).fontWeight(semibold).fg(foreground).fontFamily(mono)

      <code>ship-agent-card</code>
      <span>
        style.fontSize(xs).fontWeight(medium).fontFamily(body).fg(warning-foreground).bg(warning).borderWidth(1).border(warning).radius(full).padX(2).padY(half).whitespace(nowrap).ms(3)
domain-stub — awaits backend spec</span>
    </h2>
    <p>
      style.fg(muted-foreground).fontSize(sm).leading(relaxed)

      Props: <code>id: string</code>. Slot: <code>trailing</code> (action buttons).
      In production the controller calls <code>ship.agents.detail(&#123; id &#125;)</code>
      via <code>live()</code>; this stub passes a static mock id.
    </p>
    <column gap="4">
      <ship-agent-card id="agent-mock-001">
        <button slot="trailing">
          style.fontSize(sm).padY(1).padX(3).borderWidth(1).border(border).radius(sm).bg(surface-2).fg(foreground).cursor(default)
Edit</button>
      </ship-agent-card>
    </column>
  </column>

  <!-- ship-agent-thread -->
  <column gap="4">
    style.borderWidth(1).border(border).radius(lg).bg(surface-1).pad(5)

    <h2>
      style.fontSize(lg).fontWeight(semibold).fg(foreground).fontFamily(mono)

      <code>ship-agent-thread</code>
      <span>
        style.fontSize(xs).fontWeight(medium).fontFamily(body).fg(warning-foreground).bg(warning).borderWidth(1).border(warning).radius(full).padX(2).padY(half).whitespace(nowrap).ms(3)
domain-stub — awaits backend spec</span>
    </h2>
    <p>
      style.fg(muted-foreground).fontSize(sm).leading(relaxed)

      Props: <code>session-id: string</code>. No slots.
      Composes <code>sail-message-list</code> + <code>sail-composer</code>;
      the Ship wrapper owns the session binding and submit handler.
    </p>
    <column gap="4">
      <ship-agent-thread session-id="sess-mock-001"></ship-agent-thread>
    </column>
  </column>

  <!-- ship-agents-table -->
  <column gap="4">
    style.borderWidth(1).border(border).radius(lg).bg(surface-1).pad(5)

    <h2>
      style.fontSize(lg).fontWeight(semibold).fg(foreground).fontFamily(mono)

      <code>ship-agents-table</code>
      <span>
        style.fontSize(xs).fontWeight(medium).fontFamily(body).fg(warning-foreground).bg(warning).borderWidth(1).border(warning).radius(full).padX(2).padY(half).whitespace(nowrap).ms(3)
domain-stub — awaits backend spec</span>
    </h2>
    <p>
      style.fg(muted-foreground).fontSize(sm).leading(relaxed)

      No props — data is fetched internally via <code>live()</code> from
      <code>ship.agents.summaries()</code>. Renders a sortable
      <code>sail-table</code> with columns: name, description, source, skills, mcp.
    </p>
    <column gap="4">
      <ship-agents-table></ship-agents-table>
    </column>
  </column>

  <!-- sail-harness-blob — all enum combinations sampled -->
  <column gap="4">
    style.borderWidth(1).border(border).radius(lg).bg(surface-1).pad(5)

    <h2>
      style.fontSize(lg).fontWeight(semibold).fg(foreground).fontFamily(mono)

      <code>sail-harness-blob</code>
      <span>
        style.fontSize(xs).fontWeight(medium).fontFamily(body).fg(warning-foreground).bg(warning).borderWidth(1).border(warning).radius(full).padX(2).padY(half).whitespace(nowrap).ms(3)
domain-stub — awaits backend spec</span>
    </h2>
    <p>
      style.fg(muted-foreground).fontSize(sm).leading(relaxed)

      Props: <code>harness: claude|codex|cursor|gemini|opencode</code>,
      <code>size: sm|md|lg</code>, <code>variant: tile|glyph</code>.
      Slot: default (label text). Inlines brand SVG — no network fetch.
    </p>

    <column gap="4">
      <row gap="3">
        <span>
          style.fontFamily(mono).fontSize(xs).fg(muted-foreground).minW(20)
tile / md</span>
        <sail-harness-blob harness="claude" size="md" variant="tile">Claude</sail-harness-blob>
        <sail-harness-blob harness="codex" size="md" variant="tile">Codex</sail-harness-blob>
        <sail-harness-blob harness="cursor" size="md" variant="tile">Cursor</sail-harness-blob>
        <sail-harness-blob harness="gemini" size="md" variant="tile">Gemini</sail-harness-blob>
        <sail-harness-blob harness="opencode" size="md" variant="tile">OpenCode</sail-harness-blob>
      </row>

      <row gap="3">
        <span>
          style.fontFamily(mono).fontSize(xs).fg(muted-foreground).minW(20)
tile / sm</span>
        <sail-harness-blob harness="claude" size="sm" variant="tile">Claude</sail-harness-blob>
        <sail-harness-blob harness="codex" size="sm" variant="tile">Codex</sail-harness-blob>
        <sail-harness-blob harness="cursor" size="sm" variant="tile">Cursor</sail-harness-blob>
        <sail-harness-blob harness="gemini" size="sm" variant="tile">Gemini</sail-harness-blob>
        <sail-harness-blob harness="opencode" size="sm" variant="tile">OpenCode</sail-harness-blob>
      </row>

      <row gap="3">
        <span>
          style.fontFamily(mono).fontSize(xs).fg(muted-foreground).minW(20)
tile / lg</span>
        <sail-harness-blob harness="claude" size="lg" variant="tile">Claude</sail-harness-blob>
        <sail-harness-blob harness="codex" size="lg" variant="tile">Codex</sail-harness-blob>
        <sail-harness-blob harness="cursor" size="lg" variant="tile">Cursor</sail-harness-blob>
        <sail-harness-blob harness="gemini" size="lg" variant="tile">Gemini</sail-harness-blob>
        <sail-harness-blob harness="opencode" size="lg" variant="tile">OpenCode</sail-harness-blob>
      </row>

      <row gap="3">
        <span>
          style.fontFamily(mono).fontSize(xs).fg(muted-foreground).minW(20)
glyph / md</span>
        <sail-harness-blob harness="claude" size="md" variant="glyph">Claude</sail-harness-blob>
        <sail-harness-blob harness="codex" size="md" variant="glyph">Codex</sail-harness-blob>
        <sail-harness-blob harness="cursor" size="md" variant="glyph">Cursor</sail-harness-blob>
        <sail-harness-blob harness="gemini" size="md" variant="glyph">Gemini</sail-harness-blob>
        <sail-harness-blob harness="opencode" size="md" variant="glyph">OpenCode</sail-harness-blob>
      </row>
    </column>
  </column>
</column>
