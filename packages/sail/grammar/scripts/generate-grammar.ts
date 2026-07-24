#!/usr/bin/env bun
/**
 * Generate `grammar.js` and `queries/highlights.scm` for the
 * tree-sitter-sail grammar package from the compiler's vocabulary dump.
 *
 * The whole files are generated from templates because the vocabulary
 * (void tags, directive prefixes, directive elements, block names and
 * clauses) is interleaved throughout both files. Section-based
 * substitution would be fragile; regenerating the whole file from a
 * template keeps one source of truth in the compiler manifests and a
 * single deterministic output here.
 *
 * Usage:
 *   bun run scripts/generate-grammar.ts --vocab vocabulary.json --out-dir .
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

const NO_ARG_CLAUSES: Record<string, true> = { '{:else}': true, '{:default}': true };

function parseArgs(argv: string[]) {
  let vocab: string | undefined;
  let outDir: string | undefined;
  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === '--vocab') vocab = argv[++i];
    else if (arg === '--out-dir') outDir = argv[++i];
  }
  return { vocab: vocab ?? '-', outDir: outDir ?? '.' };
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
  return `${open.replace('{#', '').replace('}', '').replace(/-/g, '_')}_block`;
}

function clauseRuleName(clause: string): string {
  return `${clause.replace('{:', '').replace('}', '').replace(/\s/g, '_').replace(/-/g, '_')}_clause`;
}

function attributeRuleName(prefix: string): string {
  return prefix === 'on' ? 'event_attr' : `${prefix}_attr`;
}

function blockVariableField(open: string): string {
  if (open === '{#if}') return 'cond';
  if (open === '{#for}') return 'iter';
  if (open === '{#match}') return 'value';
  if (open === '{#while-loading}') return 'signal';
  if (open === '{#while-error}') return 'signal';
  return 'cond';
}

function clauseVariableField(clause: string): string {
  return clause.startsWith('{:else') ? 'cond' : 'value';
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

  const blockRules = v.blocks
    .map((b) => {
      const rule = blockRuleName(b.open);
      const openPrefix = b.open.replace('}', '');
      const close = b.open.replace('{#', '{/');
      const clauses = b.clauses
        .map((c) =>
          c in NO_ARG_CLAUSES
            ? `  optional($.${clauseRuleName(c)}),`
            : `  repeat($.${clauseRuleName(c)}),`,
        )
        .join('\n');

      let body: string;
      if (b.open === '{#for}') {
        body = `${rule}: $ => seq(
  ${jsString(openPrefix)}, field('item', $.for_pattern), 'of',
  field('iter', $._iter_expr),
  optional(seq('key', '=', field('key', $.brace_expression))),
  '}',
  repeat($._template_node),
  optional($.else_clause),
  ${jsString(close)},
),`;
      } else if (b.open === '{#while-loading}') {
        body = `${rule}: $ => seq(
  ${jsString(openPrefix)}, 'of', field('signal', $.identifier), '}',
  repeat($._template_node),
  ${jsString(close)},
),`;
      } else if (b.open === '{#while-error}') {
        body = `${rule}: $ => seq(
  ${jsString(openPrefix)},
  field('error', $.identifier), 'of',
  field('signal', $.identifier),
  '}',
  repeat($._template_node),
  ${jsString(close)},
),`;
      } else if (b.open === '{#if}') {
        body = `${rule}: $ =\u003e seq(
  ${jsString(openPrefix)}, field('cond', $._expr), '}',
  repeat($._template_node),
${clauses}
  ${jsString(close)},
),`;
      } else if (b.open === '{#match}') {
        body = `${rule}: $ =\u003e seq(
  ${jsString(openPrefix)}, field('value', $._expr), '}',
${clauses}
  ${jsString(close)},
),`;
      } else {
        body = `${rule}: $ => seq(
  ${jsString(openPrefix)}, field('cond', $._expr), '}',
  repeat($._template_node),
${clauses}
  ${jsString(close)},
),`;
      }
      return body;
    })
    .join('\n\n');

  const blockNodeChoices = v.blocks
    .map((b) => `      $.${blockRuleName(b.open)},`)
    .join('\n');

  // Clause rules are defined once each, even if used by multiple blocks
  // (e.g. `{:else}` appears in both `{#if}` and `{#for}`).
  const uniqueClauses = [...new Map(v.blocks.flatMap((b) => b.clauses).map((c) => [c, c])).values()];
  const clauseRules = uniqueClauses
    .map((c) => {
      const rule = clauseRuleName(c);
      if (c in NO_ARG_CLAUSES) {
        return `${rule}: $ => seq(
  ${jsString(c)},
  repeat($._template_node),
),`;
      }
      return `${rule}: $ => seq(
  ${jsString(c.replace('}', ''))},
  field('${clauseVariableField(c)}', $._expr),
  '}',
  repeat($._template_node),
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
      if (prefix === 'motion') {
        return `    // \`motion:trigger={{ prop: 'preset' }}\` — animation config.
    ${rule}: $ => seq(
      'motion:',
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
//   crates/sail/compiler/src/l1_manifest.rs        (VOID_TAGS)
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
    // confused with binding prefixes.
    tag_name: $ => /[a-zA-Z][a-zA-Z0-9-]*/,

    // ── attributes ────────────────────────────────────────────────
    // \`style:\` and \`let:\` are deliberately absent: \`style:\` is a hard
    // compile error (runtime geometry is \`use:<action>\` now) and
    // \`let:\` has no compiler-side implementation and no real-world
    // usage. Both fall through to \`attr_name\`, which excludes \`:\`, so
    // \`style:foo\` / \`let:err\` produce an ERROR node rather than
    // parsing as legitimate directives.
    _attribute: $ => choice(
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

    // Attribute names exclude \`:\` so the \`bind:\` / \`on:\` / … literals
    // tokenise as their own keyword. No production attribute uses
    // SVG-style namespacing (\`xlink:href\`, etc.) today.
    attr_name: $ => /[a-zA-Z_][a-zA-Z0-9_-]*/,
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
    // \`parse::template::blocks::for_header::parse_for_binding\`, which
    // only accepts a bare identifier or a \`(item, index)\` pair; a
    // bare comma list without parens is NOT accepted.
    for_pattern: $ => choice(
      $.identifier,
      seq('(', $.identifier, ',', $.identifier, ')'),
    ),

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

  const blockMarkerLines = v.blocks.flatMap((b) => {
    const markers: string[] = [b.open.replace('}', '')];
    for (const c of b.clauses) {
      markers.push(c in NO_ARG_CLAUSES ? c : c.replace('}', ''));
    }
    markers.push(b.open.replace('{#', '{/'));
    return markers;
  });
  const maxMarkerLen = Math.max(...blockMarkerLines.map((m) => m.length));
  const blockMarkers = v.blocks
    .map((b) => {
      const lines: string[] = [];
      const pushMarker = (m: string) =>
        lines.push(`"${m}"${' '.repeat(maxMarkerLen - m.length + 3)}@keyword.directive`);
      pushMarker(b.open.replace('}', ''));
      for (const c of b.clauses) {
        pushMarker(c in NO_ARG_CLAUSES ? c : c.replace('}', ''));
      }
      pushMarker(b.open.replace('{#', '{/'));
      return lines.join('\n');
    })
    .join('\n');

  const identifierTargets: string[] = [
    '(interpolation (identifier) @variable)',
    '(brace_expression (identifier) @variable)',
  ];
  for (const b of v.blocks) {
    const rule = blockRuleName(b.open);
    const field = blockVariableField(b.open);
    identifierTargets.push(`(${rule} ${field}: (identifier) @variable)`);
  }
  for (const b of v.blocks) {
    for (const c of b.clauses) {
      if (c in NO_ARG_CLAUSES) continue;
      const rule = clauseRuleName(c);
      const field = clauseVariableField(c);
      identifierTargets.push(`(${rule} ${field}: (identifier) @variable)`);
    }
  }

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
; the runtime contract at a glance. \`<data>\`, \`<state>\`, \`<component>\`,
; \`<prop>\` are retired; element-form \`<while-loading>\`/\`<while-error>\`
; are retired in favor of the brace-block forms below.
((tag_name) @keyword.directive
  (#match? @keyword.directive "^(${directiveElements})$"))

; ── binding attribute prefixes ───────────────────────────────────────
; \`style:\` and \`let:\` are deliberately absent: the compiler hard-refuses
; \`style:\` and never implemented \`let:\` — neither is live syntax.
${directivePrefixLines}
"|"       @punctuation.delimiter

; ── template control flow markers ────────────────────────────────────
; \`{:elif}\` is deliberately absent — the compiler's tokenizer only
; recognises \`{:else if}\`.
${blockMarkers}

; Brace boundaries.
"{" @punctuation.special
"}" @punctuation.special

; ── expression body (inside \`{…}\`) ─────────────────────────────────
; Identifiers in expression position are variable reads; member
; access shows up as a single \`.foo\` token (the \`_member\` rule).
${identifierTargets.join('\n')}

; For-loop binding pattern names are locals introduced by the loop.
(for_pattern (identifier) @variable.parameter)

; \`{#while-error <errVar> of <signal>}\` — the error binding is a local
; introduced by the block, scoped to its body.
(while_error_block error: (identifier) @variable.parameter)

; ── comments ─────────────────────────────────────────────────────────
(html_comment) @comment
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
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
