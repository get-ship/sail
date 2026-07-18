<!--
  /why — the argument. Spine: the bottleneck in agent-authored UI is decisions,
  not reactivity; every open decision is variance; variance is where one-shot
  beauty dies. So remove the decisions and keep the expression. Evidence, not
  assertion — every claim on this page is a number we can produce on demand.
  Do not dunk on competitors; the honest section is load-bearing for trust.
-->
<column sx="p-0 max-w-breakpoint-tablet mx-auto">
  <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2">Introduction</p>
  <h1>Why SAIL</h1>
  <p sx="text-lg text-muted-foreground leading-relaxed">
    SAIL is built to be written by agents. Not "agent-friendly" as a feature —
    agents are the assumed author, and every decision in the language follows
    from that. The goal is to give a model 95% of the expression that code
    offers with 10% of the decisions.
  </p>

  <h2>The web, with default styles</h2>
  <p>
    Component libraries exist because nobody owns HTML: the platform's
    vocabulary froze decades ago, so every team rebuilds the missing patterns
    — dialogs, comboboxes, toasts — in userspace, forever, with constant
    governance and constant drift. We own our platform: the compiler, the
    lint, the LSP, the runtime. So the missing tier isn't a library on top —
    it is the language. A combobox is a tag the way a paragraph is a tag,
    with the behavior written once in the toolchain, its anatomy published as
    a contract, and a default skin any theme can replace wholesale. Remove
    the userspace tier and you remove its failure mode.
  </p>

  <h2>The bottleneck was never reactivity</h2>
  <p>
    Hand a model a spec and it makes hundreds of small choices on the way to a
    screen: a color, a spacing step, a focus ring, whether that dropdown is
    genuinely reachable by keyboard. It gets them confidently, plausibly wrong.
    And when it isn't sure, it falls back on what it knows best — a
    <code>&lt;div&gt;</code> and a class name. Give it a library of a hundred
    components and it will route around the library, because HTML is the surface
    it is fluent in.
  </p>
  <p>
    So <strong>every decision a language leaves open is variance</strong>, and
    variance is where one-shot beauty dies. Two agents diverge. The same agent
    diverges from itself on Tuesday. One team's animation library meets another
    team's. Preference leaks into state management. None of that is a skill
    problem you can prompt your way out of — it's a property of the language.
  </p>

  <h2>So the decisions are gone</h2>
  <p>
    Not discouraged, not linted with a warning — <em>absent</em>. There is one
    way to hold state, one way to animate, one way to style, one component
    library, and it is made of tags the model already knows.
  </p>
  <ul sx="list-none p-0">
    <li>
      <strong>The component library is the platform.</strong> A dialog is
      <code>&lt;dialog&gt;</code>. A menu is <code>[popover]</code>. There are 48
      L1 elements and they compile to native markup plus a stylesheet — so the
      model writes the HTML it already knows, and gets the keyboard handling,
      the ARIA, and the focus management for free.
    </li>
    <li>
      <strong>Styling is a subset, on purpose.</strong> <code>sx</code> is a
      deliberately small slice of CSS keyed to the design system. An off-system
      color is a build error, not a review note. There is no second styling
      system to choose, so there is nothing to choose.
    </li>
    <li>
      <strong>One animation system.</strong> <code>motion:</code>, eight
      triggers, nine presets. You cannot reach for GSAP instead, so no two
      screens move differently.
    </li>
    <li>
      <strong>One way to reach an element.</strong> Behavior attaches with
      <code>use:</code>. There is no ref to thread, no imperative escape hatch
      to be tempted by.
    </li>
  </ul>

  <h2>The compiler is the feedback loop</h2>
  <p>
    This is the part that matters most, and it's the part that's easy to miss.
    An agent writing React gets no oracle: a wrong program <em>renders</em>,
    plausibly, and ships. An agent writing SAIL gets told exactly what is wrong,
    in the language's own vocabulary, the moment it writes it — an unknown
    motion trigger, a color that isn't on the system, a layout that should have
    been a primitive.
  </p>
  <p>
    Agents run in a loop. A language that can <em>correct</em> them in one turn
    beats a language they happen to have memorized. We don't need the model to
    have read a million lines of SAIL. We need it to be correctable.
  </p>

  <h2>What the constraint bought</h2>
  <p>
    The point of giving things up is getting something back — and what comes
    back is the ability to <em>trust</em> the output without reading every
    line:
  </p>
  <ul sx="list-none p-0">
    <li>
      <strong>Consistency you can check.</strong> Every element renders in
      every state across every theme on every commit, driven in a real
      browser — the design system is enforced by machinery, not memory.
    </li>
    <li>
      <strong>Accessibility by construction.</strong> Not because the author
      was careful; because the element already was, and the language offers
      no inaccessible way to write the screen.
    </li>
    <li>
      <strong>Performance by architecture.</strong> Templates compile ahead
      of time, updates are fine-grained signal writes, and components
      compile away to native markup — there is no virtual DOM and no
      framework wrapper in the output to pay for.
    </li>
  </ul>

  <h2>The honest version</h2>
  <p>
    Nobody can tell you what an agent-generated React app <em>is</em>. It might
    be accessible. It might be on-brand. It might have invented four shades of
    blue and three ways to open a modal. You find out in review, or in
    production, or never. That is the trade the ecosystem made when it optimized
    for human preference: a tool that is pleasant to hand-write and impossible
    to guarantee.
  </p>
  <p>
    SAIL takes the other side. It will feel constrained if you came looking for a
    flexible toolkit to hand-write — and it should. That constraint is the
    product. It is what turns "an agent built this" from a warning into a
    guarantee.
  </p>

  <row sx="mt-8 pt-4 border-t border-t-border" gap="2">
    <a variant="default" href="/overview">← Introduction</a>
    <a variant="solid" tone="primary" href="/components">Component library →</a>
  </row>
</column>
