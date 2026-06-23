# tree-sitter-set

Tree-sitter grammar for the SUI `.html.set` / `.set` template
language.

## Status

V2. Conformance corpus passes against `views/agents/view.html.set`
with no ERROR nodes. ABI 15.

## What the grammar parses

`.html.set` is HTML extended with template grammar. The grammar
recognises:

- **Directive elements** — `<data>`, `<component>`, `<prop>`,
  `<while-loading>`, `<while-error>` (just HTML elements as far as
  the grammar is concerned; the runtime extracts them by tag name;
  queries flag them visually).
- **Standard HTML elements** — both the explicit-close form
  (`<div>…</div>`) and the self-closing form (`<ship-icon …/>`),
  plus the void-element forms (`<link>`, `<br>`, `<img>`, …).
- **Binding attributes** — `bind:value={x}`, `on:event[|mod]*={fn}`,
  `class:active={cond}`, `style:--tone={value}`, `attr:{name}={value}`,
  `let:err`. Plain attributes (`name="x"`, `name={expr}`, bare-name
  boolean attrs) round-trip through their own rules.
- **Brace interpolation** — `{expr}` in text and attribute-value
  position. The expression body is captured as an opaque balanced
  atom stream; host-language highlighting comes from a TypeScript
  injection (`queries/injections.scm`).
- **Control-flow blocks**
  - `{#if cond}` / `{:else if cond}` / `{:elif cond}` / `{:else}` / `{/if}`
  - `{#for x of items key={x.id}}` / `{:else}` / `{/for}` (key is optional;
    pattern accepts `(item, idx)`-style tuples)
  - `{#match value}` / `{:case 'a'}` / `{:default}` / `{/match}`
- **HTML comments** `<!-- … -->`.

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
cd packages/sui/grammar
npm install                 # one-shot install of tree-sitter-cli
npx tree-sitter generate    # writes src/parser.c
npx tree-sitter test        # runs the corpus
```

## Validation

```bash
cd packages/sui/grammar
npx tree-sitter parse ../../views/agents/view.html.set --quiet \
  || echo "FAIL: views/agents/view.html.set"
```

Expected: prints nothing.

## Known v3 work

- **External scanner.** Multi-line balanced expressions and
  raw-text element bodies (`<style>` / `<script>` without `src`)
  rely on a regex/precedence approximation. An external C scanner
  would handle template literals containing `${\``}` (a backtick
  inside `${…}` inside a backtick — none in the corpus today) and
  let inline `<style>` bodies span unmatched `{` characters cleanly.
- **`queries/locals.scm`.** Would expose declaration scopes
  (component props, `let:` bindings) for jump-to-def in editors
  that consume the `@local.*` family. Currently absent.
- **HTML attribute namespaces.** `attr_name` excludes `:` so the
  `bind:` / `on:` / `class:` / `style:` / `attr:` / `let:`
  directives win the lexer race; the corpus has no SVG-style
  namespaced attributes (`xlink:href`, …) so this is safe today
  but worth revisiting if SVG lands as a first-class template
  surface.
