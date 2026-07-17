# SUI Template Expressions

`{expr}` regions in `.sui` files are not arbitrary JavaScript. They
parse as a **constrained subset** designed to be stable, statically
checkable, and predictable inside the brace-counting boundaries of
the template parser.

This file is the source of truth for what is and isn't allowed.

## Why a subset

Two reasons.

1. **Termination.** `{...}` regions are scanned by depth-counting
   string-aware brace matchers. If the embedded source can contain
   arbitrary syntax — assignments, function declarations, nested
   blocks — the matcher can't reason about closure cleanly. A defined
   grammar makes the close brace unambiguous and lets the rest of the
   pipeline rely on the substring boundary the grammar reports.
2. **Static checks.** The same source that ships to the runtime is
   re-parsed by the compiler for type and identity checks (see
   `EXPRESSIONS.md → Static checks`). The simpler the surface, the
   tighter those checks can be.

## Where the grammar applies

Three positions:

- **Text interpolation** — `<p>{expr}</p>`. Allowed: full subset
  below. Disallowed: arrow functions.
- **Attribute expression** — `attr={expr}`, `class:foo={expr}`,
  `style:color={expr}`, `prop:foo={expr}`, `bind:value={expr}`.
  Allowed: full subset. Disallowed: arrow functions.
- **Event handler** — `on:click={expr}`. Allowed: full subset
  **plus** arrow functions and statement bodies (`(e) => x.set(1)`,
  `(e) => { x.set(1); y.set(2) }`).

Block headers (`{#if cond}`, `{#for x of xs}`, `{#match disc}`,
`{:case test}`, `{:else if cond}`) wrap a single expression after the
keyword and are subject to the same subset (no arrows).

## Grammar

EBNF, in operator-precedence order. `Expr` is the start symbol.

```
Expr        ::= Ternary
Ternary     ::= Coalesce ( '?' Expr ':' Expr )?
Coalesce    ::= LogicalOr ( '??' LogicalOr )*
LogicalOr   ::= LogicalAnd ( '||' LogicalAnd )*
LogicalAnd  ::= Equality ( '&&' Equality )*
Equality    ::= Relational ( ( '===' | '!==' | '==' | '!=' ) Relational )*
Relational  ::= Additive ( ( '<=' | '>=' | '<' | '>' ) Additive )*
Additive    ::= Multiplicative ( ( '+' | '-' ) Multiplicative )*
Multiplicative
            ::= Unary ( ( '*' | '/' | '%' ) Unary )*
Unary       ::= ( '!' | '-' | '+' )? Postfix
Postfix     ::= Primary ( PostfixOp )*
PostfixOp   ::= '.'  Ident
              | '?.' Ident
              | '?.' '(' ArgList? ')'
              | '?.' '[' Expr ']'
              | '['  Expr ']'
              | '('  ArgList? ')'
ArgList     ::= Expr ( ',' Expr )*
Primary     ::= Number
              | String
              | Template
              | Bool
              | Null
              | Undefined
              | Ident
              | ArrayLit
              | ObjectLit
              | Paren
              | Arrow                  -- handler regions only
ArrayLit    ::= '[' ( Element ( ',' Element )* ','? )? ']'
Element     ::= Spread | Expr
ObjectLit   ::= '{' ( ObjEntry ( ',' ObjEntry )* ','? )? '}'
ObjEntry    ::= Spread
              | ( Ident | String ) ':' Expr
              | Ident                  -- shorthand
Spread      ::= '...' Expr
Paren       ::= '(' Expr ')'
Arrow       ::= ArrowParams '=>' ArrowBody
ArrowParams ::= '(' ParamList? ')' | Ident
ArrowBody   ::= Expr | Block
Block       ::= '{' Stmt ( ';' Stmt )* ';'? '}'
Stmt        ::= Expr                   -- statement-expr only;
                                       -- assignments at statement
                                       -- top-level are allowed in
                                       -- handler bodies for signal
                                       -- writes via `.set(...)`.
Template    ::= '`' ( Char | Escape | '${' Expr '}' )* '`'
String      ::= '"' Char* '"' | "'" Char* "'"
Number      ::= /-? \d+ (\. \d+)? ( [eE] [+-]? \d+ )?/
Ident       ::= /[A-Za-z_$][A-Za-z0-9_$]*/
Bool        ::= 'true' | 'false'
Null        ::= 'null'
Undefined   ::= 'undefined'
```

`Char` excludes the matching quote; `\` escapes the next byte. Unicode
identifiers beyond ASCII are not part of the subset (the compiler
emits the source byte-for-byte so they would still execute, but the
checker won't recognise them as identifiers).

## Allowed semantic constructs

| Construct | Example | Notes |
|-----------|---------|-------|
| Identifier read | `count` | Resolves against `<data>`, `<prop>`, controller `scope` / `derived` / `on` / `use` keys, plus a small builtins allowlist. |
| Property access | `obj.field`, `arr.length` | |
| Optional chaining | `a?.b`, `a?.()`, `a?.[k]` | |
| Indexing | `arr[0]`, `obj['key']` | |
| Zero-arg call | `signal()`, `agents().value` | Reads a signal/derived. |
| Allow-listed method call | `xs.slice(0, 4)`, `xs.map(toName)`, `'abc'.includes('b')` | Allowed methods: `slice`, `map`, `filter`, `includes`, `toString`, `length`, `keys`, `values`, `entries`. |
| Arithmetic | `+ - * / %` | Unary `-` and `+` permitted. |
| Comparison | `< <= > >= === !== == !=` | |
| Logical | `&& \|\| !` | |
| Nullish coalesce | `??` | |
| Ternary | `cond ? a : b` | |
| Literals | `42`, `'foo'`, `"foo"`, `true`, `false`, `null`, `undefined` | |
| Template literal | `` `count=${count()}` `` | Interpolations recurse through the subset. |
| Array literal | `[a, b, ...rest]` | Spread allowed inside collection literals. |
| Object literal | `{ k: v, ...rest }` | Shorthand keys allowed. |
| Parenthesised | `(expr)` | |
| Arrow function | `(e) => x.set(e.target.value)` | **Handlers only.** |

## Disallowed constructs

These are rejected by the parser and/or the post-parse subset check:

- Statements (`;` outside an arrow block body).
- Assignments (`=`) outside event-handler block bodies.
- Increment / decrement (`++`, `--`).
- `new`, `await`, `yield`, `this`, `super`, `delete`, `void`, `function`, `class`.
- Tagged template literals (`` html`...` ``).
- Non-allow-listed method calls with arguments (e.g. `xs.push(3)`).
- Non-allow-listed plain calls with arguments (e.g. `doStuff(a, b)`).
  Helpers must be invoked via `setup:` actions or `derived` keys
  declared in the controller.
- Regular-expression literals (`/foo/`).
- Spread outside an array or object literal.
- Function declarations / class declarations / generator declarations.
- Async / await.

## Static checks

The compiler runs the following passes against every parsed
expression. Each closes a class of bug that "looks like JS so it's
fine" used to hide:

1. **Subset check** — the grammar above is enforced. Source for the
   checker lives in `crates/sail-compiler/src/check/expr_subset.rs`.
2. **Scope binding** — every free identifier resolves to a declared
   name. See `check/scope_bound.rs`.
3. **`scope.X` member typo** — when the head of a member chain is
   `scope`, the second segment must be a key returned by the paired
   controller's `setup:` factory. See `check/scope_member.rs`.
4. **Stable `key=` identity** — `{#for}` `key={expr}` rejects fresh
   array / object literals (always-new identity defeats keyed reuse).
   See `check/key_identity.rs`.
5. **`bind:this` target** — must be a controller-declared `ref(...)`
   in `setup:`. See `check/bind_this_target.rs`.
6. **`prop:foo` typo** — when the child element is a sibling SUI
   component file with declared `<prop>` directives, `prop:foo={…}`
   must match a declared prop name. See `check/prop_target.rs`.
7. **`on:event` callable** — non-arrow handlers must reference an
   `on:` action, a `setup:` action, or a callable in scope. See
   `check/on_callable.rs`.
8. **Mutation chain** — `mutation="ship.X.Y"` is `ship.<ns>.<method>`
   shape only. See `check/mod.rs::validate_mutation_expr`.

## LSP intelligence

The grammar's AST powers `crates/lsp/src/expr_intel.rs`:

- **Hover**: at a cursor inside `{expr}`, the LSP looks up the symbol
  under cursor and reports its declaration source from the paired
  controller (key, kind, line).
- **Definition**: jumps from `scope.foo`, `derived.X`, `data.X`, etc.
  to the corresponding line in `*.controller.ts`.
- **Completion**: at `scope.` (or `derived.`, `data.`, `props.`) the
  LSP returns the harvested key list. At top level it offers the
  full scope ident allow-list plus the small builtin set.

The intelligence layer is purely **symbol-level** in v1: there is no
TypeScript type checking. Cross-file inference beyond the
`controller.ts` ↔ `.sui` pair is out of scope.
