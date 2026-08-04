; Embedded language injections for `.sail`.
;
; Routes interpolation/binding bodies to TypeScript and inline `<style>`
; blocks to CSS so editors get host-language IntelliSense, formatting,
; and diagnostics inside those ranges. Also injects TypeScript into
; `<controller>` / `<scope>` block bodies (SPEC-LSP-1 highlighting).

; ── TypeScript injections inside expression braces ───────────────────
((interpolation) @injection.content
  (#set! injection.language "typescript")
  (#offset! @injection.content 0 1 0 -1))

((brace_expression) @injection.content
  (#set! injection.language "typescript")
  (#offset! @injection.content 0 1 0 -1))

; ── TypeScript injection for <controller> / <scope> block bodies ─────
; Controller bodies are TypeScript source. Scope bodies hold props /
; style declarations (TS-ish). Both are plain `element` nodes today;
; match on tag_name like the style→css rule. Brace fragments inside the
; body still get TS via the interpolation rule above.
((element
  (tag_name) @_t
  (text) @injection.content)
  (#any-of? @_t "controller" "scope")
  (#set! injection.language "typescript"))

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
