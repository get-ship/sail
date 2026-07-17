// Tree-sitter grammar for `.sail` template files.
//
// `.sail` is HTML extended with template grammar:
//
//   • brace interpolations   `{expr}`
//   • control-flow blocks    `{#if}` / `{#for}` / `{#match}` /
//                             `{#while-loading}` / `{#while-error}`
//   • prefixed binding attrs `bind:` `on:` `class:` `attr:` `prop:`
//                             `use:` `motion:`
//   • directive elements     `<data>` `<scope>` (just HTML elements as
//                             far as the grammar is concerned)
//
// Directive elements are *just* HTML elements as far as the grammar
// is concerned — the runtime picks them out by tag name. Highlight
// queries flag them visually.
//
// Expression bodies are tokenised as opaque balanced atom sequences;
// host-language highlighting comes from `tree-sitter-typescript` via
// `queries/injections.scm`. Inline `<style>` content gets `tree-sitter-css`.
//
// The block/clause vocabulary below is kept in lockstep with the
// compiler's `block_manifest::BLOCKS` table
// (`crates/sail/compiler/src/block_manifest.rs`) by
// `bindings/rust/tests/block_vocabulary.rs` — a new block or clause
// added to the compiler without a matching grammar rule fails that
// test.
//
// Retired/refused surface this grammar deliberately does NOT parse as
// valid template syntax:
//   - `style:` attrs — the compiler hard-refuses them
//     (`parse/template/attrs.rs`); runtime geometry is `use:<action>`.
//   - `let:` attrs — unimplemented, unused; slot/async bindings are
//     declared on the block header itself (`{#while-error err of x}`).
//   - `<state>` as a top-level directive — retired
//     (`parse::reject`/`state_directive_retired`); use the
//     controller's `setup:` factory.
//   - `<component>`/`<prop>` directive elements — retired; replaced by
//     `<scope>` (`props: { … } variants: { … }`).
//   - element-form `<while-loading>`/`<while-error>` — these are brace
//     blocks now, not elements.
//   - `{:elif}` — the compiler's tokenizer only recognises
//     `{:else if}`.
//
// Conformance target: real `.sail` sources across the repo parse with
// no ERROR nodes. Validation lives in `test/corpus/`.

export default grammar({
  name: 'sail',

  extras: $ => [/\s+/],

  word: $ => $.identifier,

  rules: {
    source_file: $ => repeat($._template_node),

    // ── template nodes ────────────────────────────────────────────
    _template_node: $ => choice(
      $.html_comment,
      $.if_block,
      $.for_block,
      $.match_block,
      $.while_loading_block,
      $.while_error_block,
      $.element,
      $.interpolation,
      $.text,
    ),

    // `<!-- ... -->` — opaque single token.
    html_comment: $ => token(seq('<!--', /[\s\S]*?-->/)),

    // Plain text. Stops at `<` (next tag) or `{` (next brace marker).
    // Requires at least one non-whitespace character so pure-
    // whitespace runs between elements are consumed as extras
    // instead of producing noisy `(text)` nodes.
    text: $ => token(/[^<{]*[^<{\s][^<{]*/),

    // ── elements ──────────────────────────────────────────────────
    element: $ => choice(
      $._self_closing_element,
      $._void_element,
      $._regular_element,
    ),
    _self_closing_element: $ => seq(
      '<',
      field('tag', $.tag_name),
      repeat($._attribute),
      '/>',
    ),
    // HTML void elements (`<link>`, `<br>`, `<img>`, …) — open tag
    // only; no closing tag is ever emitted. The tag-name literals
    // tokenise via tree-sitter's keyword-extraction path so they
    // outrank the generic `tag_name` regex; the trailing close
    // accepts both `>` and `/>` so `<input bind:value={x} />`
    // and `<link href="x">` both parse here.
    _void_element: $ => seq(
      '<',
      field('tag', $._void_tag_name),
      repeat($._attribute),
      choice('>', '/>'),
    ),
    _void_tag_name: $ => alias(
      choice(
        'area', 'base', 'br', 'col', 'embed', 'hr', 'img', 'input',
        'link', 'meta', 'source', 'track', 'wbr',
      ),
      $.tag_name,
    ),
    _regular_element: $ => seq(
      '<',
      field('tag', $.tag_name),
      repeat($._attribute),
      '>',
      repeat($._template_node),
      '</',
      field('close_tag', $.tag_name),
      '>',
    ),
    // Tag names allow hyphens (custom elements: `ship-button`,
    // `error-boundary`, …) but not `:` so SVG-style namespaces aren't
    // confused with binding prefixes.
    tag_name: $ => /[a-zA-Z][a-zA-Z0-9-]*/,

    // ── attributes ────────────────────────────────────────────────
    // `style:` and `let:` are deliberately absent: `style:` is a hard
    // compile error (runtime geometry is `use:<action>` now) and
    // `let:` has no compiler-side implementation and no real-world
    // usage. Both fall through to `attr_name`, which excludes `:`, so
    // `style:foo` / `let:err` produce an ERROR node rather than
    // parsing as legitimate directives.
    _attribute: $ => choice(
      $.bind_attr,
      $.event_attr,
      $.class_attr,
      $.attr_attr,
      $.prop_attr,
      $.use_attr,
      $.motion_attr,
      $.dynamic_attr,
      $.static_attr,
      $.boolean_attr,
    ),
    bind_attr: $ => seq(
      'bind:',
      field('name', $.attr_local),
      '=',
      field('value', $.brace_expression),
    ),
    event_attr: $ => seq(
      'on:',
      field('event', $.attr_local),
      repeat(seq('|', field('modifier', $.attr_local))),
      '=',
      field('handler', $.brace_expression),
    ),
    class_attr: $ => seq(
      'class:',
      field('name', $.attr_local),
      '=',
      field('value', $.brace_expression),
    ),
    // `attr:name={value}` — bare name (the common form: `attr:hidden={…}`,
    // `attr:data-col={…}`), or `attr:{nameExpr}={value}` — dynamic
    // attribute name in braces. Both compile via
    // `parse::template::attrs::compile_dynamic_attr`.
    attr_attr: $ => seq(
      'attr:',
      field('name', choice($.attr_local, $.brace_expression)),
      '=',
      field('value', $.brace_expression),
    ),
    // `prop:name={value}` — write a declared `<scope>` prop by identity.
    prop_attr: $ => seq(
      'prop:',
      field('name', $.attr_local),
      '=',
      field('value', $.brace_expression),
    ),
    // `use:action` / `use:action={param}` — attach a reusable action.
    // The param is optional (many actions are `void`-parameterized).
    use_attr: $ => seq(
      'use:',
      field('action', $.attr_local),
      optional(seq('=', field('param', $.brace_expression))),
    ),
    // `motion:trigger={{ prop: 'preset' }}` — animation config. The
    // value is a static object literal at the compiler level; the
    // grammar just captures it as a brace expression.
    motion_attr: $ => seq(
      'motion:',
      field('trigger', $.attr_local),
      '=',
      field('config', $.brace_expression),
    ),

    dynamic_attr: $ => seq(
      field('name', $.attr_name),
      '=',
      field('value', $.brace_expression),
    ),
    static_attr: $ => seq(
      field('name', $.attr_name),
      '=',
      field('value', $.string_literal),
    ),
    // Lower precedence so `name="x"` and `name={x}` win over a bare
    // boolean read of `name`.
    boolean_attr: $ => prec(-1, $.attr_name),

    // Attribute names exclude `:` so the `bind:` / `on:` / … literals
    // tokenise as their own keyword. No production attribute uses
    // SVG-style namespacing (`xlink:href`, etc.) today.
    attr_name: $ => /[a-zA-Z_][a-zA-Z0-9_-]*/,
    attr_local: $ => /[a-zA-Z_][a-zA-Z0-9_-]*/,

    // Quoted literal — opaque content. No interpolation inside quoted
    // attribute values: authors use `name={…}` for that. `sx="…"`
    // strings (including `(--token)` reads) round-trip through this
    // rule unchanged; the sx atom vocabulary is a compile-time
    // concern, not a grammar concern.
    string_literal: $ => choice(
      token(seq('"', /[^"]*/, '"')),
      token(seq("'", /[^']*/, "'")),
    ),

    // ── interpolation / control flow ──────────────────────────────
    interpolation: $ => seq('{', optional($._expr), '}'),
    // Same shape as `interpolation` but used in attribute-value
    // position so editors can target them distinctly.
    brace_expression: $ => seq('{', optional($._expr), '}'),

    if_block: $ => seq(
      '{#if', field('cond', $._expr), '}',
      repeat($._template_node),
      repeat($.else_if_clause),
      optional($.else_clause),
      '{/if}',
    ),
    // Only `{:else if cond}` is recognised — the compiler's tokenizer
    // (`parse/template/tokenize.rs`) has no `{:elif}` alias.
    else_if_clause: $ => seq(
      '{:else if',
      field('cond', $._expr),
      '}',
      repeat($._template_node),
    ),
    else_clause: $ => seq(
      '{:else}',
      repeat($._template_node),
    ),

    for_block: $ => seq(
      '{#for', field('item', $.for_pattern), 'of',
      field('iter', $._iter_expr),
      optional(seq('key', '=', field('key', $.brace_expression))),
      '}',
      repeat($._template_node),
      optional($.else_clause),
      '{/for}',
    ),
    // `x of items` (bare binding) or `(x, i) of items` (parenthesized
    // tuple with index) — matches
    // `parse::template::blocks::for_header::parse_for_binding`, which
    // only accepts a bare identifier or a `(item, index)` pair; a
    // bare comma list without parens is NOT accepted.
    for_pattern: $ => choice(
      $.identifier,
      seq('(', $.identifier, ',', $.identifier, ')'),
    ),

    match_block: $ => seq(
      '{#match', field('value', $._expr), '}',
      repeat($.case_clause),
      optional($.default_clause),
      '{/match}',
    ),
    case_clause: $ => seq(
      '{:case', field('value', $._expr), '}',
      repeat($._template_node),
    ),
    default_clause: $ => seq(
      '{:default}',
      repeat($._template_node),
    ),

    // `{#while-loading of <signal>}` … `{/while-loading}` — renders
    // while an async query's promise is pending. Brace-block form
    // only; there is no `<while-loading>` element.
    while_loading_block: $ => seq(
      '{#while-loading', 'of', field('signal', $.identifier), '}',
      repeat($._template_node),
      '{/while-loading}',
    ),
    // `{#while-error <errVar> of <signal>}` … `{/while-error}` —
    // renders while an async query's promise is rejected; binds the
    // rejection reason to `<errVar>` for the block body.
    while_error_block: $ => seq(
      '{#while-error',
      field('error', $.identifier), 'of',
      field('signal', $.identifier),
      '}',
      repeat($._template_node),
      '{/while-error}',
    ),

    // ── opaque expression body ────────────────────────────────────
    // Inside `{…}` we capture everything as a balanced atom stream so
    // the host-language injection (TypeScript) can highlight it. The
    // top-level rule stops at `}` (which is not a valid `_expr_part`).
    _expr: $ => prec.right(repeat1($._expr_part)),
    _expr_part: $ => choice(
      $.identifier,
      $._member,
      $._number,
      $._punct,
      $._paren,
      $._bracket,
      $._brace,
      $._dq_string,
      $._sq_string,
      $._template_string,
    ),
    // For-loop iterator body: like `_expr` but excludes a top-level
    // `=` so the optional `key=…` continuation parses cleanly.
    _iter_expr: $ => prec.right(repeat1($._iter_part)),
    _iter_part: $ => choice(
      $.identifier,
      $._member,
      $._number,
      $._iter_punct,
      $._paren,
      $._bracket,
      $._brace,
      $._dq_string,
      $._sq_string,
      $._template_string,
    ),

    identifier: $ => /[a-zA-Z_$][a-zA-Z0-9_$]*/,
    // `.foo` member access — captured as a single token so the
    // `.foo` segment can't be mis-tokenised as `.` + identifier
    // (which would let keyword extraction trigger on names like
    // `.key`, `.of`, …).
    _member: $ => token(prec(1, /\.[a-zA-Z_$][a-zA-Z0-9_$]*/)),
    _number: $ => token(/[0-9][0-9_.]*(?:[eE][-+]?[0-9_]+)?/),
    // Punctuation runs — any non-identifier, non-bracket, non-quote
    // character (so JS operators `===`, `||`, `?.`, `??`, … land in
    // a single atom).
    _punct: $ => token(/[^\sa-zA-Z_$0-9{}()\[\]"'`]+/),
    // Iterator-body punct excludes `=` so `key=` peels off cleanly.
    // `=>` is admitted as a single multi-char atom for arrow-fn
    // headers nested inside `(…)`.
    _iter_punct: $ => token(/(?:=>|[^=\sa-zA-Z_$0-9{}()\[\]"'`]+)/),

    _paren: $ => seq('(', repeat($._balanced), ')'),
    _bracket: $ => seq('[', repeat($._balanced), ']'),
    _brace: $ => seq('{', repeat($._balanced), '}'),
    _balanced: $ => choice(
      $._balanced_atom,
      $._paren, $._bracket, $._brace,
      $._dq_string, $._sq_string, $._template_string,
    ),
    _balanced_atom: $ => token(prec(1, /[^\s{}()\[\]"'`]+/)),
    _dq_string: $ => token(/"(?:[^"\\]|\\.)*"/),
    _sq_string: $ => token(/'(?:[^'\\]|\\.)*'/),
    _template_string: $ => token(/`(?:[^`\\]|\\.)*`/),
  },
});
