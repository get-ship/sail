<!--
  /why — the argument, written fresh 2026-07-18 (owner + Claude pairing).
  Shape: (1) the author changed — agents AND newcomers; the decision gauntlet;
  (2) the one-shot cliff — the bet; (3) the missing layer is a compiler;
  (4) built from the platform — ARIA/best practices by construction;
  (5) where it came from — Ship, integration, a toolchain without the node
  ecosystem. Prior page mined as input only; absorbed the thesis (/thesis is
  gone). Voice: plain, confident, zero showmanship, no unverified numbers.
-->
<column sx="max-w-breakpoint-tablet mx-auto" gap="4">
  <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2">Why Sail</p>
  <h1>The author changed</h1>

  <p sx="text-lg text-muted-foreground leading-relaxed">
    Every mainstream web framework is an ergonomics project — built to fit a
    professional's hands. Hooks, hot reload, clever DX: all of it optimizes
    for a person who already knows the territory. Two things happened to that
    assumption at once. Agents started writing a large share of the code. And
    AI put website-building in front of people who have never written any —
    for whom the framework was never the hard part.
  </p>
  <p>
    Watch what happens to a newcomer today. Before a single element is on
    screen, they are routed through a gauntlet of expert decisions: rendering
    strategy, framework, bundler, data-fetching library, component kit — and
    then styling, where there are genuinely hundreds of paths. Each library
    is excellent in isolation and optimized for a different goal, and making
    them agree is its own skill that has nothing to do with the website. The
    naive alternative — plain HTML and CSS — gets surprisingly far, and then
    falls apart exactly when the project starts to matter. Agents don't need
    the ergonomics; they need determinism. Newcomers don't need the decision
    tree; they need it gone. Sail is one system where those are the same
    answer.
  </p>

  <h2>The one-shot cliff</h2>
  <p>
    Agents are genuinely good at page one. A single prompt produces a
    handsome landing page, and it is easy to conclude the problem is solved.
    Then you ask for page twelve. A second spacing rhythm appears, a fourth
    shade of the brand colour, two animation approaches, state held three
    ways. Nothing malfunctioned — every decision a language leaves open is a
    place where two generations can differ, and the differences compound
    with every file.
  </p>
  <p>
    That is the bet Sail makes: consistency cannot be a review step or a
    prompt instruction, because variance is a property of the language, not
    of the author's skill. Close the decisions — one way to hold state, one
    way to style, one way to animate — and the five-hundredth page holds the
    system as well as the first, no matter who or what wrote it.
  </p>

  <h2>The missing layer is a compiler</h2>
  <p>
    A person describes, a model writes — and in most stacks, that's the whole
    pipeline. There is no oracle: a wrong program renders, plausibly, and
    ships. You find out in review, in production, or never. Sail puts a
    compiler between the model and the artifact. Off-system styling, a
    stylesheet beside a component, an unknown motion trigger — these fail the
    build, and every refusal names the correct form in the language's own
    vocabulary. A model doesn't need to have memorized a million lines of
    Sail; it needs to be correctable in one turn, and that is a property we
    can design.
  </p>
  <p>
    The same compiler is a contract each team tunes. A brand platform runs
    strict — every colour a token, raw containers rare. A weekend project
    opens the dials. What ships is guaranteed either way, because the
    guarantee is machinery, not memory.
  </p>

  <h2>Built from the platform</h2>
  <p>
    Sail doesn't wrap the browser — it compiles to it. A dialog is
    <code>&lt;dialog&gt;</code>. A menu is a native <code>popover</code>.
    Components compile to real markup and a stylesheet, so what the model
    writes is the HTML it already knows, and what a user gets is keyboard
    handling, focus management, and ARIA that come with the element — by
    construction, not because the author remembered. Best practice isn't a
    checklist here; it's the only form the language offers. The same holds
    for design: colours, spacing, and type are tokens, themes are data, and
    a rebrand is a data change rather than a rewrite.
  </p>

  <h2>Part of SHIP</h2>
  <p>
    Sail is part of the SHIP ecosystem. SHIP is a consumer platform for
    building local-first AI applications, and Sail is one of its core
    technologies — the front end of the platform. That origin set a hard
    requirement: the framework had to run on a portable toolchain — no Node
    dependency, a compiler that runs in the browser (this site compiles Sail
    live in your tab) — and that constraint shaped many of the language's
    design decisions.
  </p>

  <p>
    Sail will feel constrained if you came looking for a flexible toolkit to
    hand-write — and it should. The constraint is the product. It is what
    turns "an agent built this" from a warning into a guarantee.
  </p>

  <row sx="mt-8 pt-4 border-t border-t-border" gap="2">
    <a variant="solid" tone="primary" href="/getting-started">Installation →</a>
    <a variant="default" href="/overview">Introduction</a>
    <a variant="default" href="/tutorial">Tutorial</a>
  </row>
</column>
