# Behavioral spec format

A behavioral spec is the runnable form of "what should this view *do*," written
the way a user describes it — in terms of **what they see and what they click**,
never CSS classes, signal names, or controller internals. It is:

- the **target** an authoring agent builds against (implement until green),
- the **score** the eval reports (% scenarios passing),
- derived from a **prompt + light structure**, never hand-written by end users.

It runs in **one substrate** — the Ship webview (real DOM, real controller),
with `ship` data **faked at the SDK boundary** so runs are deterministic. No
node, no bun, no separate test engine.

## Why query by what the user perceives

Scenarios locate elements by **visible text, role, and label** — not selectors.
Two reasons: (1) it's the user's mental model, so prompt→spec translation is
natural; (2) it's robust to implementation variance, which an eval *needs* —
two agents can build the same behavior with different markup and both pass.

## Schema

```jsonc
{
  "view": "create-space",          // the view under test
  "scenarios": [
    {
      "name": "welcome advances to naming",   // user-language
      "given": {                               // optional setup
        "url": "/create-space",                // entry route
        "ship": { "git.overview": { "branch": "main" } }  // faked ops → values
      },
      "steps": [
        { "see": "Create your first space" },  // visible text present
        { "click": "Create your first space" },// click by visible label/role
        { "see": "Brand Voice" },              // step 1 default is shown
        { "notSee": "what a Space is" }         // welcome copy is gone
      ]
    }
  ]
}
```

### Step verbs (the whole vocabulary — keep it small)

| Verb | Meaning | Form |
|---|---|---|
| `see` | visible text / control is present | `"text"` or `"role:button name:Continue"` |
| `notSee` | absent or not visible | same as `see` |
| `click` | activate a control by its visible label/role | `"Continue"` |
| `fill` | type into a labeled input | `{ "field": "Name", "value": "My Space" }` |
| `choose` | pick one option in a labeled single-select group | `{ "option": "Dark", "in": "Theme" }` |

`given` is per-scenario setup, not a step: `url` sets the entry route; `ship`
maps `domain.op` → the value a faked `<data>`/`live()` resolves to.

A scenario **passes** when every step is satisfiable against the real DOM, with
a flush between steps so reactive effects settle.

## Prompt → spec (the light structure)

The create flow collects intent. Even fully freeform works; the lightest useful
structure is three lists:

- **shows** — text/sections the view presents
- **does** — interactions and their visible outcome ("click X → see Y")
- **needs** — data the view reads (→ becomes `given.ship` fakes)

Each "does" line is one scenario; each "shows" line is a `see` step. That's the
entire translation. Mockups, when present, only enrich labels — the spec keys
off named text + controls, so **a text-only prompt yields the same spec**.

See `create-space.behavior.jsonc` (this task, mockup-derived) and the
mockup-free example below.

## Mockup-free example

Prompt: *"a counter — a big number starting at 0 and a + button that bumps it."*

Light structure → spec:

```jsonc
{
  "view": "counter",
  "scenarios": [
    { "name": "starts at zero", "steps": [{ "see": "0" }] },
    {
      "name": "plus increments",
      "steps": [
        { "see": "0" },
        { "click": "role:button name:+" },
        { "see": "1" }
      ]
    }
  ]
}
```

No picture needed — the prompt named the number, the button, and the outcome.
