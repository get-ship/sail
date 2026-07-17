<column gap="8" sx="p-6 max-w-breakpoint-tablet mx-auto">
  <header sx="border-b border-b-border pb-4">
    <h1 sx="text-2xl font-bold text-foreground mb-2">ship/agents — Domain Stubs</h1>
    <p sx="text-muted-foreground text-sm leading-relaxed">
      Pack <strong>ship/agents</strong>. Each card below renders the real custom
      element tag with static mock attribute values so the shell registers and
      paints. A visible badge on each card marks it as a domain stub — no
      <code>ship.*</code> calls, no live data.
    </p>
  </header>

  <!-- ship-agent-card -->
  <column gap="4" sx="border border-border rounded-lg bg-surface-1 p-5">
    <h2 sx="text-lg font-semibold text-foreground font-mono">
      <code>ship-agent-card</code>
      <span sx="text-xs font-medium font-body text-warning-foreground bg-warning border border-warning rounded-full px-2 py-half whitespace-nowrap ms-3">domain-stub — awaits backend spec</span>
    </h2>
    <p sx="text-muted-foreground text-sm leading-relaxed">
      Props: <code>id: string</code>. Slot: <code>trailing</code> (action buttons).
      In production the controller calls <code>ship.agents.detail(&#123; id &#125;)</code>
      via <code>live()</code>; this stub passes a static mock id.
    </p>
    <column gap="4">
      <ship-agent-card id="agent-mock-001">
        <button slot="trailing" sx="text-sm py-1 px-3 border border-border rounded-sm bg-surface-2 text-foreground cursor-default">Edit</button>
      </ship-agent-card>
    </column>
  </column>

  <!-- ship-agent-thread -->
  <column gap="4" sx="border border-border rounded-lg bg-surface-1 p-5">
    <h2 sx="text-lg font-semibold text-foreground font-mono">
      <code>ship-agent-thread</code>
      <span sx="text-xs font-medium font-body text-warning-foreground bg-warning border border-warning rounded-full px-2 py-half whitespace-nowrap ms-3">domain-stub — awaits backend spec</span>
    </h2>
    <p sx="text-muted-foreground text-sm leading-relaxed">
      Props: <code>session-id: string</code>. No slots.
      Composes <code>sail-message-list</code> + <code>sail-composer</code>;
      the Ship wrapper owns the session binding and submit handler.
    </p>
    <column gap="4">
      <ship-agent-thread session-id="sess-mock-001"></ship-agent-thread>
    </column>
  </column>

  <!-- ship-agents-table -->
  <column gap="4" sx="border border-border rounded-lg bg-surface-1 p-5">
    <h2 sx="text-lg font-semibold text-foreground font-mono">
      <code>ship-agents-table</code>
      <span sx="text-xs font-medium font-body text-warning-foreground bg-warning border border-warning rounded-full px-2 py-half whitespace-nowrap ms-3">domain-stub — awaits backend spec</span>
    </h2>
    <p sx="text-muted-foreground text-sm leading-relaxed">
      No props — data is fetched internally via <code>live()</code> from
      <code>ship.agents.summaries()</code>. Renders a sortable
      <code>sail-table</code> with columns: name, description, source, skills, mcp.
    </p>
    <column gap="4">
      <ship-agents-table></ship-agents-table>
    </column>
  </column>

  <!-- sail-harness-blob — all enum combinations sampled -->
  <column gap="4" sx="border border-border rounded-lg bg-surface-1 p-5">
    <h2 sx="text-lg font-semibold text-foreground font-mono">
      <code>sail-harness-blob</code>
      <span sx="text-xs font-medium font-body text-warning-foreground bg-warning border border-warning rounded-full px-2 py-half whitespace-nowrap ms-3">domain-stub — awaits backend spec</span>
    </h2>
    <p sx="text-muted-foreground text-sm leading-relaxed">
      Props: <code>harness: claude|codex|cursor|gemini|opencode</code>,
      <code>size: sm|md|lg</code>, <code>variant: tile|glyph</code>.
      Slot: default (label text). Inlines brand SVG — no network fetch.
    </p>

    <column gap="4">
      <row gap="3">
        <span sx="font-mono text-xs text-muted-foreground min-w-20">tile / md</span>
        <sail-harness-blob harness="claude" size="md" variant="tile">Claude</sail-harness-blob>
        <sail-harness-blob harness="codex" size="md" variant="tile">Codex</sail-harness-blob>
        <sail-harness-blob harness="cursor" size="md" variant="tile">Cursor</sail-harness-blob>
        <sail-harness-blob harness="gemini" size="md" variant="tile">Gemini</sail-harness-blob>
        <sail-harness-blob harness="opencode" size="md" variant="tile">OpenCode</sail-harness-blob>
      </row>

      <row gap="3">
        <span sx="font-mono text-xs text-muted-foreground min-w-20">tile / sm</span>
        <sail-harness-blob harness="claude" size="sm" variant="tile">Claude</sail-harness-blob>
        <sail-harness-blob harness="codex" size="sm" variant="tile">Codex</sail-harness-blob>
        <sail-harness-blob harness="cursor" size="sm" variant="tile">Cursor</sail-harness-blob>
        <sail-harness-blob harness="gemini" size="sm" variant="tile">Gemini</sail-harness-blob>
        <sail-harness-blob harness="opencode" size="sm" variant="tile">OpenCode</sail-harness-blob>
      </row>

      <row gap="3">
        <span sx="font-mono text-xs text-muted-foreground min-w-20">tile / lg</span>
        <sail-harness-blob harness="claude" size="lg" variant="tile">Claude</sail-harness-blob>
        <sail-harness-blob harness="codex" size="lg" variant="tile">Codex</sail-harness-blob>
        <sail-harness-blob harness="cursor" size="lg" variant="tile">Cursor</sail-harness-blob>
        <sail-harness-blob harness="gemini" size="lg" variant="tile">Gemini</sail-harness-blob>
        <sail-harness-blob harness="opencode" size="lg" variant="tile">OpenCode</sail-harness-blob>
      </row>

      <row gap="3">
        <span sx="font-mono text-xs text-muted-foreground min-w-20">glyph / md</span>
        <sail-harness-blob harness="claude" size="md" variant="glyph">Claude</sail-harness-blob>
        <sail-harness-blob harness="codex" size="md" variant="glyph">Codex</sail-harness-blob>
        <sail-harness-blob harness="cursor" size="md" variant="glyph">Cursor</sail-harness-blob>
        <sail-harness-blob harness="gemini" size="md" variant="glyph">Gemini</sail-harness-blob>
        <sail-harness-blob harness="opencode" size="md" variant="glyph">OpenCode</sail-harness-blob>
      </row>
    </column>
  </column>
</column>
