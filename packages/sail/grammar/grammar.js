// Tree-sitter grammar for `.html.set` / `.set` template files.
//
// `.html.set` is HTML extended with template grammar:
//
//   • brace interpolations  `{expr}`
//   • control-flow blocks   `{#if}` / `{#for}` / `{#match}`
//   • prefixed binding attrs `bind:` `on:` `class:` `style:` `attr:`
//   • directive elements    `<data>` `<state>` `<component>` `<prop>`
//                           `<while-loading>` `<while-error>`
//
// Directive elements are *just* HTML elements as far as the grammar
// is concerned — the runtime picks them out by tag name. Highlight
// queries flag them visually.
//
// Expression bodies are tokenised as opaque balanced atom sequences;
// host-language highlighting comes from `tree-sitter-typescript` via
// `queries/injections.scm`. Inline `<style>` content gets `tree-sitter-css`.
//
// Conformance target: `views/agents/view.html.set` parses with no
// ERROR nodes. Validation lives in `test/corpus/`.

export default grammar({
  name: 'set',

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
    // `while-loading`, …) but not `:` so SVG-style namespaces aren't
    // confused with binding prefixes.
    tag_name: $ => /[a-zA-Z][a-zA-Z0-9-]*/,

    // ── attributes ────────────────────────────────────────────────
    _attribute: $ => choice(
      $.bind_attr,
      $.event_attr,
      $.class_attr,
      $.style_attr,
      $.attr_attr,
      $.let_attr,
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
    style_attr: $ => seq(
      'style:',
      field('name', $.style_prop_name),
      '=',
      field('value', $.brace_expression),
    ),
    // `attr:{name}={value}` — dynamic attribute name in braces.
    attr_attr: $ => seq(
      'attr:',
      field('name', $.brace_expression),
      '=',
      field('value', $.brace_expression),
    ),
    // `let:err` — slot/scope let-binding. Boolean-style; no value.
    let_attr: $ => seq('let:', field('name', $.attr_local)),

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
    // `style:--tone` — CSS custom property names start with `--`. We
    // also allow plain CSS property names (`color`, `background-image`)
    // so `style:color={…}` round-trips.
    style_prop_name: $ => /(?:--|-)?[a-zA-Z_][a-zA-Z0-9_-]*/,

    // Quoted literal — opaque content. No interpolation inside quoted
    // attribute values: authors use `name={…}` for that.
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
      repeat($.elif_clause),
      optional($.else_clause),
      '{/if}',
    ),
    elif_clause: $ => seq(
      // Both `{:else if cond}` and `{:elif cond}` are accepted; the
      // runtime tokeniser only emits the former but authors frequently
      // type the latter.
      choice('{:else if', '{:elif'),
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
    for_pattern: $ => seq(
      $.identifier,
      repeat(seq(',', $.identifier)),
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
