<!--
  <card> — L1 language element (element mode). The flagship surface: a
  generic container with a header/body/footer structure, tone-tinted
  border/rail/fill, an accent bar, and an interactive hover lift. Bare tag:
  it lowers away at compile time; the skin routes to the framework layer
  keyed on .sail-card; the contract feeds the element manifest.

  Faithful to the packages/components/sail/primitives/src/sail-card/
  baseline for tone/variant/padding/accent-bar/status-rail/fill. tone rides
  the global [tone] channel (contracts.css) — no per-value CSS here, same
  as the baseline's "NEW SURFACE" note.

  STRUCTURAL DIVERGENCE — header/footer slots: element mode's use-site
  splicing (parse/preprocess.rs) only projects a single DEFAULT slot today;
  named-slot projection ("<slot name=x>" picking out only the matching
  use-site children) is unimplemented compiler machinery, and extending it
  is out of scope for a stdlib-source-only promotion. Since element mode is
  light DOM (the tag is replaced by its structure snippet, no real shadow
  root), a use site's children splice in with their ORIGINAL attributes
  intact — including `slot="header"`/`slot="footer"`. So instead of real
  named slots, this skin selects on those attributes directly against the
  root's direct children: `:host > [slot='header']` / `:host > [slot=
  'footer']`. Every existing `<sail-card>` use site already writes
  slot="header"/"footer" markup (it was authoring for real named slots
  before) — that markup needs ZERO changes; only the tag rename applies.
  Presence-selection replaces the baseline's `reflect-empty` (a runtime
  slotchange listener, itself component-only / unavailable in element
  mode): a header/footer border only ever renders when a matching child
  exists, which the attribute selector already expresses for free.

  LANG-GAP — `interactive`'s a11y wiring: the baseline's controller sets
  role="button"/tabindex="0" on mount and an Enter/Space→click keydown
  listener (packages/components/.../sail-card.controller.ts). That is
  runtime JS — element mode is static structure, so it cannot ship here.
  The prop keeps its full CSS treatment (cursor, hover lift, active
  press, focus-visible ring — all static :host([interactive]) rules), but
  a use site that needs real keyboard-activatable semantics must add
  role="button" tabindex="0" and its own keydown handling itself (or slot
  in a real <button>/<a>). No current usage sets `interactive`, so this
  migrates clean today.
-->

<scope>
  props: {
    tone: 'default' | 'primary' | 'info' | 'success' | 'warning' | 'destructive' | 'accent' | 'muted' = 'default'
    variant: 'default' | 'outline' = 'default'
    padding: 'md' | 'none' | 'sm' | 'lg' = 'md'
    interactive = false
    'accent-bar': 'off' | 'hover' | 'static' = 'off'
    'status-rail' = false
    fill = false
  }
</scope>

<style>
  .sail-card {
    display: block;
    position: relative;
    background: var(--surface-1);
    color: var(--card-foreground);
    border: 1px solid var(--border);
    --card-radius: var(--radius-lg);
    border-radius: var(--card-radius);
    overflow: clip;
    /* Frost on glass themes (--card-backdrop-filter resolves to a blur there,
       `none` everywhere else). isolation establishes a stacking context so the
       backdrop-filter clips to --card-radius — belt-and-suspenders with the
       existing overflow:clip. */
    backdrop-filter: var(--card-backdrop-filter);
    -webkit-backdrop-filter: var(--card-backdrop-filter);
    isolation: isolate;
    box-shadow: var(--shadow-card-rest);
    --card-px: var(--space-5);
    --card-py: var(--space-4);
    --card-gap: var(--space-3);
    padding: var(--card-py) var(--card-px);
    transition:
      background-color var(--duration-fast) var(--ease-out),
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out),
      transform var(--duration-fast) var(--ease-out);
  }

  /* variant=outline: background:transparent is a raw CSS keyword, not a
     token — belongs in <style>, not the sx vocabulary. */
  .sail-card[variant='outline'] { background: transparent; }

  /* Tone-bound border. Default tone resolves --tone-border to --border (the
     same as the host baseline), so the override is a no-op for it. */
  .sail-card[tone]:not([tone='default']) { border-color: var(--tone-border); }

  /* Accent bar. hover fades in on hover; static stays at full opacity
   * (hero cards, print exports, touch contexts where hover never fires). */
  .sail-card[accent-bar='hover']::before,
  .sail-card[accent-bar='static']::before {
    content: '';
    position: absolute;
    top: 0; left: 0; right: 0;
    height: 2px;
    background: linear-gradient(90deg, var(--primary), var(--accent));
    opacity: 0;
    border-radius: var(--card-radius) var(--card-radius) 0 0;
    transition: opacity var(--duration-medium) var(--ease-out);
    pointer-events: none;
  }
  .sail-card[accent-bar='hover']:hover::before { opacity: 1; }
  .sail-card[accent-bar='static']::before { opacity: 1; }

  .sail-card[interactive] { cursor: pointer; -webkit-tap-highlight-color: transparent; }
  .sail-card[interactive]:hover {
    border-color: var(--border-strong);
    box-shadow: var(--shadow-hover);
    transform: translateY(-1px);
  }
  @media (prefers-reduced-motion: reduce) {
    .sail-card[interactive]:hover { transform: none; }
  }
  .sail-card[interactive]:active {
    box-shadow: var(--shadow-card-rest);
    transform: translateY(0);
    transition-duration: 60ms;
  }
  .sail-card[interactive]:focus-visible {
    outline: var(--focus-ring);
    outline-offset: var(--focus-ring-offset);
  }

  /* Tone-rail. Only meaningful once a tone is set — otherwise --tone-fg
   * resolves to --foreground and the rail would read as body-text-colored
   * decoration with the weight of an emphasis cue. */
  .sail-card[status-rail] {
    border-left: 3px solid var(--tone-fg);
    padding-inline-start: calc(var(--card-px) - var(--space-1));
  }
  .sail-card[status-rail]:not([tone]),
  .sail-card[status-rail][tone='default'] {
    border-left-color: var(--border-strong);
  }

  .sail-card[fill] { background: var(--tone-bg); border-color: var(--tone-border); }

  /* Per-padding local-var recompute + fallback padding. */
  .sail-card[padding='none'] { --card-px: 0px; --card-py: 0px; --card-gap: 0px; padding: 0; }
  .sail-card[padding='sm']   { --card-px: var(--space-3); --card-py: var(--space-2); --card-gap: var(--space-2); padding: var(--space-2) var(--space-3); }
  .sail-card[padding='md']   { --card-px: var(--space-5); --card-py: var(--space-4); padding: var(--space-4) var(--space-5); }
  .sail-card[padding='lg']   { --card-px: var(--space-6); --card-py: var(--space-5); --card-gap: var(--space-4); padding: var(--space-5) var(--space-6); }

  /* Header/footer — presence-selected direct children (see the structural
     divergence note above), edge-to-edge via negative margins that cancel
     the host's own padding, separated by a hairline. */
  .sail-card > [slot='header'] {
    display: block;
    margin: calc(var(--card-py) * -1) calc(var(--card-px) * -1) var(--card-gap);
    padding: var(--card-py) var(--card-px);
    border-bottom: 1px solid var(--border-subtle);
    font-family: var(--heading-font);
    font-weight: var(--heading-weight);
  }
  .sail-card > [slot='footer'] {
    display: block;
    margin: var(--card-gap) calc(var(--card-px) * -1) calc(var(--card-py) * -1);
    padding: var(--card-py) var(--card-px);
    border-top: 1px solid var(--border-subtle);
  }
  .sail-card[padding='none'] > [slot='header'] {
    margin: 0 0 var(--space-3);
    padding: 0 0 var(--space-3);
  }
  .sail-card[padding='none'] > [slot='footer'] {
    margin: var(--space-3) 0 0;
    padding: var(--space-3) 0 0;
  }

  @media (forced-colors: active) {
    .sail-card[interactive]:focus-visible { outline: 2px solid CanvasText; outline-offset: 2px; }
  }
</style>

<div>
  <slot></slot>
</div>
