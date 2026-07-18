<!--
  /thesis — THE WORKING THESIS (pairing draft, 2026-07-18 →).
  This page is the tracked argument for the whole docs restructure. Edit in
  place; rulings from the pairing land here first, then propagate to the tree.
  Spine: human → LLM → compiler → long-lived artifact. The compiler is the
  guarantee layer. Control only where consistency is brand; full expression
  everywhere else; one blessed path per intent; tunable per team.
  Relationship to /why: /why argues the variance problem; this page states the
  system that answers it. IA decision pending: /why likely folds in here.
-->
<column sx="p-0 max-w-breakpoint-tablet mx-auto">
  <p sx="font-mono text-xs uppercase tracking-wide text-accent mb-2">Working thesis</p>
  <h1>A language between the model and the artifact</h1>

  <p sx="text-lg text-muted-foreground leading-relaxed">
    A person describes what they want. A model writes it. What comes out the
    other side has to be something a real brand can live on — consistent,
    themable, accessible, fast, maintainable across a real application with a
    real number of pages. SAIL is the layer that makes that true: between the
    model and the artifact sits a compiler — deterministic, traditional
    software — that turns generated code into a long-lived artifact with
    guarantees.
  </p>

  <h2>The pipeline</h2>
  <p>
    Human intent → LLM → <strong>compiler</strong> → artifact. Every serious
    attempt at agent-authored UI runs the first two steps and then hopes. The
    third step is the difference between output and infrastructure: the
    compiler is where hope becomes a contract. It is also the only step that
    can be <em>tuned</em> — the same compiler, dialed differently, produces a
    different language contract for a different team.
  </p>

  <h2>Where control belongs — and where it doesn't</h2>
  <p>
    The rules exist only where they make things better. One test decides:
    <strong>would a rebrand or retheme need to change this value?</strong>
    Yes — it is a token, and the compiler holds the line (color, spacing
    rhythm, type, radius). No — it is free (an animation's duration, a
    diagram's geometry, all logic). Control is for brand consistency, not
    restriction. When a rule makes the work worse, the rule is wrong — that is
    a bug in the language, and we treat it like one.
  </p>

  <h2>One blessed path</h2>
  <p>
    What rots an agent-written codebase is not CSS — it is <em>second
    paths</em>: the style block over here, the inline mutation over there,
    the class in a file no compiler ever sees. Invisible alternatives are
    where consistency dies and where agents burn turns guessing. So each
    intent has exactly one way: styling is sx, layout is elements, ambient
    motion is <code>motion:</code>, choreography is <code>timeline()</code>,
    theme writes go through <code>theme</code>. One path is teachable; a model
    compounds competency on it instead of relearning a house style per
    project.
  </p>

  <h2>Full expression inside the path</h2>
  <p>
    The path is not a cage. The standard is gorgeous — a real hero section
    from a world-class site, expressed exactly: multi-stop choreography,
    signal-driven scenes, springs, scroll. Where the language cannot express
    something beautiful, that is a gap in the language, not a discipline
    problem in the author. We find those gaps by recreating the real thing
    and closing what breaks.
  </p>

  <h2>Tunable, per team</h2>
  <p>
    Because the guarantee layer is a compiler, the contract is a setting, not
    a dogma. A brand platform runs strict: tokens only, no hatches. A personal
    site runs open. The dial belongs to the team that owns the artifact — and
    the tooling carries the complexity, so the settings never become a job in
    themselves.
  </p>

  <h2>The loop that makes it better</h2>
  <p>
    Every time an agent meets the compiler, the language learns something:
    what was written, what it should have been, how many turns it took. That
    stream — opt-in, honest, yours to switch off — is how the language gets
    better at being written, release over release. The compiler is not just
    the guarantee layer; it is the measurement instrument.
  </p>

  <h2>What this buys</h2>
  <p>
    A team ships a brand site where every page holds the system without a
    review gate. An agent one-shots a screen that would have been fifteen
    turns of correction. A rebrand is a data change. The artifact outlives
    the conversation that produced it — which is the entire point.
  </p>
</column>
