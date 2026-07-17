# props-as-TS: the whole prop surface in one typed block

`props.ts` — the typed `props()` collector + built-in prop descriptors
(`tone`, `size`, `bool`, `string`, `enumOf`). Naming a built-in IS the
declaration + contract opt-in.

`button.ok.ts`   — button's entire prop surface as ONE typed block. tsgo passes.
`button.bad.ts`  — undefined value ref + wrong type → tsgo errors.
`button.shorthand.ts` — a typo'd built-in (`sizee`) → tsgo TS2552
                        "Cannot find name 'sizee'. Did you mean 'size'?"
                        (autocomplete-grade, the whole point).

## Before → after (sail-button)
BEFORE: 9 <prop name=... values=... default=... reflect /> tags + <theme prop="size">.
AFTER:
  <props>{
    tone, size,                                   // built-ins: union + default + reflect + contract
    variant: ['solid','soft','outline','ghost','default'],
    disabled: bool, loading: bool, label: string,
  }</props>

<prop> XML gone. <theme> gone (tone/size built-ins carry the contract). One typed block.

## What's proven vs what's left
Proven: the authoring surface + real tsgo inference (typos, types, suggestions).
Left (compiler): parse the <props> block, generate attribute registration +
wire each built-in's contract; support extra enum values (button's `xs` size);
migrate 76 components.
