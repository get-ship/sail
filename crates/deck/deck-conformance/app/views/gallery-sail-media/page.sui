<column gap="8" sx="p-6 max-w-breakpoint-tablet mx-auto">
  <header sx="border-b border-b-border pb-4">
    <h1 sx="text-2xl font-bold text-foreground mb-2">sui/media — Domain Stubs</h1>
    <p sx="text-muted-foreground text-sm">
      Pack <strong>sui/media</strong>. Each card renders the real custom element
      tag with static mock attribute values so the shell registers and paints.
      A visible badge marks each card as a domain stub — no <code>ship.*</code>
      calls, no live data. Structured inputs (<code>prop:src</code>,
      <code>prop:table</code>) arrive via identity at runtime; this gallery
      exercises only reflected scalar attributes and the <code>text</code>
      string fallback prop.
    </p>
  </header>

  <!-- ── sail-media-av ──────────────────────────────────────────────── -->
  <column gap="4" sx="border border-border rounded-lg bg-surface-1 p-5">
    <h2 sx="text-lg font-semibold text-foreground font-mono">
      <code>sail-media-av</code>
      <span sx="text-xs font-medium font-body text-warning-foreground bg-warning rounded-sm px-2 py-1 tracking-wide ms-3">domain-stub — awaits backend spec</span>
    </h2>
    <p sx="text-muted-foreground text-sm leading-relaxed">
      Props: <code>kind: video|audio|auto</code> (reflect),
      <code>autoplay: bool</code> (reflect, default false),
      <code>controls: bool</code> (reflect, default true),
      <code>waveform: bool</code> (reflect, default false).
      The resolved media source arrives via <code>prop:src</code> identity —
      never a URL attribute. This stub shows the tag shell with reflected
      attributes; no source is wired so the native element renders its
      default empty state.
    </p>
    <column gap="4">
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">kind=video controls</span>
        <sail-media-av kind="video" controls></sail-media-av>
      </column>
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">kind=audio controls</span>
        <sail-media-av kind="audio" controls></sail-media-av>
      </column>
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">kind=audio controls waveform</span>
        <sail-media-av kind="audio" controls waveform></sail-media-av>
      </column>
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">kind=auto controls (default)</span>
        <sail-media-av kind="auto" controls></sail-media-av>
      </column>
    </column>
  </column>

  <!-- ── sail-media-csv ─────────────────────────────────────────────── -->
  <column gap="4" sx="border border-border rounded-lg bg-surface-1 p-5">
    <h2 sx="text-lg font-semibold text-foreground font-mono">
      <code>sail-media-csv</code>
      <span sx="text-xs font-medium font-body text-warning-foreground bg-warning rounded-sm px-2 py-1 tracking-wide ms-3">domain-stub — awaits backend spec</span>
    </h2>
    <p sx="text-muted-foreground text-sm leading-relaxed">
      Props: <code>delimiter: comma|tab|auto</code> (reflect, default auto),
      <code>header: bool</code> (reflect, default true),
      <code>text: string</code> (SSR/declarative fallback — inline CSV text).
      The resolved CSV/TSV bytes arrive via <code>prop:src</code> identity;
      the <code>text</code> prop is the static fallback and is used here.
      Composes <code>sail-table variant="data"</code> internally.
    </p>
    <column gap="4">
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">delimiter=comma header text (inline CSV)</span>
        <sail-media-csv delimiter="comma" header text="Name,Role,Status
Alice,Engineer,active
Bob,Designer,active
Carol,PM,inactive"></sail-media-csv>
      </column>
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">delimiter=tab header text (inline TSV)</span>
        <sail-media-csv delimiter="tab" header text="File	Size	Modified
readme.md	1.2 KB	2026-06-01
main.ts	4.8 KB	2026-06-04
style.css	2.1 KB	2026-05-30"></sail-media-csv>
      </column>
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">delimiter=auto no-header</span>
        <sail-media-csv delimiter="auto" text="alpha,1,true
beta,2,false
gamma,3,true"></sail-media-csv>
      </column>
    </column>
  </column>

  <!-- ── sail-media-image ───────────────────────────────────────────── -->
  <column gap="4" sx="border border-border rounded-lg bg-surface-1 p-5">
    <h2 sx="text-lg font-semibold text-foreground font-mono">
      <code>sail-media-image</code>
      <span sx="text-xs font-medium font-body text-warning-foreground bg-warning rounded-sm px-2 py-1 tracking-wide ms-3">domain-stub — awaits backend spec</span>
    </h2>
    <p sx="text-muted-foreground text-sm leading-relaxed">
      Props: <code>alt: string</code> (reflect),
      <code>fit: contain|cover|actual</code> (reflect, default contain),
      <code>zoomable: bool</code> (reflect, default true),
      <code>gallery: bool</code> (reflect, default false).
      The resolved image source arrives via <code>prop:src</code> identity;
      the stage renders its empty state here. In gallery mode a
      <code>prop:src</code> array drives the thumbnail strip.
    </p>
    <column gap="4">
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">fit=contain zoomable (default)</span>
        <sail-media-image alt="Sample image — contain fit" fit="contain" zoomable></sail-media-image>
      </column>
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">fit=cover zoomable</span>
        <sail-media-image alt="Sample image — cover fit" fit="cover" zoomable></sail-media-image>
      </column>
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">fit=actual (no zoom)</span>
        <sail-media-image alt="Sample image — actual size" fit="actual"></sail-media-image>
      </column>
      <column gap="2" sx="border border-divider rounded-md p-3 bg-surface-2">
        <span sx="text-xs font-medium text-muted-foreground lowercase tracking-wide font-mono">gallery mode (strip, no src array wired)</span>
        <sail-media-image alt="Gallery demo" fit="contain" gallery></sail-media-image>
      </column>
    </column>
  </column>
</column>
