# tree-sitter-sail

Tree-sitter grammar for the SAIL `.sail` template
language.

## Status

V2. Corpus + a battery of real `.sail` sources across the repo parse
with no ERROR nodes. ABI 15. Block/clause vocabulary is checked
against the compiler's `block_manifest::BLOCKS` table by
`bindings/rust/tests/block_vocabulary.rs` so this grammar can't
silently re-drift from `crates/sail/compiler`.

## What the grammar parses

`.sail` is HTML extended with template grammar. The grammar
recognises:

- **Directive elements** — `<scope>` (static props/types) (just
  an HTML element as far as the grammar is concerned; the runtime
  extracts it by tag name; queries flag it visually).
- **Standard HTML elements** — both the explicit-close form
  (`<div>…</div>`) and the self-closing form (`<ship-icon …/>`),
  plus the void-element forms (`<link>`, `<br>`, `<img>`, …).
- **Binding attributes** — `bind:value={x}`, `on:event[|mod]*={fn}`,
  `use:action` / `use:action={param}`, `fx:trigger={…}`.
  Plain attributes (`name="x"`, `name={expr}`, `sx="…"`, bare-name
  boolean attrs) round-trip through their own rules.
- **Brace interpolation** — `{expr}` in text and attribute-value
  position. The expression body is captured as an opaque balanced
  atom stream; host-language highlighting comes from a TypeScript
  injection (`queries/injections.scm`).
- **Control-flow blocks** (mirrors
  `crates/sail/compiler/src/block_manifest.rs`)
  - `{#if cond}` / `{:else if cond}` / `{:else}` / `{/if}`
  - `{#for x of items key={x.id}}` / `{:else}` / `{/for}` (key is
    optional; the item binding is a bare identifier or a
    parenthesized `(item, index)` tuple — a bare comma list without
    parens is not valid)
  - `{#match value}` / `{:case 'a'}` / `{:default}` / `{/match}`
  - `{#while-loading of signal}` / `{/while-loading}` — pending state
    of an async query
  - `{#while-error errVar of signal}` / `{/while-error}` — rejected
    state of an async query, binding the rejection to `errVar`
- **HTML comments** `<!-- … -->`.

### Deliberately NOT recognised (retired/refused surface)

- `style:` / `prop:` / `attr:` / `class:` attrs — compiler hard-refuses
  (`parse/template/attrs.rs`); plain `name={expr}` is field binding.
- `let:` attrs — never implemented; slot/async bindings sit on the
  block header (`{#while-error err of x}`).
- `<state>`, `<component>`, `<prop>` directive elements — not language.
- Element-form `<while-loading>` / `<while-error>` — brace blocks only.
- `{:elif}` — only `{:else if}` is compiler-recognized.

## Architecture

The grammar is a thin HTML-shaped outer layer plus the template
extensions. Expression bodies (inside `{…}`) are tokenised as
balanced atom sequences so the parser doesn't re-implement
TypeScript — `queries/injections.scm` routes those ranges to
`tree-sitter-typescript`. Inline `<style>…</style>` content is
similarly injected to `tree-sitter-css`.

Two tokeniser carve-outs keep the host language and the framework
keywords disambiguated:

- `_member` matches `.foo` as one token so member access can't be
  mis-tokenised as `.` + identifier (which would let keyword
  extraction trigger on names like `.key`, `.of`, …).
- The for-loop iterator body uses an `_iter_punct` rule that excludes
  a top-level `=` so the optional `key={…}` continuation peels off
  cleanly.

## Build

```bash
cd packages/sail/grammar
npx tree-sitter generate --js-runtime native   # writes src/parser.c
npx tree-sitter test                           # runs the corpus
```

`--js-runtime native` uses tree-sitter's built-in QuickJS runtime
instead of shelling out to a `node` binary — needed in environments
where `node` isn't a real Node.js install (e.g. only Bun's `node`
shim is on `PATH`). Drop the flag if a real Node.js is available.

## Validation

Parse a battery of real `.sail` sources (site routes, stdlib
elements, pack components) and confirm none produce an ERROR node:

```bash
cd packages/sail/grammar
fail=0
while IFS= read -r f; do
  npx tree-sitter parse --quiet --stat total-only "$f" | grep -q ERROR && { echo "FAIL: $f"; fail=1; }
done < <(find ../../.. -name '*.sail' -not -path '*/node_modules/*')
exit $fail
```

Expected: no `FAIL:` lines.

## Known v3 work

- **External scanner.** Multi-line balanced expressions and
  raw-text element bodies (`<style>`, `<controller>`, `<script>`
  without `src`) rely on a regex/precedence approximation. `<style>`
  CSS and `<controller>` TypeScript bodies are NOT raw-text today —
  they're walked as ordinary template nodes, so CSS/TS punctuation
  (`<` combinators, generic `Foo<Bar>`, unbalanced braces across
  statements, …) routinely produces ERROR nodes on real sources. An
  external C scanner would let both element bodies span to their
  matching close tag as one opaque raw-text token, and would also
  handle template literals containing `` ${`}` `` (a backtick inside
  `${…}` inside a backtick — none in the corpus today).
- **`queries/locals.scm`.** Would expose declaration scopes
  (`<scope>` props, `{#while-error err of x}`'s `err` binding) for
  jump-to-def in editors that consume the `@local.*` family.
  Currently absent.
- **HTML attribute namespaces.** `attr_name` excludes `:` so the
  `bind:` / `on:` / `use:` / `fx:` directives win the lexer race;
  the corpus has no SVG-style namespaced attributes (`xlink:href`, …)
  so this is safe today but worth revisiting if SVG lands as a
  first-class template surface.
