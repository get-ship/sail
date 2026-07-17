; Tree-sitter highlight scopes for `.sail` template files.
;
; Scope names follow the conventional `tree-sitter-highlight` palette
; (https://tree-sitter.github.io/tree-sitter/syntax-highlighting).
; Editors map them onto their own theme tokens.

; ── HTML element / attribute structure ───────────────────────────────
(tag_name) @tag
(attr_name) @attribute
(attr_local) @attribute
(style_prop_name) @attribute

(string_literal) @string

; Element bracket punctuation.
"<"  @punctuation.bracket
">"  @punctuation.bracket
"</" @punctuation.bracket
"/>" @punctuation.bracket

; ── directive elements (recognised by tag name) ──────────────────────
; Highlight `<data>`, `<component>`, `<prop>`, `<while-loading>`,
; `<while-error>` distinctively so authors can see the runtime
; contract at a glance.
((tag_name) @keyword.directive
  (#match? @keyword.directive "^(data|component|prop|while-loading|while-error|slot)$"))

; ── binding attribute prefixes ───────────────────────────────────────
"bind:"  @keyword.directive
"on:"    @keyword.directive
"class:" @keyword.directive
"style:" @keyword.directive
"attr:"  @keyword.directive
"let:"   @keyword.directive
"|"      @punctuation.delimiter

; ── template control flow markers ────────────────────────────────────
"{#if"      @keyword.directive
"{:elif"    @keyword.directive
"{:else if" @keyword.directive
"{:else}"   @keyword.directive
"{/if}"     @keyword.directive
"{#for"     @keyword.directive
"of"        @keyword.directive
"key"       @keyword.directive
"{/for}"    @keyword.directive
"{#match"   @keyword.directive
"{:case"    @keyword.directive
"{:default}" @keyword.directive
"{/match}"  @keyword.directive

; Brace boundaries.
"{" @punctuation.special
"}" @punctuation.special

; ── expression body (inside `{…}`) ───────────────────────────────────
; Identifiers in expression position are variable reads; member
; access shows up as a single `.foo` token (the `_member` rule).
(interpolation (identifier) @variable)
(brace_expression (identifier) @variable)
(if_block cond: (identifier) @variable)
(for_block iter: (identifier) @variable)
(match_block value: (identifier) @variable)
(case_clause value: (identifier) @variable)

; For-loop binding pattern names are locals introduced by the loop.
(for_pattern (identifier) @variable.parameter)

; let:NAME — slot binding name.
(let_attr name: (attr_local) @variable.parameter)

; ── comments ─────────────────────────────────────────────────────────
(html_comment) @comment
