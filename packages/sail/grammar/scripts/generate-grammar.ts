#!/usr/bin/env bun
/**
 * Generate `grammar.js`, `queries/highlights.scm`, and (optionally) the
 * CM6 overlay vocab table from the compiler's vocabulary dump.
 *
 * The whole files are generated from templates because the vocabulary
 * (void tags, directive prefixes, directive elements, block names and
 * clauses) is interleaved throughout both files. Section-based
 * substitution would be fragile; regenerating the whole file from a
 * template keeps one source of truth in the compiler manifests and a
 * single deterministic output here.
 *
 * Usage:
 *   bun run scripts/generate-grammar.ts --vocab vocabulary.json --out-dir . \
 *     --overlay-vocab ../../components/sail/code-editor/src/sail-code-editor/sail-highlight-vocab.ts
 *
 * With stdin:
 *   cargo run -q -p sail-compiler -- vocabulary | \
 *     bun run scripts/generate-grammar.ts --vocab - --out-dir .
 */

import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';

interface Block {
  open: string;
  clauses: string[];
  summary: string;
}

interface Vocabulary {
  version: number;
  void_tags: string[];
  directive_prefixes: string[];
  directive_elements: string[];
  blocks: Block[];
}

/** Tag from element-form open like `<if {…}>` → `if`. */
function tagFromOpen(open: string): string {
  const m = open.match(/^<([a-z][a-z0-9-]*)/i);
  if (!m) throw new Error(`block open is not element form: ${open}`);
  return m[1]!;
}

/** Element-form clause open tag (`<else>`) vs arm form (`{else => …}`). */
function isElementClause(clause: string): boolean {
  return clause.trimStart().startsWith('<');
}

/** Tag from element-form clause like `<else>` → `else`. Arms have no tag. */
function tagFromClause(clause: string): string | null {
  const m = clause.match(/^<\/?([a-z][a-z0-9-]*)/i);
  return m ? m[1]! : null;
}

function parseArgs(argv: string[]) {
  let vocab: string | undefined;
  let outDir: string | undefined;
  let overlayVocab: string | undefined;
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--vocab') vocab = argv[++i];
    else if (arg === '--out-dir') outDir = argv[++i];
    else if (arg === '--overlay-vocab') overlayVocab = argv[++i];
  }
  return { vocab: vocab ?? '-', outDir: outDir ?? '.', overlayVocab };
}

async function readVocab(path: string): Promise<Vocabulary> {
  const input =
    path === '-' || path === undefined
      ? await readStdin()
      : await readFile(path, 'utf-8');
  const parsed = JSON.parse(input) as Vocabulary;
  if (parsed.version !== 1) {
    throw new Error(`unsupported vocabulary version: ${parsed.version}`);
  }
  return parsed;
}

async function readStdin(): Promise<string> {
  const chunks: Buffer[] = [];
  for await (const chunk of process.stdin) {
    chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
  }
  return Buffer.concat(chunks).toString('utf-8');
}

function blockRuleName(open: string): string {
  return `${tagFromOpen(open).replace(/-/g, '_')}_block`;
}

function clauseRuleName(clause: string): string {
  const tag = tagFromClause(clause);
  if (!tag) throw new Error(`arm clauses have no element rule: ${clause}`);
  return `${tag.replace(/-/g, '_')}_clause`;
}

function attributeRuleName(prefix: string): string {
  return prefix === 'on' ? 'event_attr' : `${prefix}_attr`;
}

function jsString(s: string): string {
  const out = JSON.stringify(s);
  return `'${out.slice(1, -1).replace(/\\'/g, "'").replace(/'/g, "\\'").replace(/\\"/g, '"')}'`;
}

function indent(text: string, spaces: number): string {
  const prefix = ' '.repeat(spaces);
  return text
    .split('\n')
    .map((line) => (line ? prefix + line : line))
    .join('\n');
}

function generateGrammar(v: Vocabulary): string {
  const voidTags = (() => {
    const tags = v.void_tags.map(jsString);
    const rows: string[] = [];
    for (let i = 0; i < tags.length; i += 6) {
      rows.push(tags.slice(i, i + 6).join(', '));
    }
    return rows.map((row) => `        ${row}`).join(',\n');
  })();
  const directiveElements = v.directive_elements.join('|');

  // Open tokens include the delimiter (`<if>`, `<if `, `<if{`) so `<form>`
  // cannot tokenize as `<for` + boolean `m`. Tree-sitter has no lookahead
  // and higher prec beats a longer tag_name match.
  const openChoice = (tag: string) => `choice(
  alias(token(/<${tag}>/), $.control_open),
  seq(alias(token(/<${tag}\\{/), $.control_open), optional($._expr), '}', repeat($._attribute), '>'),
  seq(alias(token(/<${tag}[\\s]+/), $.control_open), optional($.brace_expression), repeat($._attribute), '>'),
)`;
  const closeTok = (tag: string) => `alias(token(/<\\/${tag}>/), $.control_close)`;

  const blockRules = v.blocks
    .map((b) => {
      const rule = blockRuleName(b.open);
      const tag = tagFromOpen(b.open);
      return `${rule}: $ => seq(
  ${openChoice(tag)},
  repeat($._template_node),
  ${closeTok(tag)},
),`;
    })
    .join('\n\n');


  // Block opens only — arm clauses (`{else => …}`) parse as interpolations.
  const allControlRules = v.blocks.map((b) => blockRuleName(b.open));
  const blockNodeChoices = allControlRules
    .map((name) => `      $.${name},`)
    .join('\n');

  // Residual element-form clauses only (none under BLOCKS-1 arm syntax).
  const uniqueElementClauses = [
    ...new Map(
      v.blocks
        .flatMap((b) => b.clauses)
        .filter(isElementClause)
        .map((c) => [c, c]),
    ).values(),
  ];
  const clauseRules = uniqueElementClauses
    .map((c) => {
      const rule = clauseRuleName(c);
      const tag = tagFromClause(c)!;
      return `${rule}: $ => seq(
  ${openChoice(tag)},
  repeat($._template_node),
  ${closeTok(tag)},
),`;
    })
    .join('\n\n');

  const allBlockRules = clauseRules
    ? `${blockRules}\n\n${clauseRules}`
    : blockRules;

  const attributeRules = v.directive_prefixes
    .map((prefix) => {
      const rule = attributeRuleName(prefix);
      if (prefix === 'on') {
        return `    // \`on:\` event handler with optional chained \`|modifier\` modifiers.
    ${rule}: $ => seq(
      'on:',
      field('event', $.attr_local),
      repeat(seq('|', field('modifier', $.attr_local))),
      '=',
      field('handler', $.brace_expression),
    ),`;
      }
      if (prefix === 'use') {
        return `    // \`use:action\` / \`use:action={param}\` — attach a reusable action.
    ${rule}: $ => seq(
      'use:',
      field('action', $.attr_local),
      optional(seq('=', field('param', $.brace_expression))),
    ),`;
      }
      if (prefix === 'fx') {
        return `    // \`fx:trigger={{ prop: 'preset' }}\` — animation config.
    ${rule}: $ => seq(
      'fx:',
      field('trigger', $.attr_local),
      '=',
      field('config', $.brace_expression),
    ),`;
      }
      return `    ${rule}: $ => seq(
      '${prefix}:',
      field('name', $.attr_local),
      '=',
      field('value', $.brace_expression),
    ),`;
    })
    .join('\n');

  const attributeChoice = v.directive_prefixes
    .map((prefix) => `      $.${attributeRuleName(prefix)},`)
    .join('\n');

  return `// Tree-sitter grammar for \`.sail\` template files.
//
// GENERATED by packages/sail/grammar/scripts/generate-grammar.ts.
// Do not edit by hand. Regenerate with:
//   cargo run -q -p sail-compiler -- vocabulary | \\
//     bun run scripts/generate-grammar.ts --vocab - --out-dir .
//
// Source of truth for the closed vocabulary (void tags, directive prefixes,
// directive elements, block names and clauses) is the compiler's manifests:
//   crates/sail/compiler/src/l1_manifest.rs        (author_void_tags)
//   crates/sail/compiler/src/directive_manifest.rs (DIRECTIVE_PREFIXES, DIRECTIVE_ELEMENTS)
//   crates/sail/compiler/src/block_manifest.rs       (BLOCKS)
//
// Expression parsing rules and the HTML-shaped outer layer are authored in
// the generator template; the vocabulary sections are substituted from the
// dump so the grammar cannot drift from the compiler.

export default grammar({
  name: 'sail',

  extras: $ => [/\\s+/],

  word: $ => $.identifier,

  rules: {
    source_file: $ => repeat($._template_node),

    // ── template nodes ────────────────────────────────────────────
    _template_node: $ => choice(
      $.html_comment,
      $.controller,
      $.scope,
${blockNodeChoices}
      $.element,
      $.interpolation,
      $.text,
    ),

    // \`<!-- ... -->\` — opaque single token.
    html_comment: $ => token(seq('<!--', /[\\s\\S]*?-->/)),

    // Plain text. Stops at \`<\` (next tag) or \`{\` (next brace marker).
    // Requires at least one non-whitespace character so pure-
    // whitespace runs between elements are consumed as extras
    // instead of producing noisy \`(text)\` nodes.
    text: $ => token(/[^<{]*[^<{\\s][^<{]*/),

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
    // HTML void elements (\`<link>\`, \`<br>\`, \`<img>\`, …) — open tag
    // only; no closing tag is ever emitted. The tag-name literals
    // tokenise via tree-sitter's keyword-extraction path so they
    // outrank the generic \`tag_name\` regex; the trailing close
    // accepts both \`>\` and \`/>\` so \`<input bind:value={x} />\`
    // and \`<link href="x">\` both parse here.
    _void_element: $ => seq(
      '<',
      field('tag', $._void_tag_name),
      repeat($._attribute),
      choice('>', '/>'),
    ),
    _void_tag_name: $ => alias(
      choice(
${voidTags},
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
    // Tag names allow hyphens (custom elements: \`ship-button\`,
    // \`error-boundary\`, …) but not \`:\` so SVG-style namespaces aren't
    // confused with binding prefixes. \`'style'\` is a keyword (style_attr /
    // style_decl) so it must also be a legal tag_name — otherwise \`<style>\`
    // CSS blocks fail to parse.
    tag_name: $ => choice('style', /[a-zA-Z][a-zA-Z0-9-]*/),

    // ── attributes ────────────────────────────────────────────────
    // \`style:\` and \`let:\` are deliberately absent: \`style:\` is a hard
    // compile error (runtime geometry is \`use:<action>\` now) and
    // \`let:\` has no compiler-side implementation and no real-world
    // usage. Both fall through to \`attr_name\`, which excludes \`:\`, so
    // \`style:foo\` / \`let:err\` produce an ERROR node rather than
    // parsing as legitimate directives.
    _attribute: $ => choice(
      $.style_attr,
${attributeChoice}
      $.dynamic_attr,
      $.static_attr,
      $.boolean_attr,
    ),
${attributeRules}

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
    // Lower precedence so \`name="x"\` and \`name={x}\` win over a bare
    // boolean read of \`name\`.
    boolean_attr: $ => prec(-1, $.attr_name),

    // Live Style attach: style(name), style({ … }), style(name, { … }),
    // optional \`.when({…})\` chain. Must beat boolean_attr on \`style\`.
    style_attr: $ => prec(1, seq(
      'style',
      field('args', $.style_call),
      repeat(seq('.', field('chain', $.identifier), $.style_call)),
    )),
    style_call: $ => seq(
      '(',
      optional(seq($._style_arg, repeat(seq(',', $._style_arg)))),
      ')',
    ),
    _style_arg: $ => choice($.identifier, $.brace_expression),

    // ── <controller> raw TypeScript ────────────────────────────────
    // Same contract as the compiler RAW_TEXT_TAGS: the body is not a
    // template. A regular element would split on \`{\` / \`<\` (generics).
    // Tree-sitter regex has no lookahead, so match-until-</controller>
    // is the usual "eat anything that isn't the closer" expansion.
    controller: $ => seq(
      choice(
        alias(token(/<controller>/), $.control_open),
        seq(alias(token(/<controller[\\s]+/), $.control_open), repeat($._attribute), '>'),
      ),
      optional(field('body', $.controller_body)),
      alias(token(/<\\/controller>/), $.control_close),
    ),
    controller_body: $ => token(new RegExp(
      '(?:[^<]|<[^/]|</[^c]|</c[^o]|</co[^n]|</con[^t]|</cont[^r]|</contr[^o]|</contro[^l]|</control[^l]|</controll[^e]|</controlle[^r]|</controller[^>])+'
    )),

    // ── <scope> Style + static surface ─────────────────────────────
    // Compiler members: style name = …, props: {…}, type/import, residual
    // compound. Unknown lines fall through so a props/import cannot ERROR
    // the rest of the block (and kill highlighting).
    scope: $ => seq(
      choice(
        alias(token(/<scope>/), $.control_open),
        seq(alias(token(/<scope[\\s]+/), $.control_open), repeat($._attribute), '>'),
      ),
      repeat($._scope_item),
      alias(token(/<\\/scope>/), $.control_close),
    ),
    _scope_item: $ => choice(
      $.style_decl,
      $.props_block,
      $.line_comment,
      $.html_comment,
      // One-token leftovers (\`type\` / \`import\` / \`=\` / \`{…}\`) so a
      // type alias cannot ERROR the rest of the block.
      $.identifier,
      $.brace_expression,
      $.string_literal,
      $.scope_punct,
    ),
    style_decl: $ => prec(1, seq(
      'style',
      field('name', $.identifier),
      '=',
      field('value', $._style_value),
      optional(';'),
    )),
    // \`{…}\`, compose \`host\`, or \`host, {…}\`.
    _style_value: $ => choice(
      $.brace_expression,
      seq($.identifier, optional(seq(',', $.brace_expression))),
    ),
    props_block: $ => prec(1, seq(
      $.props_kw,
      ':',
      field('body', $.brace_expression),
      optional(';'),
    )),
    // Named token — not the string literal 'props' — so we do not steal
    // \`props\` from attribute position (\`<partial name props: Type>\`).
    props_kw: $ => token(prec(1, /props/)),
    line_comment: $ => token(seq('//', /[^\\n]*/)),
    scope_punct: $ => token(/[^\\s<\"'\`a-zA-Z_{]+/),

    // Attribute names exclude \`:\` so the \`bind:\` / \`on:\` / … literals
    // tokenise as their own keyword. \`'style'\` is a keyword so it must
    // remain a legal attr_name (\`style="…"\` residual HTML).
    attr_name: $ => choice('style', /[a-zA-Z_][a-zA-Z0-9_-]*/),
    attr_local: $ => /[a-zA-Z_][a-zA-Z0-9_-]*/,

    // Quoted literal — opaque content. No interpolation inside quoted
    // attribute values: authors use \`name={…}\` for that. \`sx="…"\`
    // strings (including \`(--token)\` reads) round-trip through this
    // rule unchanged; the sx atom vocabulary is a compile-time
    // concern, not a grammar concern.
    string_literal: $ => choice(
      token(seq('"', /[^"]*/, '"')),
      token(seq("'", /[^']*/, "'")),
    ),

    // ── interpolation / control flow ──────────────────────────────
    interpolation: $ => seq('{', optional($._expr), '}'),
    // Same shape as \`interpolation\` but used in attribute-value
    // position so editors can target them distinctly.
    brace_expression: $ => seq('{', optional($._expr), '}'),

${indent(allBlockRules, 4)}

    // \`x of items\` (bare binding) or \`(x, i) of items\` (parenthesized
    // tuple with index) — matches
    // ── opaque expression body ────────────────────────────────────
    // Inside \`{…}\` we capture everything as a balanced atom stream so
    // the host-language injection (TypeScript) can highlight it. The
    // top-level rule stops at \`}\` (which is not a valid \`_expr_part\`).
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
    // For-loop iterator body: like \`_expr\` but excludes a top-level
    // \`=\` so the optional \`key=…\` continuation parses cleanly.
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
    // \`.foo\` member access — captured as a single token so the
    // \`.foo\` segment can't be mis-tokenised as \`.\` + identifier
    // (which would let keyword extraction trigger on names like
    // \`.key\`, \`.of\`, …).
    _member: $ => token(prec(1, /\\.[a-zA-Z_$][a-zA-Z0-9_$]*/)),
    _number: $ => token(/[0-9][0-9_.]*(?:[eE][-+]?[0-9_]+)?/),
    // Punctuation runs — any non-identifier, non-bracket, non-quote
    // character (so JS operators \`===\`, \`||\`, \`?.\`, \`??\`, … land in
    // a single atom).
    _punct: $ => token(/[^\\sa-zA-Z_$0-9{}()\\[\\]"'\`]+/),
    // Iterator-body punct excludes \`=\` so \`key=\` peels off cleanly.
    // \`=>\` is admitted as a single multi-char atom for arrow-fn
    // headers nested inside \`(…)\`.
    _iter_punct: $ => token(/(?:=>|[^=\\sa-zA-Z_$0-9{}()\\[\\]"'\`]+)/),

    _paren: $ => seq('(', repeat($._balanced), ')'),
    _bracket: $ => seq('[', repeat($._balanced), ']'),
    _brace: $ => seq('{', repeat($._balanced), '}'),
    _balanced: $ => choice(
      $._balanced_atom,
      $._paren, $._bracket, $._brace,
      $._dq_string, $._sq_string, $._template_string,
    ),
    _balanced_atom: $ => token(prec(1, /[^\\s{}()\\[\\]"'\`]+/)),
    _dq_string: $ => token(/"(?:[^"\\\\]|\\\\.)*"/),
    _sq_string: $ => token(/'(?:[^'\\\\]|\\\\.)*'/),
    _template_string: $ => token(/\`(?:[^\`\\\\]|\\\\.)*\`/),
  },
});
`;
}

function generateHighlights(v: Vocabulary): string {
  const directiveElements = v.directive_elements.join('|');
  const directivePrefixes = v.directive_prefixes.map((p) => `${p}:`);
  const maxPrefixLen = Math.max(...directivePrefixes.map((p) => p.length));
  const directivePrefixLines = directivePrefixes
    .map((p) => `"${p}"${' '.repeat(maxPrefixLen - p.length + 3)}@keyword.directive`)
    .join('\n');

  const controlTags = [...new Set(v.blocks.map((b) => tagFromOpen(b.open)))];
  const controlTagAlt = controlTags.join('|');
  // Named `control_tag` (not `"<if"` literals) so `<form>` cannot tokenize
  // as `'<for'` + `m`. Helix also drops all highlights on unknown anons.
  const blockMarkers = `; control-flow + dedicated tags (if/for/match/when/partial/controller/scope)
(control_open) @keyword.directive
(control_close) @keyword.directive
((tag_name) @keyword.directive
  (#match? @keyword.directive "^(${controlTagAlt}|controller|scope)$"))`;

  const identifierTargets: string[] = [
    '(interpolation (identifier) @variable)',
    '(brace_expression (identifier) @variable)',
  ];

  return `; Tree-sitter highlight scopes for \`.sail\` template files.
;
; GENERATED by packages/sail/grammar/scripts/generate-grammar.ts.
; Do not edit by hand. Regenerate with:
;   cargo run -q -p sail-compiler -- vocabulary | \\
;     bun run scripts/generate-grammar.ts --vocab - --out-dir .
;
; Source of truth for the closed vocabulary (directive elements, directive
; prefixes, block names and clauses) is the compiler's manifests:
;   crates/sail/compiler/src/directive_manifest.rs (DIRECTIVE_PREFIXES, DIRECTIVE_ELEMENTS)
;   crates/sail/compiler/src/block_manifest.rs       (BLOCKS)
;
; Scope names follow the conventional \`tree-sitter-highlight\` palette
; (https://tree-sitter.github.io/tree-sitter/syntax-highlighting).
; Editors map them onto their own theme tokens.

; ── HTML element / attribute structure ───────────────────────────────
(tag_name) @tag
(attr_name) @attribute
(attr_local) @attribute

(string_literal) @string

; Element bracket punctuation.
"<"  @punctuation.bracket
">"  @punctuation.bracket
"</" @punctuation.bracket
"/>" @punctuation.bracket

; ── directive elements (recognised by tag name) ──────────────────────
; Highlight \`<scope>\` and \`<slot>\` distinctively so authors can see
; the runtime contract at a glance. Retired own-API tags are not listed
; (DEEMBALM: generic unknown-vocabulary after scrub).
((tag_name) @keyword.directive
  (#match? @keyword.directive "^(${directiveElements})$"))

; ── binding attribute prefixes ───────────────────────────────────────
; \`style:\` and \`let:\` are deliberately absent: the compiler hard-refuses
; \`style:\` and never implemented \`let:\` — neither is live syntax.
${directivePrefixLines}
"|"       @punctuation.delimiter

; ── template control flow markers ────────────────────────────────────
; Four blocks only: if / for / match / when. Arms are brace interpolations.
${blockMarkers}

; Do not capture anonymous \`{\` / \`}\` — Helix's query compiler treats
; those quotes as an empty node type and drops all highlights.

; ── expression body (inside \`{…}\`) ─────────────────────────────────
; Identifiers in expression position are variable reads; member
; access shows up as a single \`.foo\` token (the \`_member\` rule).
; Arm heads (\`{loading => …}\`, \`{ok as x => …}\`) land here too.
${identifierTargets.join('\n')}

; ── Style (scope decls + element attach) ────────────────────────────
(style_decl "style" @keyword)
(style_attr "style" @keyword)
(style_decl (identifier) @function)
(style_attr (identifier) @function)
(props_block (props_kw) @keyword)
(line_comment) @comment

; ── comments ─────────────────────────────────────────────────────────
(html_comment) @comment
`;
}

function uniqueControlTags(v: Vocabulary): string[] {
  const tags = [...new Set(v.blocks.map((b) => tagFromOpen(b.open)))];
  for (const c of v.blocks.flatMap((b) => b.clauses).filter(isElementClause)) {
    const t = tagFromClause(c);
    if (t) tags.push(t);
  }
  return [...new Set(tags)];
}

/**
 * Typed table the studio/site CM6 overlay imports. Same prefixes / tags
 * as `queries/highlights.scm` so the overlay cannot teach retired syntax.
 */
function generateHighlightVocab(v: Vocabulary): string {
  const asConst = (xs: string[]) => `[${xs.map(jsString).join(', ')}] as const`;
  return `/**
 * GENERATED by packages/sail/grammar/scripts/generate-grammar.ts.
 * Do not edit by hand. Regenerate with:
 *   bun run generate
 * in packages/sail/grammar.
 *
 * Closed highlight vocabulary for <sail-code-editor>. Same compiler
 * manifests as grammar.js and queries/highlights.scm.
 */

export const directivePrefixes = ${asConst(v.directive_prefixes)}

export const directiveElements = ${asConst(v.directive_elements)}

export const controlTags = ${asConst(uniqueControlTags(v))}
`;
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  const vocab = await readVocab(args.vocab);

  const outDir = resolve(args.outDir);
  await mkdir(outDir, { recursive: true });

  await writeFile(resolve(outDir, 'grammar.js'), generateGrammar(vocab), 'utf-8');
  await mkdir(resolve(outDir, 'queries'), { recursive: true });
  await writeFile(resolve(outDir, 'queries/highlights.scm'), generateHighlights(vocab), 'utf-8');

  console.log(`wrote ${resolve(outDir, 'grammar.js')}`);
  console.log(`wrote ${resolve(outDir, 'queries/highlights.scm')}`);

  if (args.overlayVocab) {
    const overlayPath = resolve(args.overlayVocab);
    await mkdir(resolve(overlayPath, '..'), { recursive: true });
    await writeFile(overlayPath, generateHighlightVocab(vocab), 'utf-8');
    console.log(`wrote ${overlayPath}`);
  }
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
