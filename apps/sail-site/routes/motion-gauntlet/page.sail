<!--
  /motion-gauntlet — THROWAWAY. A pixel-for-pixel recreation of motion.dev's
  "Animations that move." feature atlas (the `home-diag` diagrams), built to
  stress-test sail's motion + sx vocabulary. Disposable; do not link from nav.

  Ground truth captured in
  helm/data/missions/m-deck-standalone/scratch/motion-capture/. The 8 declarative
  WAAPI animations live in three cells:
    • cell 01 (Independent transforms) — the `tile` 7-stop compound loop
    • cell 06 (Exit animation)        — 3 `rows`, slide+fade, staggered
    • cell 07 (Timeline sequences)    — 4 `bars`, scaleX fill, staggered
  The other five cells are canvas / FLIP / spring / drag driven by JS in the
  original (imperative, not the `motion:` surface) — rendered here statically as
  visual scaffold, with light in-vocab motion where a preset covers it honestly.

  Theme pinned to `contrast` (near-black ground + amber/yellow accent, blocky
  radii, bold weights) — the stdlib theme closest to motion.dev's "fresh" look.
-->

<column data-theme-preview="contrast" sx="bg-background text-foreground p-12 min-h-screen" gap="8">

  <!-- ── Section header ─────────────────────────────────────────────── -->
  <column sx="max-w-breakpoint-desktop mx-auto w-full" gap="3">
    <p sx="font-mono text-xs uppercase tracking-wider text-muted-foreground">Features</p>
    <h2 sx="text-4xl font-bold tracking-tight">Animations that move.</h2>
    <p sx="text-lg text-muted-foreground leading-relaxed max-w-breakpoint-tablet">
      Create high-performance web animations with a small, declarative API — from
      simple transforms to advanced interactive gestures.
    </p>
  </column>

  <!-- ── Feature atlas: 4 × 2 ───────────────────────────────────────── -->
  <grid cols="4" gap="4" sx="max-w-breakpoint-desktop mx-auto w-full">

    <!-- 01 · Independent transforms · the TILE (7-stop compound loop) ── -->
    <column sx="bg-surface-1 border border-border rounded-md p-5 overflow-hidden" gap="4">
      <stack sx="relative h-40 rounded-sm bg-surface-0 overflow-hidden">
        <!-- axes -->
        <span sx="absolute left-6 top-6 bottom-6 w-size-px bg-border-subtle"></span>
        <span sx="absolute left-6 right-6 bottom-6 h-size-px bg-border-subtle"></span>
        <!-- ghost outline at rest position -->
        <span sx="absolute left-8 bottom-12 size-10 rounded-sm border border-dashed border-border-subtle"></span>
        <!-- the tile — cell 01: three INDEPENDENT transform channels on one
             element (x + rotate + scale), each its own preset & duration.
             GAP: the original's 7-stop timeline SEQUENCES them (x settles, THEN
             scale pops) with holds; 2-stop presets run them in parallel. -->
        <span data-diag="tile" sx="absolute left-8 bottom-12 size-10 rounded-sm bg-hue-yellow origin-center"
              motion:loop={{ translate-x: 'drift', rotate: 'tilt', scale: 'pop' }}></span>
      </stack>
      <column gap="2">
        <span sx="font-mono text-xs text-muted-foreground">01</span>
        <row justify="between" align="center">
          <h3 sx="text-base font-semibold">Independent transforms</h3>
          <icon name="arrow-right" size="sm" sx="text-muted-foreground"></icon>
        </row>
        <p sx="text-sm text-muted-foreground leading-relaxed">
          Animate <code>x</code>, <code>y</code>, <code>rotate</code>, <code>scale</code>
          on one element, no wrappers.
        </p>
        <p sx="font-mono text-xs text-hue-yellow">{'{ rotate: 15, x: "50%" }'}</p>
      </column>
    </column>

    <!-- 02 · Scroll animation (accent cell, static canvas → ring) ────── -->
    <column sx="bg-hue-yellow text-background rounded-md p-5 overflow-hidden" gap="4">
      <center sx="relative h-40 rounded-sm">
        <span sx="size-24 rounded-full border-4 border-background opacity-30"></span>
        <span sx="absolute size-24 rounded-full border-4 border-t-transparent border-r-transparent border-background"></span>
      </center>
      <column gap="2">
        <span sx="font-mono text-xs opacity-70">02</span>
        <row justify="between" align="center">
          <h3 sx="text-base font-semibold">Scroll animation</h3>
          <icon name="arrow-right" size="sm"></icon>
        </row>
        <p sx="text-sm leading-relaxed opacity-80">
          Hardware-accelerated scroll-linked motion via <code>ScrollTimeline</code>.
        </p>
        <p sx="font-mono text-xs">scroll()</p>
      </column>
    </column>

    <!-- 03 · Native gestures (static) ────────────────────────────────── -->
    <column sx="bg-surface-1 border border-border rounded-md p-5 overflow-hidden" gap="4">
      <center sx="relative h-40 rounded-sm bg-surface-0">
        <span sx="size-16 rounded-md bg-surface-3 border border-border"></span>
      </center>
      <column gap="2">
        <span sx="font-mono text-xs text-muted-foreground">03</span>
        <row justify="between" align="center">
          <h3 sx="text-base font-semibold">Native gestures</h3>
          <icon name="arrow-right" size="sm" sx="text-muted-foreground"></icon>
        </row>
        <p sx="text-sm text-muted-foreground leading-relaxed">
          <code>hover</code>, <code>press</code>, and <code>drag</code> that feel native.
        </p>
        <p sx="font-mono text-xs text-hue-yellow">drag = true</p>
      </column>
    </column>

    <!-- 04 · Layout animation (static FLIP snapshot) ─────────────────── -->
    <column sx="bg-surface-1 border border-border rounded-md p-5 overflow-hidden" gap="4">
      <center sx="relative h-40 rounded-sm bg-surface-0">
        <grid cols="2" gap="2">
          <span sx="w-16 h-8 rounded-sm bg-surface-3"></span>
          <span sx="w-16 h-8 rounded-sm bg-surface-3"></span>
          <span sx="w-16 h-8 rounded-sm bg-surface-3"></span>
          <span sx="w-16 h-8 rounded-sm bg-surface-3"></span>
        </grid>
      </center>
      <column gap="2">
        <span sx="font-mono text-xs text-muted-foreground">04</span>
        <row justify="between" align="center">
          <h3 sx="text-base font-semibold">Layout animation</h3>
          <icon name="arrow-right" size="sm" sx="text-muted-foreground"></icon>
        </row>
        <p sx="text-sm text-muted-foreground leading-relaxed">
          Animate between layouts with a single <code>layout</code> flag.
        </p>
        <p sx="font-mono text-xs text-hue-yellow">layout = true</p>
      </column>
    </column>

    <!-- 05 · Spring physics (ball — in-vocab bounce) ─────────────────── -->
    <column sx="bg-surface-1 border border-border rounded-md p-5 overflow-hidden" gap="4">
      <center sx="relative h-40 rounded-sm bg-surface-0">
        <span sx="absolute bottom-6 left-6 right-6 h-size-px bg-border-subtle"></span>
        <!-- in-vocab: `bounce` preset (translate-y, infinite alternate) -->
        <span data-diag="ball" sx="size-8 rounded-full bg-hue-yellow"
              motion:loop={{ translate-y: 'bounce' }}></span>
      </center>
      <column gap="2">
        <span sx="font-mono text-xs text-muted-foreground">05</span>
        <row justify="between" align="center">
          <h3 sx="text-base font-semibold">Spring physics</h3>
          <icon name="arrow-right" size="sm" sx="text-muted-foreground"></icon>
        </row>
        <p sx="text-sm text-muted-foreground leading-relaxed">
          Real <code>spring</code> math that reacts naturally to input.
        </p>
        <p sx="font-mono text-xs text-hue-yellow">type = "spring"</p>
      </column>
    </column>

    <!-- 06 · Exit animation · the ROWS (slide+fade, staggered) ───────── -->
    <column sx="bg-surface-1 border border-border rounded-md p-5 overflow-hidden" gap="4">
      <!-- cell 06: 3 rows slide-left + fade, staggered. translate-x + opacity
           are independent motions (same `amble` duration → in phase); `stagger`
           gives the per-row 450ms offset. The 3 spans share this parent, so the
           stagger index runs 0,1,2 across them. -->
      <column sx="relative h-40 rounded-sm bg-surface-0 p-5" gap="3">
        <span data-diag="row" sx="h-6 rounded-sm bg-surface-3 w-full" motion:loop={{ translate-x: 'slip', opacity: 'dim', stagger: '450ms' }}></span>
        <span data-diag="row" sx="h-6 rounded-sm bg-surface-3 w-full" motion:loop={{ translate-x: 'slip', opacity: 'dim', stagger: '450ms' }}></span>
        <span data-diag="row" sx="h-6 rounded-sm bg-surface-3 w-full" motion:loop={{ translate-x: 'slip', opacity: 'dim', stagger: '450ms' }}></span>
      </column>
      <column gap="2">
        <span sx="font-mono text-xs text-muted-foreground">06</span>
        <row justify="between" align="center">
          <h3 sx="text-base font-semibold">Exit animation</h3>
          <icon name="arrow-right" size="sm" sx="text-muted-foreground"></icon>
        </row>
        <p sx="text-sm text-muted-foreground leading-relaxed">
          Elements stay alive so they can animate as they leave the DOM.
        </p>
        <p sx="font-mono text-xs text-hue-yellow">exit = {'{...}'}</p>
      </column>
    </column>

    <!-- 07 · Timeline sequences · the BARS (scaleX fill, staggered) ──── -->
    <column sx="bg-hue-yellow text-background rounded-md p-5 overflow-hidden" gap="4">
      <!-- cell 07: 4 horizontal bars fill left→right via the NEW `scale-x`
           motion property (origin-left), staggered 120ms. Two stacked layers:
           static tracks behind, animated fills in front — the 4 fills share one
           parent so `stagger` indexes them 0..3. -->
      <stack sx="relative h-40 rounded-sm p-5">
        <column sx="absolute inset-5" gap="4">
          <span sx="h-4 w-full rounded-sm bg-background opacity-20"></span>
          <span sx="h-4 w-full rounded-sm bg-background opacity-20"></span>
          <span sx="h-4 w-full rounded-sm bg-background opacity-20"></span>
          <span sx="h-4 w-full rounded-sm bg-background opacity-20"></span>
        </column>
        <column sx="absolute inset-5" gap="4">
          <span data-diag="bar" sx="h-4 w-full rounded-sm bg-background origin-left" motion:loop={{ scale-x: 'fill', stagger: '120ms' }}></span>
          <span data-diag="bar" sx="h-4 w-full rounded-sm bg-background origin-left" motion:loop={{ scale-x: 'fill', stagger: '120ms' }}></span>
          <span data-diag="bar" sx="h-4 w-full rounded-sm bg-background origin-left" motion:loop={{ scale-x: 'fill', stagger: '120ms' }}></span>
          <span data-diag="bar" sx="h-4 w-full rounded-sm bg-background origin-left" motion:loop={{ scale-x: 'fill', stagger: '120ms' }}></span>
        </column>
      </stack>
      <column gap="2">
        <span sx="font-mono text-xs opacity-70">07</span>
        <row justify="between" align="center">
          <h3 sx="text-base font-semibold">Timeline sequences</h3>
          <icon name="arrow-right" size="sm"></icon>
        </row>
        <p sx="text-sm leading-relaxed opacity-80">
          <code>stagger</code> and timelines orchestrate complex motion.
        </p>
        <p sx="font-mono text-xs">stagger(0.04)</p>
      </column>
    </column>

    <!-- 08 · Motion values (static) ─────────────────────────────────── -->
    <column sx="bg-surface-1 border border-border rounded-md p-5 overflow-hidden" gap="4">
      <center sx="relative h-40 rounded-sm bg-surface-0">
        <span sx="absolute left-6 right-6 top-half h-size-px bg-border-subtle"></span>
        <!-- in-vocab: a derived value continuously mapped → `spin` (rotate) -->
        <span data-diag="mark" sx="size-10 rounded-full border-2 border-hue-yellow border-t-transparent"
              motion:loop={{ rotate: 'spin' }}></span>
      </center>
      <column gap="2">
        <span sx="font-mono text-xs text-muted-foreground">08</span>
        <row justify="between" align="center">
          <h3 sx="text-base font-semibold">Motion values</h3>
          <icon name="arrow-right" size="sm" sx="text-muted-foreground"></icon>
        </row>
        <p sx="text-sm text-muted-foreground leading-relaxed">
          Track and derive animated values outside React's render loop.
        </p>
        <p sx="font-mono text-xs text-hue-yellow">useMotionValue()</p>
      </column>
    </column>

  </grid>
</column>
