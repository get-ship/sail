# Honest variants: real TS via tsgo (not a hand-parser)

`variants.ts` — a real typed `variants<V>()` function. `V` is the component's
prop→value-union map (the compiler generates it from `<prop values="…">`).

`button.ok.ts` — valid config, `tsgo --noEmit` exits 0.
`button.bad.ts` — three deliberate errors; tsgo catches each with real inference:
  - `solidd` → TS2561 "did you mean 'solid'?"   (value not in the declared union)
  - `sm: 42` → TS2322 number not assignable to string
  - compound `when.variant: 'ghost'` → TS2322 not in the union

## The architecture this proves
1. Compiler extracts the `<variants>` object body + generates `type V` from `<prop values>`.
2. Wraps it: `variants<V>('part', <body>)`, shells to the shipped `tsgo --noEmit`.
3. tsgo validates STRUCTURE + prop VALUES (real inference, suggestions).
4. SUI's existing sx lint validates the sx VALUE STRINGS (tsgo sees them as `string`).
5. Diagnostics map back to .sui spans.

Division of labor: tsgo owns the TypeScript; the sx lint owns sx. No shadowing.
Same mechanism extends to template expressions and (later) the inline controller.
