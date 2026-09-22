; Embedded language injections for `.sail`.
;
; Routes interpolation/binding bodies to TypeScript and inline `<style>`
; blocks to CSS so editors get host-language highlighting inside those
; ranges. `<controller>` is raw TypeScript. `<scope>` is Style decls +
; `props:` (not a TS document — style_decl / props_block are structured).

; ── TypeScript injections inside expression braces ───────────────────
; No `#offset!` — Helix 25 rejects that predicate and then loads *no*
; highlights for the language. include-children keeps `{…}` text in the
; injected document (braces are unnamed tokens).
((interpolation) @injection.content
  (#set! injection.language "typescript")
  (#set! injection.include-children))

((brace_expression) @injection.content
  (#set! injection.language "typescript")
  (#set! injection.include-children))

; ── TypeScript for <controller> raw body ─────────────────────────────
; Leaf token — no include-children (same as html `raw_text` → javascript).
; Helix loads `grammars/typescript.so`; helix-setup must not wipe it.
((controller_body) @injection.content
  (#set! injection.language "typescript"))

; Style `{ … }` chains and `props: { … }` — TS so pad/bg and types paint.
((style_decl
  (brace_expression) @injection.content)
  (#set! injection.language "typescript")
  (#set! injection.include-children))

((props_block
  (brace_expression) @injection.content)
  (#set! injection.language "typescript")
  (#set! injection.include-children))

; ── CSS injection for inline `<style>` blocks ────────────────────────
; The runtime treats `<style src="…"/>` (self-closing, external) as a
; paired CSS file reference and inline `<style>…</style>` as embedded
; CSS. We inject CSS into any element whose tag-name is `style` and
; whose body is a single text node.
((element
  (tag_name) @_t
  (text) @injection.content)
  (#eq? @_t "style")
  (#set! injection.language "css"))
