<column gap="8" sx="p-6 max-w-breakpoint-tablet mx-auto">
  <header sx="border-b border-b-border pb-4">
    <h1 sx="text-2xl font-bold text-foreground mb-2">ship/pdf — Domain Stubs</h1>
    <p sx="text-muted-foreground text-sm">
      Pack <strong>ship/pdf</strong>. Each card below renders the real custom
      element tag with static mock attribute values so the shell registers and
      paints. A visible badge on each card marks it as a domain stub — no
      <code>ship.*</code> calls, no live data.
    </p>
  </header>

  <!-- ship-pdf -->
  <column gap="4" sx="border border-border rounded-lg bg-surface-1 p-5">
    <h2 sx="text-lg font-semibold text-foreground font-mono">
      <code>ship-pdf</code>
      <span sx="text-xs font-medium font-body text-warning-foreground bg-warning rounded-sm px-2 py-1 tracking-wide ms-3">domain-stub — awaits backend spec</span>
    </h2>
    <p sx="text-muted-foreground text-sm leading-relaxed">
      Props: <code>src: string</code> (URL or blob reference, default empty),
      <code>page: number</code> (1-based page index, default 1),
      <code>zoom: number</code> (scale factor, default 1),
      <code>label: string</code> (accessible label, default "PDF page").
      Slot: <code>toolbar</code> (custom toolbar controls). No live data is
      wired in this stub — <code>src</code> is intentionally empty so the
      component renders its idle/empty shell state.
    </p>

    <column gap="4">
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">default — no src, page=1, zoom=1</span>
        <ship-pdf label="PDF page"></ship-pdf>
      </column>

      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">page=2 zoom=1.5 — page and zoom attrs reflected</span>
        <ship-pdf page="2" zoom="1.5" label="PDF page 2 at 150%"></ship-pdf>
      </column>

      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">page=1 zoom=0.75 — reduced zoom</span>
        <ship-pdf page="1" zoom="0.75" label="PDF page at 75%"></ship-pdf>
      </column>

      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">with toolbar slot — custom controls slotted</span>
        <ship-pdf page="1" zoom="1" label="PDF with toolbar">
          <row slot="toolbar" gap="3" sx="p-2 bg-surface-1 rounded-sm border border-border">
            <button sx="text-sm px-2 py-1 border border-border rounded-sm bg-surface-2 text-muted-foreground cursor-default" disabled>&#8592; Prev</button>
            <span sx="text-sm text-foreground font-mono">Page 1</span>
            <button sx="text-sm px-2 py-1 border border-border rounded-sm bg-surface-2 text-muted-foreground cursor-default" disabled>Next &#8594;</button>
            <span sx="text-xs text-muted-foreground italic ml-auto">slotted toolbar demo</span>
          </row>
        </ship-pdf>
      </column>
    </column>
  </column>
</column>
