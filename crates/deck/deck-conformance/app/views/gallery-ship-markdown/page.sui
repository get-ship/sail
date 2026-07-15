<column gap="8" sx="p-6 max-w-breakpoint-tablet mx-auto">
  <header>
    <h1 sx="text-2xl font-bold mb-2">ship/markdown — Domain Stubs</h1>
    <p sx="text-muted-foreground">
      Pack <strong>ship/markdown</strong>. Each card below renders the real
      custom element tag with static mock attribute values so the shell
      registers and paints. A visible badge on each card marks it as a domain
      stub — no <code>ship.*</code> calls, no live data.
    </p>
  </header>

  <!-- ship-markdown-editor: default / md size -->
  <section>
    <h2 sx="text-sm font-semibold text-muted-foreground uppercase tracking-wide mb-3 pb-2 border-b border-b-border">
      <code>ship-markdown-editor</code>
      <span sx="text-xs font-medium text-warning-foreground bg-warning border border-warning rounded-sm px-2 py-half whitespace-nowrap normal-case tracking-normal ms-3">domain-stub — awaits backend spec</span>
    </h2>
    <p sx="text-sm text-muted-foreground mb-4 leading-relaxed">
      Props: <code>value: string</code> (form-associated),
      <code>placeholder: string</code>,
      <code>size: sm|md|lg</code>,
      <code>tone: string</code>,
      <code>disabled: bool</code>,
      <code>invalid: bool</code>,
      <code>rows: number</code>,
      <code>full-height: bool</code>,
      <code>name: string</code>,
      <code>label: string</code>,
      <code>show-selection-toolbar: bool</code>,
      <code>allow-comment: bool</code>,
      <code>allow-generate: bool</code>.
      No slots. Wraps TipTap v3; toolbar + bubble toolbar + optional comment
      composer are built-in.
    </p>

    <column sx="bg-card border border-border rounded-lg p-5">
      <ship-markdown-editor
        name="demo-md-default"
        label="Default (md)"
        placeholder="Write something in markdown…"
        size="md"
        rows="8">
      </ship-markdown-editor>
    </column>
  </section>

  <!-- ship-markdown-editor: sm size -->
  <section>
    <h2 sx="text-sm font-semibold text-muted-foreground uppercase tracking-wide mb-3 pb-2 border-b border-b-border">
      <code>ship-markdown-editor</code> — size sm
      <span sx="text-xs font-medium text-warning-foreground bg-warning border border-warning rounded-sm px-2 py-half whitespace-nowrap normal-case tracking-normal ms-3">domain-stub — awaits backend spec</span>
    </h2>
    <p sx="text-sm text-muted-foreground mb-4 leading-relaxed">
      Small variant. Reduced font-size, toolbar height, and padding via
      <code>size="sm"</code>. Placeholder text demonstrates the empty state.
    </p>
    <column sx="bg-card border border-border rounded-lg p-5">
      <ship-markdown-editor
        name="demo-md-sm"
        label="Small editor"
        placeholder="Compact editor…"
        size="sm"
        rows="6">
      </ship-markdown-editor>
    </column>
  </section>

  <!-- ship-markdown-editor: lg size -->
  <section>
    <h2 sx="text-sm font-semibold text-muted-foreground uppercase tracking-wide mb-3 pb-2 border-b border-b-border">
      <code>ship-markdown-editor</code> — size lg
      <span sx="text-xs font-medium text-warning-foreground bg-warning border border-warning rounded-sm px-2 py-half whitespace-nowrap normal-case tracking-normal ms-3">domain-stub — awaits backend spec</span>
    </h2>
    <p sx="text-sm text-muted-foreground mb-4 leading-relaxed">
      Large variant. Larger font-size and row height. Suitable for document
      authoring surfaces.
    </p>
    <column sx="bg-card border border-border rounded-lg p-5">
      <ship-markdown-editor
        name="demo-md-lg"
        label="Large editor"
        placeholder="Document body…"
        size="lg"
        rows="10">
      </ship-markdown-editor>
    </column>
  </section>

  <!-- ship-markdown-editor: disabled -->
  <section>
    <h2 sx="text-sm font-semibold text-muted-foreground uppercase tracking-wide mb-3 pb-2 border-b border-b-border">
      <code>ship-markdown-editor</code> — disabled
      <span sx="text-xs font-medium text-warning-foreground bg-warning border border-warning rounded-sm px-2 py-half whitespace-nowrap normal-case tracking-normal ms-3">domain-stub — awaits backend spec</span>
    </h2>
    <p sx="text-sm text-muted-foreground mb-4 leading-relaxed">
      <code>disabled</code> is a boolean, form-disabled prop. The shell
      applies 60% opacity and blocks pointer events. Toolbar buttons are
      also disabled.
    </p>
    <column sx="bg-card border border-border rounded-lg p-5">
      <ship-markdown-editor
        name="demo-md-disabled"
        label="Disabled editor"
        placeholder="Read-only content here…"
        size="md"
        rows="6"
        disabled>
      </ship-markdown-editor>
    </column>
  </section>

  <!-- ship-markdown-editor: invalid -->
  <section>
    <h2 sx="text-sm font-semibold text-muted-foreground uppercase tracking-wide mb-3 pb-2 border-b border-b-border">
      <code>ship-markdown-editor</code> — invalid
      <span sx="text-xs font-medium text-warning-foreground bg-warning border border-warning rounded-sm px-2 py-half whitespace-nowrap normal-case tracking-normal ms-3">domain-stub — awaits backend spec</span>
    </h2>
    <p sx="text-sm text-muted-foreground mb-4 leading-relaxed">
      <code>invalid</code> applies the destructive ring to the editor shell.
      Used in form-validation contexts when the field value fails a constraint.
    </p>
    <column sx="bg-card border border-border rounded-lg p-5">
      <ship-markdown-editor
        name="demo-md-invalid"
        label="Field with validation error"
        placeholder="Content required…"
        size="md"
        rows="6"
        invalid>
      </ship-markdown-editor>
    </column>
  </section>

  <!-- ship-markdown-editor: no selection toolbar, no comment -->
  <section>
    <h2 sx="text-sm font-semibold text-muted-foreground uppercase tracking-wide mb-3 pb-2 border-b border-b-border">
      <code>ship-markdown-editor</code> — toolbar-only mode
      <span sx="text-xs font-medium text-warning-foreground bg-warning border border-warning rounded-sm px-2 py-half whitespace-nowrap normal-case tracking-normal ms-3">domain-stub — awaits backend spec</span>
    </h2>
    <p sx="text-sm text-muted-foreground mb-4 leading-relaxed">
      <code>show-selection-toolbar="false"</code> hides the floating bubble
      toolbar. <code>allow-comment="false"</code> removes the comment button
      from the bubble. Both are false here — a simple editing surface without
      collaborative features.
    </p>
    <column sx="bg-card border border-border rounded-lg p-5">
      <ship-markdown-editor
        name="demo-md-simple"
        label="Toolbar-only editor"
        placeholder="Editing without bubble or comments…"
        size="md"
        rows="7"
        show-selection-toolbar="false"
        allow-comment="false">
      </ship-markdown-editor>
    </column>
  </section>

  <!-- ship-markdown-editor: allow-generate -->
  <section>
    <h2 sx="text-sm font-semibold text-muted-foreground uppercase tracking-wide mb-3 pb-2 border-b border-b-border">
      <code>ship-markdown-editor</code> — allow-generate
      <span sx="text-xs font-medium text-warning-foreground bg-warning border border-warning rounded-sm px-2 py-half whitespace-nowrap normal-case tracking-normal ms-3">domain-stub — awaits backend spec</span>
    </h2>
    <p sx="text-sm text-muted-foreground mb-4 leading-relaxed">
      <code>allow-generate="true"</code> adds a sparkles button to the
      selection bubble toolbar. Clicking it fires the <code>generate</code>
      event — the host is responsible for wiring to an AI back-end.
    </p>
    <column sx="bg-card border border-border rounded-lg p-5">
      <ship-markdown-editor
        name="demo-md-generate"
        label="AI-enabled editor"
        placeholder="Select text to invoke AI generation…"
        size="md"
        rows="7"
        allow-generate="true">
      </ship-markdown-editor>
    </column>
  </section>
</column>
